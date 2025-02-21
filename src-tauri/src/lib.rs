#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// Assume that the dumbpipe module exists (or is available as a dependency)
mod dumbpipe;

use anyhow::Result;
use std::str::FromStr;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

// Import dumbpipe and iroh types
use dumbpipe::{NodeTicket, ALPN, HANDSHAKE};
use iroh::{endpoint::Endpoint, SecretKey};
// Use the OsRng type from the rand crate so that the trait bounds match.
use rand::rngs::OsRng;

/// Global state for the active chat session.
/// (We allow only one session at a time in this simple prototype.)
#[derive(Default)]
struct ChatState(pub Arc<Mutex<Option<chat::ChatSession>>>);

/// Tauri command to create a chat room (i.e. listen for an incoming connection)
#[tauri::command]
async fn create_chat_room(
    app_handle: tauri::AppHandle,
    chat_state: tauri::State<'_, ChatState>,
) -> Result<String, String> {
    match chat::create_chat_room(app_handle, chat_state).await {
        Ok(ticket) => Ok(ticket.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// Tauri command to join an existing chat room given a NodeTicket string.
#[tauri::command]
async fn join_chat_room(
    app_handle: tauri::AppHandle,
    ticket: String,
    chat_state: tauri::State<'_, ChatState>,
) -> Result<(), String> {
    chat::join_chat_room(app_handle, ticket, chat_state)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri command to send a message on the active chat session.
#[tauri::command]
async fn send_message(
    message: String,
    chat_state: tauri::State<'_, ChatState>,
) -> Result<(), String> {
    chat::send_message(chat_state, message)
        .await
        .map_err(|e| e.to_string())
}

/// The chat module implements the core connection logic using dumbpipe and iroh.
mod chat {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// A chat session holds the send half of the bidirectional stream.
    pub struct ChatSession {
        pub sender: Arc<Mutex<iroh::endpoint::SendStream>>,
    }

    /// Helper to get or create a secret key.
    async fn get_or_create_secret() -> Result<SecretKey> {
        if let Ok(secret) = std::env::var("IROH_SECRET") {
            SecretKey::from_str(&secret).map_err(|e| e.into())
        } else {
            let key = SecretKey::generate(OsRng);
            eprintln!("using secret key {}", key);
            Ok(key)
        }
    }

    /// Create a chat room: bind a dumbpipe endpoint and wait for an incoming connection.
    /// When a peer connects and sends the proper handshake, the sender half is saved
    /// in the global chat state and incoming messages are emitted as Tauri events.
    pub async fn create_chat_room(
        app_handle: tauri::AppHandle,
        chat_state: tauri::State<'_, super::ChatState>,
    ) -> Result<NodeTicket> {
        let secret_key = get_or_create_secret().await?;
        let builder = Endpoint::builder()
            .alpns(vec![ALPN.to_vec()])
            .secret_key(secret_key);
        // Bind to a random available port.
        let endpoint = builder.bind().await?;
        // Wait until the endpoint’s relay is ready.
        endpoint.home_relay().initialized().await?;
        let node_addr = endpoint.node_addr().await?;
        let ticket = NodeTicket::new(node_addr.clone());

        // Spawn a background task to accept a connection.
        let app_handle_clone = app_handle.clone();
        let chat_state_clone = chat_state.0.clone();
        tokio::spawn(async move {
            if let Some(connecting) = endpoint.accept().await {
                if let Ok(connection) = connecting.await {
                    if let Ok((mut send, mut recv)) = connection.accept_bi().await {
                        // Read and verify the fixed handshake.
                        let mut buf = [0u8; HANDSHAKE.len()];
                        if let Ok(()) = recv.read_exact(&mut buf).await {
                            if buf == HANDSHAKE {
                                // Spawn a task to continuously read messages and emit them to the frontend.
                                let app_handle_inner = app_handle_clone.clone();
                                tokio::spawn(async move {
                                    let mut buffer = vec![0u8; 1024];
                                    loop {
                                        match recv.read(&mut buffer).await {
                                            Ok(Some(n)) if n == 0 => break, // connection closed
                                            Ok(Some(n)) => {
                                                let message = String::from_utf8_lossy(&buffer[..n])
                                                    .to_string();
                                                let _ =
                                                    app_handle_inner.emit("new-message", message);
                                            }
                                            Ok(None) => break,
                                            Err(_) => break,
                                        }
                                    }
                                });
                                // Save the send stream so that outgoing messages can be sent.
                                let session = ChatSession {
                                    sender: Arc::new(Mutex::new(send)),
                                };
                                let mut state_lock = chat_state_clone.lock().await;
                                *state_lock = Some(session);
                            }
                        }
                    }
                }
            }
        });

        Ok(ticket)
    }

    /// Join an existing chat room using the provided NodeTicket string.
    /// The function connects to the remote endpoint, sends the handshake,
    /// and spawns a task to forward incoming messages as Tauri events.
    pub async fn join_chat_room(
        app_handle: tauri::AppHandle,
        ticket_str: String,
        chat_state: tauri::State<'_, super::ChatState>,
    ) -> Result<()> {
        let ticket = NodeTicket::from_str(&ticket_str)?;
        let secret_key = get_or_create_secret().await?;
        let builder = Endpoint::builder()
            .secret_key(secret_key)
            .alpns(vec![ALPN.to_vec()]);
        let endpoint = builder.bind().await?;
        let addr = ticket.node_addr();
        let connection = endpoint.connect(addr.clone(), &ALPN.to_vec()).await?;
        let (mut send, mut recv) = connection.open_bi().await?;
        // Send the handshake.
        send.write_all(&HANDSHAKE).await?;
        // Spawn a task to listen for incoming messages.
        let app_handle_clone = app_handle.clone();
        tokio::spawn(async move {
            let mut buffer = vec![0u8; 1024];
            loop {
                match recv.read(&mut buffer).await {
                    Ok(Some(n)) if n == 0 => break,
                    Ok(Some(n)) => {
                        let message = String::from_utf8_lossy(&buffer[..n]).to_string();
                        let _ = app_handle_clone.emit("new-message", message);
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
        });
        let session = ChatSession {
            sender: Arc::new(Mutex::new(send)),
        };
        let mut state_lock = chat_state.0.lock().await;
        *state_lock = Some(session);
        Ok(())
    }

    /// Send a message over the active chat session.
    pub async fn send_message(
        chat_state: tauri::State<'_, super::ChatState>,
        message: String,
    ) -> Result<()> {
        let state_lock = chat_state.0.lock().await;
        if let Some(ref session) = *state_lock {
            let mut sender = session.sender.lock().await;
            sender.write_all(message.as_bytes()).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No active chat session"))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ChatState::default())
        .invoke_handler(tauri::generate_handler![
            create_chat_room,
            join_chat_room,
            send_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
