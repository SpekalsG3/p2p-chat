pub mod get;
pub mod post;

use axum::Router;
use axum::routing::get;
use crate::routers::listings::get::get_listings;
use crate::routers::listings::post::create_listing;
use crate::types::AppStateRc;

pub(crate) fn get_router() -> Router<AppStateRc> {
    Router::new()
        .route("/", get(get_listings).post(create_listing))
}
