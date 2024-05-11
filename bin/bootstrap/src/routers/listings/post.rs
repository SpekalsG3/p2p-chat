use std::net::SocketAddr;
use std::str::FromStr;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::types::{ApiResponse, AppStateRc};
use crate::utils::extract_body::ExtractBody;

#[derive(Deserialize)]
pub struct CreateListingReq {
    server_addr: String,
}

#[derive(Serialize)]
pub struct CreateListingRes {
}

fn exec(
    state: AppStateRc,
    body: Value,
) -> Result<CreateListingRes, (StatusCode, String)> {
    let req = serde_json::from_value::<CreateListingReq>(body)
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, format!("Failed to parse query: {e}")))?;

    let addr = SocketAddr::from_str(&req.server_addr)
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, format!("Failed to parse socket addr: {e}")))?;

    {
        let mut lock = state.write()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get write lock: {e}")))?;

        if lock.servers.contains_key(&addr) {
            return Err((StatusCode::BAD_REQUEST, "Server address was already committed".to_string()))
        }

        lock.servers.insert(addr, ());
    };

    Ok(CreateListingRes {
    })
}

pub async fn create_listing(
    State(state): State<AppStateRc>,
    body: ExtractBody<Value>,
) -> impl IntoResponse {
    let res = exec(state, body.0);

    let mut status_code = StatusCode::OK;
    let mut json = ApiResponse {
        success: res.is_ok(),
        data: None,
        error: None,
    };
    match res {
        Ok(data) => { json.data = Some(data) },
        Err(err) => {
            status_code = err.0;
            json.error = Some(err.1);
        }
    }
    (status_code, Json(json))
}
