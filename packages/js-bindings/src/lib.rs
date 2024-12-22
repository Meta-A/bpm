use std::sync::Arc;

use bpm_core::{
    blockchains::{errors::blockchain_error::BlockchainError, get_available_clients},
    config::init_config,
    db::client::DbClient,
    services::{
        blockchains::BlockchainsService,
        db::{
            blockchains_repository::BlockchainsRepository, packages_repository::PackagesRepository,
        },
        packages::PackagesService,
    },
};
use home::home_dir;
use neon::prelude::*;
use once_cell::sync::OnceCell;
use tokio::{runtime::Runtime, sync::mpsc};

// Return a global tokio runtime or create one if it doesn't exist.
// Throws a JavaScript exception if the `Runtime` fails to create.
fn runtime<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<&'static Runtime> {
    static RUNTIME: OnceCell<Runtime> = OnceCell::new();

    RUNTIME.get_or_try_init(|| Runtime::new().or_else(|err| cx.throw_error(err.to_string())))
}

fn fetch_package(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let rt = runtime(&mut cx)?;
    let package_name = cx.argument::<JsString>(0)?.value(&mut cx);
    let package_version = cx.argument::<JsString>(1)?.value(&mut cx);

    let channel = cx.channel();

    let (deferred, promise) = cx.promise();

    rt.spawn(async move {
        let blockchains_service = init().await;

        let packages = blockchains_service
            .find_package(&package_name, &package_version)
            .await;

        deferred.settle_with(&channel, move |mut cx| {
            let packages_objects: Vec<JsObject> = Vec::new();

            let packages_objects = JsArray::new(&mut cx, packages_objects.len());
            for (i, package) in packages.iter().enumerate() {
                let package_obj = JsObject::new(&mut cx);

                let package_obj_name_key = cx.string("package_name");
                let package_obj_name_value = cx.string(&package.name);

                package_obj.set(&mut cx, package_obj_name_key, package_obj_name_value)?;

                let package_obj_version_key = cx.string("package_version");
                let package_obj_version_value = cx.string(&package.version);

                package_obj.set(&mut cx, package_obj_version_key, package_obj_version_value)?;

                let package_obj_maintainer_key = cx.string("maintainer");
                let package_obj_maintainer_value = cx.string(hex::encode(&package.maintainer));

                package_obj.set(
                    &mut cx,
                    package_obj_maintainer_key,
                    package_obj_maintainer_value,
                )?;

                let index = u32::try_from(i).or_else(|err| cx.throw_error(err.to_string()))?;

                packages_objects.set(&mut cx, index, package_obj)?;
            }

            Ok(packages_objects)
        });
    });

    Ok(promise)
}

async fn update(blockchains_service: &Arc<BlockchainsService>) {
    let (tx_packages_update, mut rx_packages_update) = mpsc::channel(1);

    let task_blockchains_service_ref = Arc::clone(&blockchains_service);
    tokio::spawn(async move {
        let task_res = task_blockchains_service_ref.update(&tx_packages_update);

        match task_res.await {
            Ok(_) => return,
            Err(e) => match e {
                BlockchainError::NoPackagesData => {
                    // TODO console error
                }
                _ => {
                    // TODO : console error
                }
            },
        }
    });

    let mut packages_count: u128 = 0;

    while let Some(package) = rx_packages_update.recv().await {
        // TODO : console log
    }
}

async fn init() -> Arc<BlockchainsService> {
    let config_path = home_dir().unwrap();

    let config_manager = init_config(&config_path);

    let db_client = Arc::new(DbClient::from(&config_manager.get_db_path()));

    // Blockchains clients
    let available_blockchains = get_available_clients();

    // Repositories
    let blockchains_repository = Arc::new(BlockchainsRepository::from(&db_client));
    let packages_repository = Arc::new(PackagesRepository::from(&db_client));

    let packages_service = Arc::new(PackagesService::from(&packages_repository));

    let blockchains_service = Arc::new(
        BlockchainsService::new(
            &available_blockchains,
            &blockchains_repository,
            &packages_service,
        )
        .await,
    );

    blockchains_service.init_blockchains().await;

    // TODO : test, change it later
    blockchains_service.set_client(0).await;

    update(&blockchains_service).await;

    Arc::clone(&blockchains_service)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("fetchPackage", fetch_package)?;
    Ok(())
}
