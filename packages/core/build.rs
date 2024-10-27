use std::path::{Path, PathBuf};

fn main() {
    // TODO : Move to hedera dir then import here if feature enabled
    let hedera_package_path = PathBuf::from(Path::new("src").join("blockchains").join("hedera"));
    let hedera_protos_path = hedera_package_path.join("hedera-protobufs");
    let hedera_protos_services_path = hedera_protos_path.join("services");
    let hedera_protos_mirror_path = hedera_protos_path.join("mirror");

    tonic_build::configure()
        //.build_server(false)
        .build_server(true) // TODO : Only enable when debug for tests
        .include_file("mirror.rs")
        .compile_protos(
            &[
                // Mirror
                hedera_protos_mirror_path.join("consensus_service.proto"),
            ],
            &[hedera_protos_mirror_path, hedera_protos_services_path],
        )
        .unwrap();
}
