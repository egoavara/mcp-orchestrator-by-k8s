use std::{env, path::PathBuf};

use prost_wkt_build::{FileDescriptorSet, Message};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../../protobuf");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let descriptor_file = out.join("descriptors.bin");

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", "::prost_wkt_types")
        .file_descriptor_set_path(&descriptor_file)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&["../../protobuf/service.proto"], &["../../protobuf"])?;

    let descriptor_bytes = std::fs::read(descriptor_file).unwrap();

    let descriptor = FileDescriptorSet::decode(&descriptor_bytes[..]).unwrap();

    prost_wkt_build::add_serde(out, descriptor);
    Ok(())
}
