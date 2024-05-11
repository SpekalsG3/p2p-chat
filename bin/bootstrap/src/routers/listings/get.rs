use axum::extract::State;
use axum::Json;
use serde::Serialize;
use crate::types::{ApiResponse, AppStateRc};

#[derive(Serialize)]
pub struct GetListingsRes {
    count: usize,
    servers: Vec<String>,
}

fn exec(
    state: AppStateRc,
) -> Result<GetListingsRes, String> {
    let lock = state.read().map_err(|e| format!("Failed to get read lock: {e}"))?;
    Ok(GetListingsRes {
        count: lock.servers.len(),
        servers: lock.servers.keys().map(|addr| addr.to_string()).collect(),
    })
}

pub async fn get_listings(
    State(state): State<AppStateRc>,
) -> Json<ApiResponse<GetListingsRes, String>> {
    let res = exec(state);

    let mut json = ApiResponse {
        success: res.is_ok(),
        data: None,
        error: None,
    };
    if json.success {
        json.data = res.ok();
    } else {
        json.error = res.err();
    }
    Json(json)
}
