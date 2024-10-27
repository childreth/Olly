// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]



// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, ask_claude])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Claude API

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<Content>,
}

#[derive(Deserialize)]
struct Content {
    text: String,
}

#[tauri::command]
async fn ask_claude(prompt: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let request = ClaudeRequest {
        model: "claude-3-sonnet-20240229".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
        max_tokens: 1024,
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", "api_key_here")
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let claude_response: ClaudeResponse = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(claude_response.content[0].text.clone())
}

