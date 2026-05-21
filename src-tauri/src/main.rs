#![windows_subsystem = "windows"]

mod http;
mod mcp;
mod state;
mod tray;

use state::StateManager;
use state::StateChangeEvent;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::{broadcast, Mutex};
use tauri::Emitter;

#[tokio::main]
async fn main() {
    let state_manager = Arc::new(Mutex::new(StateManager::new()));
    let (tx, _rx) = broadcast::channel::<StateChangeEvent>(32);

    let sm_http = state_manager.clone();
    let tx_http = tx.clone();

    let scale = Arc::new(StdMutex::new(0.333));
    let scale_http = scale.clone();

    // Spawn HTTP server on :9527
    tokio::spawn(async move {
        let app = http::create_router(sm_http, scale_http, tx_http);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:9527").await.unwrap();
        println!("HTTP server listening on http://127.0.0.1:9527");
        axum::serve(listener, app).await.unwrap();
    });

    let sm_mcp = state_manager.clone();
    let tx_mcp = tx.clone();

    // Spawn MCP server on :9528
    tokio::spawn(async move {
        let app = mcp::create_mcp_router(sm_mcp, tx_mcp);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:9528").await.unwrap();
        println!("MCP server listening on http://127.0.0.1:9528");
        axum::serve(listener, app).await.unwrap();
    });

    // Build Tauri app
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_drag])
        .setup(move |app| {
            let handle = app.handle().clone();
            let mut rx = tx.subscribe();
            let handle2 = handle.clone();
            tokio::spawn(async move {
                while let Ok(event) = rx.recv().await {
                    let _ = handle2.emit("state-change", event);
                }
            });

            // Send initial waving state
            let _ = handle.emit(
                "state-change",
                StateChangeEvent {
                    animation: "waving".to_string(),
                    bubble: "天依来啦~".to_string(),
                },
            );

            // Enable system tray
            if let Err(e) = tray::setup(app, scale.clone()) {
                eprintln!("Failed to setup tray: {}", e);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running 洛天依 Pet");
}

#[tauri::command]
fn start_drag(window: tauri::Window) {
    let _ = window.start_dragging();
}
