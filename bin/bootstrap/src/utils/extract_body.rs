use axum::extract::{FromRequest, Request};
use axum::{Json, RequestExt};
use axum::extract::rejection::JsonRejection;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use crate::types::ApiResponse;

pub struct ExtractBody<T>(pub T);

#[async_trait::async_trait]
impl<S, T> FromRequest<S> for ExtractBody<T>
    where
        Json<T>: FromRequest<S, Rejection = JsonRejection>,
        T: DeserializeOwned + 'static,
        S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let err = match req.extract().await {
            Ok(Json(payload)) => return Ok(Self(payload)),
            Err(e) => e.to_string(),
        };

        Err(Json(ApiResponse {
            success: false,
            data: None::<()>,
            error: Some(err),
        }).into_response())
    }
}
