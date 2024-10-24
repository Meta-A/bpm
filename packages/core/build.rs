use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hedera_package_path = PathBuf::from(Path::new("src").join("blockchains").join("hedera"));
    let hedera_protos_path = hedera_package_path.join("protos");

    tonic_build::configure()
        .build_server(false)
        .include_file("mod.rs")
        .compile_protos(
            &[
                hedera_protos_path.join("timestamp.proto"),
                hedera_protos_path.join("basic_types.proto"),
                hedera_protos_path.join("consensus_submit_message.proto"),
                hedera_protos_path.join("consensus_service.proto"),
            ],
            &[hedera_package_path.join("protos")],
        )?;

    Ok(())
}

