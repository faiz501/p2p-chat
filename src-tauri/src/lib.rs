// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod iroh;
mod messages;

use anyhow::Result;
use futures_lite::StreamExt;
use iroh_docs::{rpc::client::docs::LiveEvent, ContentStatus};
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::Mutex;

use self::{
    iroh::Iroh,
    messages::{Msg, Msgs},
};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

async fn setup<R: tauri::Runtime>(handle: tauri::AppHandle<R>) -> Result<()> {
    // Get the application data directory and append "iroh_data"
    let data_root = handle.path().app_data_dir().unwrap().join("iroh_data");

    // Initialize the Iroh node using the data root
    let iroh = Iroh::new(data_root).await?;
    handle.manage(AppState::new(iroh));

    Ok(())
}

struct AppState {
    msgs: Mutex<Option<(Msgs, tokio::task::JoinHandle<()>)>>,
    iroh: Iroh,
}

impl AppState {
    fn new(iroh: Iroh) -> Self {
        AppState {
            msgs: Mutex::new(None),
            iroh,
        }
    }

    fn iroh(&self) -> &Iroh {
        &self.iroh
    }

    async fn init_todos<R: tauri::Runtime>(
        &self,
        app_handle: tauri::AppHandle<R>,
        msgs: Msgs,
    ) -> Result<()> {
        let mut events = msgs.doc_subscribe().await?;
        let events_handle = tokio::spawn(async move {
            while let Some(Ok(event)) = events.next().await {
                match event {
                    LiveEvent::InsertRemote { content_status, .. } => {
                        // Only update if we already have the content.
                        if content_status == ContentStatus::Complete {
                            app_handle.emit("update-all", ()).ok();
                        }
                    }
                    LiveEvent::InsertLocal { .. } | LiveEvent::ContentReady { .. } => {
                        app_handle.emit("update-all", ()).ok();
                    }
                    _ => {}
                }
            }
        });

        let mut t = self.msgs.lock().await;
        if let Some((_msgs, handle)) = t.take() {
            handle.abort();
        }
        *t = Some((msgs, events_handle));

        Ok(())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Clone the app handle so that it becomes 'static for the async task.
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                println!("starting backend...");
                if let Err(err) = setup(app_handle).await {
                    eprintln!("failed: {:?}", err);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            create_room,
            send_msg,
            set_ticket,
            get_ticket,
            get_msgs,
            new_room,
            new_msg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn get_msgs(state: tauri::State<'_, AppState>) -> Result<Vec<Msg>, String> {
    if let Some((msgs, _)) = &mut *state.msgs.lock().await {
        let msgs = msgs.get_msgs().await.map_err(|e| e.to_string())?;
        return Ok(msgs);
    }
    Err("not initialized".to_string())
}

#[tauri::command]
async fn new_room(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let todos = Msgs::new(None, state.iroh().clone())
        .await
        .map_err(|e| e.to_string())?;
    println!("Created new todos: {:#?}", todos);
    state
        .init_todos(app_handle, todos)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn new_msg(msg: Msg, state: tauri::State<'_, AppState>) -> Result<(), String> {
    if let Some((msgs, _)) = &mut *state.msgs.lock().await {
        msgs.add(msg.id, msg.label)
            .await
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    Err("not initialized".to_string())
}

#[tauri::command]
async fn set_ticket(
    app_handle: tauri::AppHandle,
    ticket: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let todos = Msgs::new(Some(ticket), state.iroh().clone())
        .await
        .map_err(|e| e.to_string())?;

    state
        .init_todos(app_handle, todos)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_ticket(state: tauri::State<'_, AppState>) -> Result<String, String> {
    if let Some((todos, _)) = &mut *state.msgs.lock().await {
        return Ok(todos.ticket());
    }
    Err("not initialized".to_string())
}

#[tauri::command]
fn create_room(room: &str) -> String {
    format!("Creating a room.. {}", room)
}

#[tauri::command]
fn send_msg(s_msg: &str) -> String {
    format!("Sending msg: {}", s_msg)
}
