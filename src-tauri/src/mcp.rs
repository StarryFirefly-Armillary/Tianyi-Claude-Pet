use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use crate::state::StateChangeEvent;

pub type McpState = Arc<Mutex<crate::state::StateManager>>;

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

pub fn create_mcp_router(state: McpState, tx: broadcast::Sender<StateChangeEvent>) -> Router {
    Router::new()
        .route("/mcp", post(handle_mcp_request))
        .route("/sse", get(handle_sse))
        .with_state(McpAppState { state, tx })
}

#[derive(Clone)]
struct McpAppState {
    state: McpState,
    tx: broadcast::Sender<StateChangeEvent>,
}

async fn handle_mcp_request(
    State(app): State<McpAppState>,
    Json(req): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    let response = match req.method.as_str() {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": {
                "name": "luotianyi",
                "version": "0.1.0"
            },
            "capabilities": {
                "tools": {},
                "resources": {}
            }
        }),

        "tools/list" => json!({
            "tools": [
                {
                    "name": "tianyi_show",
                    "description": "在洛天依桌宠上显示自定义气泡消息",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "msg": { "type": "string", "description": "要显示的消息" }
                        },
                        "required": ["msg"]
                    }
                },
                {
                    "name": "tianyi_ask",
                    "description": "通过洛天依桌宠向用户提问",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "question": { "type": "string" },
                            "options": {
                                "type": "array",
                                "items": { "type": "string" }
                            }
                        },
                        "required": ["question"]
                    }
                },
                {
                    "name": "tianyi_play",
                    "description": "强制播放指定动画",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "state": { "type": "string", "enum": ["idle", "thinking", "running", "review", "failed", "waving", "jumping", "chatting", "fetching", "searching", "analyzing", "building", "celebrating"] },
                            "duration_ms": { "type": "number" }
                        },
                        "required": ["state"]
                    }
                }
            ]
        }),

        "tools/call" => {
            let params = req.params.unwrap_or_default();
            let tool_name = params["name"].as_str().unwrap_or("");
            let args = &params["arguments"];

            match tool_name {
                "tianyi_show" => {
                    let msg = args["msg"].as_str().unwrap_or("");
                    let _ = app.tx.send(StateChangeEvent {
                        animation: "waiting".into(),
                        bubble: msg.to_string(),
                    });
                    json!({ "content": [{ "type": "text", "text": format!("消息已显示: {}", msg) }] })
                }
                "tianyi_ask" => {
                    let question = args["question"].as_str().unwrap_or("");
                    let _ = app.tx.send(StateChangeEvent {
                        animation: "waving".into(),
                        bubble: question.to_string(),
                    });
                    json!({ "content": [{ "type": "text", "text": "用户已关闭" }] })
                }
                "tianyi_play" => {
                    let state_name = args["state"].as_str().unwrap_or("idle");
                    let _ = app.tx.send(StateChangeEvent {
                        animation: state_name.to_string(),
                        bubble: "".into(),
                    });
                    json!({ "content": [{ "type": "text", "text": format!("正在播放: {}", state_name) }] })
                }
                _ => {
                    return Json(JsonRpcResponse {
                        id: req.id,
                        result: None,
                        error: Some(json!({"code": -32601, "message": format!("未知工具: {}", tool_name)})),
                    });
                }
            }
        }

        "resources/list" => json!({
            "resources": [
                {
                    "uri": "tianyi://status",
                    "name": "桌宠状态",
                    "description": "当前桌宠状态和动画信息"
                },
                {
                    "uri": "tianyi://history",
                    "name": "状态历史",
                    "description": "最近的状态变更记录"
                }
            ]
        }),

        "resources/read" => {
            let params = req.params.unwrap_or_default();
            let uri = params["uri"].as_str().unwrap_or("");
            match uri {
                "tianyi://status" => {
                    let mgr = app.state.lock().await;
                    let current = mgr.current_state();
                    json!({
                        "contents": [{
                            "uri": "tianyi://status",
                            "text": format!("状态: {:?}", current)
                        }]
                    })
                }
                "tianyi://history" => {
                    let mgr = app.state.lock().await;
                    let history = mgr.history();
                    json!({
                        "contents": [{
                            "uri": "tianyi://history",
                            "text": serde_json::to_string(history).unwrap_or_default()
                        }]
                    })
                }
                _ => {
                    return Json(JsonRpcResponse {
                        id: req.id,
                        result: None,
                        error: Some(json!({"code": -32602, "message": format!("未知资源: {}", uri)})),
                    });
                }
            }
        }

        _ => {
            return Json(JsonRpcResponse {
                id: req.id,
                result: None,
                error: Some(json!({"code": -32601, "message": format!("未知方法: {}", req.method)})),
            });
        }
    };

    Json(JsonRpcResponse {
        id: req.id,
        result: Some(response),
        error: None,
    })
}

async fn handle_sse() -> StatusCode {
    StatusCode::OK
}
