use axum::{
    routing::get,
    Router, Json, Extension,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use crate::web::stream::WebState;
use crate::web::PlotSettings;

pub async fn create_router(state: Arc<WebState>) -> Router {
    Router::new()
        .route("/api/settings", get(get_settings).post(update_settings))
        .route("/api/channels", get(get_channels))
        .route("/ws", get(crate::web::stream::ws_handler))
        .layer(Extension(state))
        .layer(CorsLayer::permissive())
}

async fn get_settings(Extension(state): Extension<Arc<WebState>>) -> Json<PlotSettings> {
    let settings = state.settings.read().unwrap();
    Json(settings.clone())
}

async fn update_settings(
    Extension(state): Extension<Arc<WebState>>,
    Json(new_settings): Json<PlotSettings>,
) -> Json<PlotSettings> {
    let mut settings = state.settings.write().unwrap();
    *settings = new_settings.clone();
    Json(new_settings)
}

async fn get_channels() -> Json<Vec<String>> {
    Json(vec!["SHZ".to_string(), "EHZ".to_string(), "ENE".to_string()])
}