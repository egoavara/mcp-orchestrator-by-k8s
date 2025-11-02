use chrono::Duration;
use proto::mcp::orchestrator::v1::{
    AuthorizationType, GenerateTokenRequest, GenerateTokenResponse,
};
use tonic::{Request, Response, Status};

use crate::{grpc::utils::ProtoWktTime, state::AppState};

pub async fn generate_token(
    state: &AppState,
    request: Request<GenerateTokenRequest>,
) -> Result<Response<GenerateTokenResponse>, Status> {
    let req = request.into_inner();
    let store = state.kube_store.authorization(req.namespace.clone());
    //
    if req.expire_duration.is_none() && !state.config.auth.allow_expireless_token {
        return Err(Status::invalid_argument(
            "expire_duration is not allowed, server requires expire_duration to be set",
        ));
    }
    //
    let expire_duration = req
        .expire_duration
        .map(|dur| {
            let dur = Duration::new(dur.seconds, dur.nanos as u32).unwrap();
            if dur > Duration::days(365) {
                return Err(Status::invalid_argument(
                    "expire_duration cannot be more than 365 days",
                ));
            }
            Ok(dur)
        })
        .transpose()?;
    //
    let sa = store
        .get(&req.name)
        .await
        .map_err(|e| Status::internal(format!("Failed to get authorization: {}", e)))?
        .ok_or_else(|| Status::not_found(format!("Authorization {} not found", req.name)))?;
    //
    if sa.r#type != AuthorizationType::KubernetesServiceAccount {
        return Err(Status::invalid_argument(format!(
            "Authorization {} is not of type KubernetesServiceAccount",
            req.name
        )));
    }
    let (token, expire_at) = store
        .generate_token(&req.name, &state.config.auth.audience, expire_duration)
        .await
        .map_err(|e| Status::internal(format!("Failed to generate token: {}", e)))?;

    Ok(Response::new(GenerateTokenResponse {
        token,
        expire_at: Some(expire_at.to_wkt_time()),
    }))
}
