use prost_wkt_build::{FileDescriptorSet, Message};
use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../../protobuf");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let descriptor_file = out.join("descriptors.bin");

    let mut config = prost_build::Config::new();
    config.compile_well_known_types();
    config.extern_path(".google.protobuf", "::prost_wkt_types");
    config.file_descriptor_set_path(&descriptor_file);
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    config.compile_protos(
        &[
            "../../protobuf/common.proto",
            "../../protobuf/namespace.proto",
            "../../protobuf/mcp_template.proto",
            "../../protobuf/mcp_server.proto",
            "../../protobuf/secret.proto",
            "../../protobuf/resource_limit.proto",
            "../../protobuf/authorization.proto",
        ],
        &["../../protobuf"],
    )?;

    let descriptor_bytes = std::fs::read(descriptor_file).unwrap();
    let descriptor = FileDescriptorSet::decode(&descriptor_bytes[..]).unwrap();
    prost_wkt_build::add_serde(out, descriptor);

    Ok(())
}
