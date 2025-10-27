pub mod mcp {
    pub mod orchestrator {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/mcp.orchestrator.v1.rs"));
        }
    }
}

pub use mcp::orchestrator::v1::*;
