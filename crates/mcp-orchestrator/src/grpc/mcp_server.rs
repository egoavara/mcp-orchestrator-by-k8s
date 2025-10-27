// use proto::mcp::orchestrator::v1::*;
// use tonic::{Request, Response, Status};

// use crate::state::AppState;
// use crate::storage::mcp_server_store::McpServerStore;

// pub async fn get_mcp_server(
//     state: &AppState,
//     request: Request<GetMcpServerRequest>,
// ) -> Result<Response<McpServerResponse>, Status> {
//     let req = request.into_inner();
//     let store = McpServerStore::new(
//         state.kube_client.clone(),
//         state.default_namespace.clone(),
//     );

//     let namespace = req.namespace.as_deref().unwrap_or_else(|| store.default_namespace());

//     let pod = store
//         .get(namespace, &req.id)
//         .await
//         .map_err(|e| Status::internal(format!("Failed to get MCP server: {}", e)))?
//         .ok_or_else(|| Status::not_found(format!("MCP Server {} not found", req.id)))?;

//     let status = if let Some(pod_status) = &pod.status {
//         match pod_status.phase.as_deref() {
//             Some("Running") => McpServerStatus::Running as i32,
//             Some("Pending") => McpServerStatus::Pending as i32,
//             Some("Failed") => McpServerStatus::Failed as i32,
//             Some("Succeeded") | Some("Terminated") => McpServerStatus::Terminated as i32,
//             _ => McpServerStatus::Unspecified as i32,
//         }
//     } else {
//         McpServerStatus::Pending as i32
//     };

//     let created_at = pod
//         .metadata
//         .creation_timestamp
//         .as_ref()
//         .and_then(|ts| ts.0.timestamp().try_into().ok())
//         .unwrap_or(0);

//     let started_at = pod
//         .status
//         .as_ref()
//         .and_then(|s| s.start_time.as_ref())
//         .and_then(|ts| ts.0.timestamp().try_into().ok());

//     let pod_ip = pod.status.as_ref().and_then(|s| s.pod_ip.clone());
//     let node_name = pod.spec.as_ref().and_then(|s| s.node_name.clone());
//     let pod_name = pod.metadata.name.clone();

//     Ok(Response::new(McpServerResponse {
//         id: req.id,
//         name: pod_name.clone().unwrap_or_default(),
//         namespace: pod.metadata.namespace.unwrap_or_default(),
//         template_id: String::new(),
//         status,
//         image: String::new(),
//         command: vec![],
//         args: vec![],
//         env_vars: vec![],
//         resource_limit_name: String::new(),
//         volumes: vec![],
//         labels: pod.metadata.labels.unwrap_or_default().into_iter().collect(),
//         created_at,
//         started_at,
//         pod_name,
//         pod_ip,
//         node_name,
//     }))
// }

// pub async fn list_mcp_servers(
//     state: &AppState,
//     request: Request<ListMcpServersRequest>,
// ) -> Result<Response<ListMcpServersResponse>, Status> {
//     let req = request.into_inner();
//     let store = McpServerStore::new(
//         state.kube_client.clone(),
//         state.default_namespace.clone(),
//     );

//     let namespace = req.namespace.as_deref();

//     let pods = store
//         .list(namespace, &[])
//         .await
//         .map_err(|e| Status::internal(format!("Failed to list MCP servers: {}", e)))?;

//     let responses: Vec<McpServerResponse> = pods
//         .into_iter()
//         .map(|pod| {
//             let status = if let Some(pod_status) = &pod.status {
//                 match pod_status.phase.as_deref() {
//                     Some("Running") => McpServerStatus::Running as i32,
//                     Some("Pending") => McpServerStatus::Pending as i32,
//                     Some("Failed") => McpServerStatus::Failed as i32,
//                     Some("Succeeded") | Some("Terminated") => McpServerStatus::Terminated as i32,
//                     _ => McpServerStatus::Unspecified as i32,
//                 }
//             } else {
//                 McpServerStatus::Pending as i32
//             };

//             let created_at = pod
//                 .metadata
//                 .creation_timestamp
//                 .as_ref()
//                 .and_then(|ts| ts.0.timestamp().try_into().ok())
//                 .unwrap_or(0);

//             let started_at = pod
//                 .status
//                 .as_ref()
//                 .and_then(|s| s.start_time.as_ref())
//                 .and_then(|ts| ts.0.timestamp().try_into().ok());

//             let pod_ip = pod.status.as_ref().and_then(|s| s.pod_ip.clone());
//             let node_name = pod.spec.as_ref().and_then(|s| s.node_name.clone());
//             let pod_name = pod.metadata.name.clone();

//             McpServerResponse {
//                 id: pod_name.clone().unwrap_or_default().strip_prefix("mcp-server-").unwrap_or("").to_string(),
//                 name: pod_name.clone().unwrap_or_default(),
//                 namespace: pod.metadata.namespace.unwrap_or_default(),
//                 template_id: String::new(),
//                 status,
//                 image: String::new(),
//                 command: vec![],
//                 args: vec![],
//                 env_vars: vec![],
//                 resource_limit_name: String::new(),
//                 volumes: vec![],
//                 labels: pod.metadata.labels.unwrap_or_default().into_iter().collect(),
//                 created_at,
//                 started_at,
//                 pod_name,
//                 pod_ip,
//                 node_name,
//             }
//         })
//         .collect();

//     let total = responses.len() as i32;

//     Ok(Response::new(ListMcpServersResponse {
//         servers: responses,
//         next_page_token: String::new(),
//         total_count: total,
//     }))
// }
