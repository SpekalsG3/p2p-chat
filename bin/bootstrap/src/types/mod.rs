use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<Success, Error> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Success>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
}

pub struct NodeConfig {
    pub addr: SocketAddr,
    pub network_name: String,
}

pub struct AppState {
    pub network_name: String,
    pub servers: HashMap<SocketAddr, ()>,
}

pub type AppStateRc = Arc<RwLock<AppState>>;
