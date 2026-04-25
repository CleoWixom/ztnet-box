use crate::{
    server::{error::ApiError, state::AppState, validate},
    zerotier::central::types::*,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

async fn client(
    state: &AppState,
) -> Result<crate::zerotier::central::client::ZtCentralClient, ApiError> {
    state
        .token_store
        .active_client()
        .await
        .ok_or_else(|| ApiError::NoActiveToken)
}

pub async fn list_networks(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(client(&s).await?.networks().await?))
}

pub async fn create_network(
    State(s): State<AppState>,
    Json(body): Json<NetworkCreateOrUpdate>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(client(&s).await?.create_network(&body).await?))
}

pub async fn get_network(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    validate::network_id(&id)?;
    Ok(Json(client(&s).await?.network(&id).await?))
}

pub async fn update_network(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<NetworkCreateOrUpdate>,
) -> Result<impl IntoResponse, ApiError> {
    validate::network_id(&id)?;
    Ok(Json(client(&s).await?.update_network(&id, &body).await?))
}

pub async fn delete_network(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    validate::network_id(&id)?;
    client(&s).await?.delete_network(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_members(
    State(s): State<AppState>,
    Path(net_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    validate::network_id(&net_id)?;
    Ok(Json(client(&s).await?.network_members(&net_id).await?))
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
    Json(body): Json<CentralMemberUpdate>,
) -> Result<impl IntoResponse, ApiError> {
    validate::network_id(&net_id)?;
    validate::node_id(&node_id)?;
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
    client(&s).await?.delete_member(&net_id, &node_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_user(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(client(&s).await?.user().await?))
}

pub async fn get_status(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(client(&s).await?.account_status().await?))
}
