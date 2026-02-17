use crate::web::stream::{PlotSettings, WebState};
use crate::web::alerts::{AlertEvent, AlertSettings};
use axum::{Json, Router, extract::{Query, State}, routing::get};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

pub async fn create_router(state: WebState) -> Router {
    Router::new()
        .route("/api/settings", get(get_settings).post(update_settings))
        .route("/api/channels", get(get_channels))
        .route("/api/station", get(get_station_name))
        .route("/api/alerts", get(get_alert_history))
        .route("/api/alerts/settings", get(get_alert_settings).put(update_alert_settings))
        .route("/api/capture/data", get(get_capture_data))
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

async fn get_station_name(State(state): State<WebState>) -> Json<String> {
    let name = state.station_name.read().unwrap();
    Json(name.clone())
}

async fn get_channels(State(state): State<WebState>) -> Json<Vec<String>> {
    let buffers = state.waveform_buffers.lock().unwrap();
    let mut channels: Vec<String> = buffers.keys().cloned().collect();
    channels.sort();
    Json(channels)
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

// --- Capture Data API (T003/T004) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureDataResponse {
    pub station: String,
    pub sample_rate: f64,
    pub channels: HashMap<String, ChannelWaveform>,
    pub spectrogram: HashMap<String, ChannelSpectrogram>,
    pub sensitivity: HashMap<String, f64>,
    pub settings: CaptureDataPlotSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelWaveform {
    pub samples: Vec<f64>,
    pub start_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSpectrogram {
    pub columns: Vec<Vec<u8>>,
    pub frequency_bins: usize,
    pub hop_duration: f64,
    pub first_column_timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureDataPlotSettings {
    pub filter_waveform: bool,
    pub filter_highpass: f64,
    pub filter_lowpass: f64,
    pub filter_corners: usize,
    pub deconvolve: bool,
    pub units: String,
    pub spectrogram_freq_min: f64,
    pub spectrogram_freq_max: f64,
    pub spectrogram_log_y: bool,
}

#[derive(Debug, Deserialize)]
struct CaptureDataQuery {
    channels: Option<String>,
    start: Option<String>,
    end: Option<String>,
}

async fn get_capture_data(
    State(state): State<WebState>,
    Query(params): Query<CaptureDataQuery>,
) -> Result<Json<CaptureDataResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Validate required params
    let channels_str = params.channels.as_deref().unwrap_or("");
    let channel_list: Vec<String> = channels_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if channel_list.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "channels parameter is required"})),
        ));
    }

    let start_str = params.start.as_deref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "start parameter is required"})))
    })?;
    let end_str = params.end.as_deref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "end parameter is required"})))
    })?;

    let start = DateTime::parse_from_rfc3339(start_str)
        .map_err(|_| {
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "invalid start time format, expected ISO 8601"})))
        })?
        .with_timezone(&Utc);
    let end = DateTime::parse_from_rfc3339(end_str)
        .map_err(|_| {
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "invalid end time format, expected ISO 8601"})))
        })?
        .with_timezone(&Utc);

    let duration_sec = (end - start).num_milliseconds() as f64 / 1000.0;
    if duration_sec <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "end must be after start"})),
        ));
    }

    // Extract raw data from buffers (hold lock briefly)
    struct RawChannelData {
        samples: Vec<f64>,
        actual_start: DateTime<Utc>,
        sample_rate: f64,
    }
    let raw_data: HashMap<String, RawChannelData> = {
        let buffers = state.waveform_buffers.lock().unwrap();
        channel_list
            .iter()
            .filter_map(|ch| {
                buffers.get(ch).and_then(|buffer| {
                    let (samples, actual_start) = buffer.extract_window(start, duration_sec);
                    if samples.is_empty() {
                        None
                    } else {
                        Some((
                            ch.clone(),
                            RawChannelData {
                                samples,
                                actual_start,
                                sample_rate: buffer.sample_rate,
                            },
                        ))
                    }
                })
            })
            .collect()
    };

    if raw_data.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "no data available for the specified time range"})),
        ));
    }

    // Read deconvolution settings and instrument response
    let (deconvolve, units) = {
        let ps = state.settings.read().unwrap();
        (ps.deconvolve, ps.units.clone())
    };
    let resp_map = {
        let rm = state.response_map.read().unwrap();
        rm.clone()
    };
    let sens_map = {
        let sm = state.sensitivity_map.read().unwrap();
        sm.clone()
    };

    // Compute waveforms and spectrograms outside lock
    let nfft: usize = 128;
    let hop: usize = 13;
    let noverlap = nfft - hop;
    let mut channels_map = HashMap::new();
    let mut spectrogram_map = HashMap::new();
    let mut sample_rate = 100.0;
    let deconv_water_level = 4.5; // dB, matches rsudp default

    for (ch, data) in &raw_data {
        sample_rate = data.sample_rate;

        // Apply deconvolution if enabled (same logic as WebSocket stream)
        let waveform_samples = if deconvolve {
            if let Some(response) = resp_map.get(ch) {
                if !response.poles.is_empty() {
                    let pre_filt = [0.1, 0.6, 0.95 * data.sample_rate, data.sample_rate];
                    let mut deconv = crate::filter::deconvolve_response(
                        &data.samples, response, data.sample_rate, pre_filt, deconv_water_level,
                    );
                    // Match rsudp: demean the deconvolved signal
                    let mean = deconv.iter().sum::<f64>() / deconv.len().max(1) as f64;
                    for s in &mut deconv {
                        *s -= mean;
                    }
                    if units.to_uppercase() == "GRAV" {
                        for s in &mut deconv {
                            *s /= 9.81;
                        }
                    }
                    deconv
                } else {
                    // Fallback: simple scalar division by sensitivity
                    let sensitivity = sens_map.get(ch).copied().unwrap_or(1.0);
                    let extra_divisor = if units.to_uppercase() == "GRAV" { 9.81 } else { 1.0 };
                    if sensitivity > 0.0 {
                        data.samples.iter().map(|&s| s / sensitivity / extra_divisor).collect()
                    } else {
                        data.samples.clone()
                    }
                }
            } else {
                data.samples.clone()
            }
        } else {
            data.samples.clone()
        };

        channels_map.insert(
            ch.clone(),
            ChannelWaveform {
                samples: waveform_samples,
                start_time: data.actual_start,
            },
        );

        if data.samples.len() >= nfft {
            let spec = crate::web::spectrogram::compute_spectrogram(
                &data.samples,
                data.sample_rate,
                nfft,
                noverlap,
            );

            if !spec.data.is_empty() {
                // Power-law compression: PSD^0.1 (matches rsudp)
                let compressed: Vec<Vec<f64>> = spec
                    .data
                    .iter()
                    .map(|row| row.iter().map(|&psd| psd.max(0.0).powf(0.1)).collect())
                    .collect();

                // Per-channel min/max normalization to u8
                let mut global_min = f64::MAX;
                let mut global_max = f64::MIN;
                for col in &compressed {
                    for &val in col {
                        if val < global_min {
                            global_min = val;
                        }
                        if val > global_max {
                            global_max = val;
                        }
                    }
                }

                let range = global_max - global_min;
                let columns: Vec<Vec<u8>> = compressed
                    .iter()
                    .map(|col| {
                        col.iter()
                            .map(|&val| {
                                if range > 0.0 {
                                    ((val - global_min) / range * 255.0).round().clamp(0.0, 255.0)
                                        as u8
                                } else {
                                    0u8
                                }
                            })
                            .collect()
                    })
                    .collect();

                let hop_duration = hop as f64 / data.sample_rate;
                let start_secs = data.actual_start.timestamp() as f64
                    + data.actual_start.timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
                let first_column_timestamp =
                    start_secs + (nfft as f64 / 2.0) / data.sample_rate;

                spectrogram_map.insert(
                    ch.clone(),
                    ChannelSpectrogram {
                        columns,
                        frequency_bins: nfft / 2 + 1,
                        hop_duration,
                        first_column_timestamp,
                    },
                );
            }
        }
    }

    // Read sensitivity
    let sensitivity: HashMap<String, f64> = {
        let sens = state.sensitivity_map.read().unwrap();
        channel_list
            .iter()
            .filter_map(|ch| sens.get(ch).map(|&v| (ch.clone(), v)))
            .collect()
    };

    // Read plot settings
    let settings = {
        let ps = state.settings.read().unwrap();
        CaptureDataPlotSettings {
            filter_waveform: ps.filter_waveform,
            filter_highpass: ps.filter_highpass,
            filter_lowpass: ps.filter_lowpass,
            filter_corners: ps.filter_corners,
            deconvolve: ps.deconvolve,
            units: ps.units.clone(),
            spectrogram_freq_min: ps.spectrogram_freq_min,
            spectrogram_freq_max: ps.spectrogram_freq_max,
            spectrogram_log_y: ps.spectrogram_log_y,
        }
    };

    let station = state.station_name.read().unwrap().clone();

    Ok(Json(CaptureDataResponse {
        station,
        sample_rate,
        channels: channels_map,
        spectrogram: spectrogram_map,
        sensitivity,
        settings,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::stream::{ChannelBuffer, WebState};
    use axum::body::Body;
    use axum::http::Request;
    use chrono::{Duration, Utc};
    use tower::ServiceExt;

    async fn response_body_json(response: axum::http::Response<Body>) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    fn populate_test_buffers(state: &WebState, channels: &[&str], sample_rate: f64, num_samples: usize) -> DateTime<Utc> {
        let now = Utc::now();
        let start = now - Duration::seconds((num_samples as f64 / sample_rate) as i64);
        let mut buffers = state.waveform_buffers.lock().unwrap();
        for &ch in channels {
            let mut buf = ChannelBuffer::new(num_samples * 2, sample_rate);
            let samples: Vec<f64> = (0..num_samples)
                .map(|i| (2.0 * std::f64::consts::PI * 10.0 * i as f64 / sample_rate).sin() * 1000.0)
                .collect();
            buf.push_segment(start, &samples, num_samples * 2);
            buffers.insert(ch.to_string(), buf);
        }
        start
    }

    #[tokio::test]
    async fn test_capture_data_valid_query() {
        let state = WebState::new();
        let start = populate_test_buffers(&state, &["EHZ", "EHN"], 100.0, 1000);
        let end = start + Duration::seconds(5);
        {
            let mut name = state.station_name.write().unwrap();
            *name = "AM.R6E01".to_string();
        }

        let app = create_router(state).await;
        let uri = format!(
            "/api/capture/data?channels=EHZ,EHN&start={}&end={}",
            start.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            end.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        );
        let response = app
            .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_json(response).await;
        assert_eq!(body["station"], "AM.R6E01");
        assert!(body["sample_rate"].as_f64().unwrap() > 0.0);
        assert!(body["channels"]["EHZ"].is_object());
        assert!(body["channels"]["EHN"].is_object());
        assert!(body["channels"]["EHZ"]["samples"].as_array().unwrap().len() > 0);
        assert!(body["settings"]["filter_waveform"].is_boolean());
        assert!(body["spectrogram"]["EHZ"].is_object());
    }

    #[tokio::test]
    async fn test_capture_data_missing_channels() {
        let state = WebState::new();
        let app = create_router(state).await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/capture/data?start=2024-01-01T00:00:00Z&end=2024-01-01T00:01:00Z")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_body_json(response).await;
        assert!(body["error"].as_str().unwrap().contains("channels"));
    }

    #[tokio::test]
    async fn test_capture_data_missing_start() {
        let state = WebState::new();
        let app = create_router(state).await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/capture/data?channels=EHZ&end=2024-01-01T00:01:00Z")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_body_json(response).await;
        assert!(body["error"].as_str().unwrap().contains("start"));
    }

    #[tokio::test]
    async fn test_capture_data_no_buffered_data_returns_404() {
        let state = WebState::new();
        let app = create_router(state).await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/capture/data?channels=EHZ&start=2024-01-01T00:00:00Z&end=2024-01-01T00:01:00Z")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_body_json(response).await;
        assert!(body["error"].as_str().unwrap().contains("no data"));
    }

    #[tokio::test]
    async fn test_capture_data_invalid_time_format() {
        let state = WebState::new();
        let app = create_router(state).await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/capture/data?channels=EHZ&start=not-a-date&end=2024-01-01T00:01:00Z")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_body_json(response).await;
        assert!(body["error"].as_str().unwrap().contains("start"));
    }
}
