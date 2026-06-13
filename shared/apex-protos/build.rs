use std::io::Result;

fn main() -> Result<()> {
    let proto_dir = "../protobuf";

    let proto_files = [
        "common.proto",
        "signal.proto",
        "risk.proto",
        "execution.proto",
        "position.proto",
        "portfolio.proto",
        "analytics.proto",
        "learning.proto",
        "events.proto",
    ];

    let paths: Vec<String> = proto_files
        .iter()
        .map(|f| format!("{}/{}", proto_dir, f))
        .collect();

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .out_dir("src/generated")
        .compile(&paths, &[proto_dir.into()])?;

    println!("cargo:rerun-if-changed={}", proto_dir);

    Ok(())
}
