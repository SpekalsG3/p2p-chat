pub mod listings;

use axum::Router;
use crate::types::AppStateRc;

pub fn get_router() -> Router<AppStateRc> {
    Router::new()
        .nest("/listings", listings::get_router())
}
