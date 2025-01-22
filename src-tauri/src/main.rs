// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::{info, error};
use std::path::PathBuf;
use std::fs;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
fn get_env(name: &str) -> String {
    std::env::var(name).unwrap_or_default()
}

fn main() {
    // Initialize logger
    env_logger::init();
    
    // Load .env file
    dotenvy::dotenv().ok();
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, ask_claude, get_env])
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
    temperature: f32,
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

fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".olly").join("config.env")
}

fn load_api_key() -> Result<String, String> {
    // First try environment variable
    if let Ok(key) = std::env::var("CLAUDE_API_KEY") {
        return Ok(key);
    }
    
    // Then try config file
    let config_path = get_config_path();
    if let Ok(contents) = fs::read_to_string(config_path) {
        for line in contents.lines() {
            if let Some(key) = line.strip_prefix("CLAUDE_API_KEY=") {
                return Ok(key.to_string());
            }
        }
    }
    
    Err("API key not found in environment or config file".to_string())
}

#[tauri::command]
async fn ask_claude(prompt: String) -> Result<String, String> {
    info!("Starting ask_claude with prompt: {}", prompt);
    
    let client = reqwest::Client::new();
    
    // Load API key from environment or config file
    let api_key = load_api_key()?;
    info!("Successfully loaded API key");
    
    let request = ClaudeRequest {
        model: "claude-3-5-sonnet-20241022".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
        max_tokens: 1024,
        temperature: 0.0,
    };

    info!("Sending request to Claude API...");
    let response = match client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let status = resp.status();
                    let error_text = resp.text().await.unwrap_or_else(|_| "Could not read error response".to_string());
                    error!("API request failed with status {}: {}", status, error_text);
                    return Err(format!("API request failed: {} - {}", status, error_text));
                }
                info!("Received response from Claude API with status: {}", resp.status());
                resp
            },
            Err(e) => {
                error!("Failed to send request to Claude API: {}", e);
                return Err(e.to_string());
            }
        };

    let claude_response: ClaudeResponse = match response
        .json::<ClaudeResponse>()
        .await {
            Ok(resp) => {
                info!("Successfully parsed response: {:?}", resp.content.first().map(|c| &c.text));
                resp
            },
            Err(e) => {
                error!("Failed to parse Claude API response: {}", e);
                return Err(format!("Failed to parse response: {}", e));
            }
        };

    if claude_response.content.is_empty() {
        error!("Claude response content array is empty");
        return Err("Empty response from Claude API".to_string());
    }

    info!("Returning response from Claude");
    Ok(claude_response.content[0].text.clone())
}
