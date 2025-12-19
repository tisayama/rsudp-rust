use axum::{
    routing::{get},
    extract::State,
    Router, Json,
};
use tower_http::cors::CorsLayer;
use crate::web::stream::{WebState, PlotSettings};

pub async fn create_router(state: WebState) -> Router {
    Router::new()
        .route("/api/settings", get(get_settings).post(update_settings))
        .route("/ws", get(crate::web::stream::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn get_settings(State(state): State<WebState>) -> Json<PlotSettings> {
    let settings = state.settings.read().unwrap();
    Json(settings.clone())
}

async fn update_settings(
    State(state): State<WebState>,
    Json(new_settings): Json<PlotSettings>,
) -> Json<PlotSettings> {
    let mut settings = state.settings.write().unwrap();
    *settings = new_settings;
    Json(settings.clone())
}