pub mod mcp {
    pub mod orchestrator {
        pub mod v1 {
            tonic::include_proto!("mcp.orchestrator.v1");
        }
    }
}
pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/descriptors.bin"));
