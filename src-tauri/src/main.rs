// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    p2p_chat_lib::run()
}

use rand::RngCore;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
#[tauri::command]
fn generate_crypto_key() -> String {
    let mut key = [0u8; 32]; // 256-bit key
    rand::thread_rng().fill_bytes(&mut key);
    URL_SAFE.encode(&key)
}

