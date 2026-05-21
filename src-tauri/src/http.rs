use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

use crate::state::{PetState, SharedState, SharedScale, StateChangeEvent};

#[derive(Debug, Deserialize)]
pub struct StateRequest {
    pub s: String,
    #[serde(default)]
    pub tool: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScaleRequest {
    pub scale: f64,
}

#[derive(Debug, Serialize)]
pub struct CurrentResponse {
    pub animation: String,
    pub bubble: String,
    pub scale: f64,
}

pub fn create_router(state: SharedState, scale: SharedScale, tx: broadcast::Sender<StateChangeEvent>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/state", post(handle_state))
        .route("/api/heartbeat", get(handle_heartbeat))
        .route("/api/current", get(handle_current))
        .route("/api/hook/thinking", post(handle_hook_thinking))
        .route("/api/hook/working", post(handle_hook_working))
        .route("/api/hook/done", post(handle_hook_done))
        .route("/api/hook/idle", post(handle_hook_idle))
        .route("/api/hook/permission", post(handle_hook_permission))
        .route("/api/hook/scale", post(handle_hook_scale))
        .layer(cors)
        .with_state(AppState { state, scale, tx })
}

#[derive(Clone)]
struct AppState {
    state: SharedState,
    scale: SharedScale,
    tx: broadcast::Sender<StateChangeEvent>,
}

async fn handle_state(
    State(app): State<AppState>,
    Json(body): Json<StateRequest>,
) -> StatusCode {
    let pet_state = PetState::from_hook(&body.s, body.tool.as_deref());
    let tool = body.tool.clone();
    let animation = pet_state.animation_name().to_string();
    let bubble = pet_state.bubble_text(tool.as_deref()).to_string();

    {
        let mut mgr = app.state.lock().await;
        mgr.set_state(pet_state, tool);
    }

    let _ = app.tx.send(StateChangeEvent { animation, bubble });
    StatusCode::OK
}

async fn handle_heartbeat() -> StatusCode {
    StatusCode::OK
}

async fn handle_current(
    State(app): State<AppState>,
) -> Json<CurrentResponse> {
    let mgr = app.state.lock().await;
    let current = mgr.current_state();
    let tool = mgr.current_tool();
    let animation = current.animation_name().to_string();
    let bubble = current.bubble_text(tool).to_string();
    let scale = *app.scale.lock().unwrap();
    Json(CurrentResponse { animation, bubble, scale })
}

async fn handle_hook_thinking(
    State(app): State<AppState>,
) -> StatusCode {
    set_pet_state(&app, PetState::Chatting, None).await;
    StatusCode::OK
}

async fn handle_hook_working(
    State(app): State<AppState>,
    body: String,
) -> StatusCode {
    let tool = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| v.get("tool_name").and_then(|t| t.as_str().map(String::from)));

    let state = match tool.as_deref() {
        Some("WebFetch") => PetState::Fetching,
        Some("WebSearch") => PetState::Searching,
        Some("Write") | Some("Edit") => PetState::Building,
        Some("Agent") | Some("TaskCreate") | Some("TaskUpdate") => PetState::Analyzing,
        _ => PetState::Running,
    };

    set_pet_state(&app, state, tool).await;
    StatusCode::OK
}

async fn handle_hook_done(
    State(app): State<AppState>,
) -> StatusCode {
    set_pet_state(&app, PetState::Celebrating, None).await;
    StatusCode::OK
}

async fn handle_hook_idle(
    State(app): State<AppState>,
) -> StatusCode {
    set_pet_state(&app, PetState::Idle, None).await;
    StatusCode::OK
}

async fn handle_hook_permission(
    State(app): State<AppState>,
) -> StatusCode {
    {
        let mut mgr = app.state.lock().await;
        mgr.set_state(PetState::Permission, None);
    }
    let _ = app.tx.send(StateChangeEvent {
        animation: "waving".to_string(),
        bubble: "等待指示...".to_string(),
    });
    StatusCode::OK
}

async fn handle_hook_scale(
    State(app): State<AppState>,
    Json(body): Json<ScaleRequest>,
) -> StatusCode {
    *app.scale.lock().unwrap() = body.scale;
    StatusCode::OK
}

async fn set_pet_state(app: &AppState, state: PetState, tool: Option<String>) {
    {
        let mgr = app.state.lock().await;
        let is_active = matches!(mgr.current_state(),
            PetState::Running | PetState::Chatting | PetState::Fetching
            | PetState::Searching | PetState::Analyzing | PetState::Building
        );
        if !matches!(state, PetState::Running | PetState::Chatting | PetState::Fetching
            | PetState::Searching | PetState::Analyzing | PetState::Building)
            && is_active
            && mgr.should_keep_running(800)
        {
            return;
        }
    }
    let animation = state.animation_name().to_string();
    let bubble = state.bubble_text(tool.as_deref()).to_string();
    {
        let mut mgr = app.state.lock().await;
        mgr.set_state(state, tool);
    }
    let _ = app.tx.send(StateChangeEvent { animation, bubble });
}
