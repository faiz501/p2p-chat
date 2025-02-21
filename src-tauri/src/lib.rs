// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod iroh;
mod messages;

use anyhow::Result;
use messages::Chat;
use std::sync::Arc;
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::Mutex;

/// Application state holds the chat instance.
struct AppState {
    chat: Mutex<Option<Arc<Chat>>>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            chat: Mutex::new(None),
        }
    }
}

/// Start a chat session.
/// If a nodeid is provided, join that room; if not, create a new chat room.
/// The command returns the current node id (which is our local node id when creating a new room,
/// or the remote node id if joining an existing room).
#[tauri::command]
async fn start_chat(
    app_handle: tauri::AppHandle,
    nodeid: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    // Create the chat instance (using the nodeid if provided)
    let chat = Chat::new(nodeid).await.map_err(|e| e.to_string())?;
    // Extract the node id as a string
    let current_node_id = chat
        .nodeid
        .as_ref()
        .expect("nodeid should be set")
        .to_string();
    let chat = Arc::new(chat);
    {
        let mut chat_lock = state.chat.lock().await;
        *chat_lock = Some(chat.clone());
    }

    // Spawn an async task that continuously listens for incoming messages.
    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        loop {
            match chat.receive().await {
                Ok(msg) => {
                    // Emit the new message to the frontend (use event "new_message")
                    if let Err(e) = app_handle_clone.emit("new_message", msg) {
                        eprintln!("Error emitting new_message event: {:?}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving message: {:?}", e);
                    // Sleep briefly before retrying.
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    });

    Ok(current_node_id)
}

#[tauri::command]
async fn send_msg(msg: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let chat_lock = state.chat.lock().await;
    if let Some(chat) = &*chat_lock {
        chat.send(&msg).await.map_err(|e| e.to_string())
    } else {
        Err("Chat not started".to_string())
    }
}

/// (Optional) A command to receive a single message.
/// With the background listener in place, this might not be needed.
#[tauri::command]
async fn receive_msg(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let chat_lock = state.chat.lock().await;
    if let Some(chat) = &*chat_lock {
        chat.receive().await.map_err(|e| e.to_string())
    } else {
        Err("Chat not started".to_string())
    }
}

/// The greet command remains unchanged.
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            start_chat,
            send_msg,
            receive_msg,
            greet
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
