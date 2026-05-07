use crate::{
    server::{error::ApiError, state::AppState, validate},
    zerotier::local::{client::ZtLocalClient, types::*},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

async fn client(state: &AppState) -> Result<ZtLocalClient, ApiError> {
    // Use the cached client; fall back to re-creating from config if cache is empty
    // (e.g. authtoken.secret wasn't available at startup)
    if let Some(c) = state.zt_local.read().await.clone() {
        return Ok(c);
    }
    let cfg = state.config.read().await;
    let c = ZtLocalClient::from_config(&cfg.zerotier.local)?;
    drop(cfg);
    *state.zt_local.write().await = Some(c.clone());
    Ok(c)
}

pub async fn node_status(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(client(&s).await?.node_status().await?))
}

pub async fn list_networks(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(client(&s).await?.networks().await?))
}

pub async fn get_network(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    validate::network_id(&id)?;
    Ok(Json(client(&s).await?.network(&id).await?))
}

pub async fn join_network(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<NetworkMembershipUpdate>,
) -> Result<impl IntoResponse, ApiError> {
    validate::network_id(&id)?;
    tracing::info!(network_id = %id, "joining ZeroTier network");
    Ok(Json(client(&s).await?.join_network(&id, &body).await?))
}

pub async fn leave_network(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    validate::network_id(&id)?;
    tracing::info!(network_id = %id, "leaving ZeroTier network");
    client(&s).await?.leave_network(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_peers(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(client(&s).await?.peers().await?))
}

pub async fn get_peer(
    State(s): State<AppState>,
    Path(node_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    validate::node_id(&node_id)?;
    Ok(Json(client(&s).await?.peer(&node_id).await?))
}

pub async fn list_controller_networks(
    State(s): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(client(&s).await?.controller_networks().await?))
}

pub async fn create_controller_network(
    State(s): State<AppState>,
    Json(body): Json<ControllerNetworkCreate>,
) -> Result<impl IntoResponse, ApiError> {
    let cl = client(&s).await?;
    let status = cl.node_status().await?;
    tracing::info!("creating controller network");
    Ok(Json(
        cl.create_controller_network(&status.address, &body).await?,
    ))
}

pub async fn get_controller_network(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    validate::network_id(&id)?;
    Ok(Json(client(&s).await?.controller_network(&id).await?))
}

pub async fn update_controller_network(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ControllerNetworkCreate>,
) -> Result<impl IntoResponse, ApiError> {
    validate::network_id(&id)?;
    Ok(Json(
        client(&s)
            .await?
            .update_controller_network(&id, &body)
            .await?,
    ))
}

pub async fn delete_controller_network(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    validate::network_id(&id)?;
    tracing::info!(network_id = %id, "deleting controller network");
    client(&s).await?.delete_controller_network(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_members(
    State(s): State<AppState>,
    Path(net_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    validate::network_id(&net_id)?;
    Ok(Json(client(&s).await?.network_members_bulk(&net_id).await?))
}

pub async fn get_member(
    State(s): State<AppState>,
    Path((net_id, node_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    validate::network_id(&net_id)?;
    validate::node_id(&node_id)?;
    Ok(Json(
        client(&s).await?.network_member(&net_id, &node_id).await?,
    ))
}

pub async fn update_member(
    State(s): State<AppState>,
    Path((net_id, node_id)): Path<(String, String)>,
    Json(body): Json<ControllerMemberUpdate>,
) -> Result<impl IntoResponse, ApiError> {
    validate::network_id(&net_id)?;
    validate::node_id(&node_id)?;
    tracing::info!(network_id = %net_id, node_id = %node_id, "updating controller member");
    Ok(Json(
        client(&s)
            .await?
            .update_member(&net_id, &node_id, &body)
            .await?,
    ))
}

pub async fn delete_member(
    State(s): State<AppState>,
    Path((net_id, node_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    validate::network_id(&net_id)?;
    validate::node_id(&node_id)?;
    tracing::info!(network_id = %net_id, node_id = %node_id, "deleting controller member");
    client(&s).await?.delete_member(&net_id, &node_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_moons(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(client(&s).await?.moons().await?))
}

pub async fn orbit_moon(
    State(s): State<AppState>,
    Path(world_id): Path<String>,
    Json(body): Json<OrbitRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate::world_id(&world_id)?;
    tracing::info!(world_id = %world_id, "orbiting moon");
    Ok(Json(client(&s).await?.orbit_moon(&world_id, &body).await?))
}

pub async fn deorbit_moon(
    State(s): State<AppState>,
    Path(world_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    validate::world_id(&world_id)?;
    tracing::info!(world_id = %world_id, "deorbiting moon");
    client(&s).await?.deorbit_moon(&world_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
