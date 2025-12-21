use crate::web::stream::{PlotSettings, WebState};
use crate::web::alerts::{AlertEvent, AlertSettings};
use axum::{Json, Router, extract::{State, Path}, routing::get, routing::put};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

pub async fn create_router(state: WebState) -> Router {
    Router::new()
        .route("/api/settings", get(get_settings).post(update_settings))
        .route("/api/alerts", get(get_alert_history))
        .route("/api/alerts/settings", get(get_alert_settings).put(update_alert_settings))
        .nest_service("/images/alerts", ServeDir::new("alerts"))
        .route("/ws", get(crate::web::stream::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn get_alert_history(State(state): State<WebState>) -> Json<Vec<AlertEvent>> {
    let history = state.history.lock().unwrap();
    Json(history.get_events())
}

async fn get_alert_settings(State(state): State<WebState>) -> Json<AlertSettings> {
    let history = state.history.lock().unwrap();
    Json(history.get_settings())
}

async fn update_alert_settings(
    State(state): State<WebState>,
    Json(new_settings): Json<AlertSettings>,
) -> Json<AlertSettings> {
    let mut history = state.history.lock().unwrap();
    history.update_settings(new_settings.clone());
    Json(new_settings)
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
