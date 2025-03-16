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
    match name {
        "CLAUDE_API_KEY" => load_api_key().unwrap_or_default(),
        "PERPLEXITY_API_KEY" => load_perplexity_api_key().unwrap_or_default(),
        _ => std::env::var(name).unwrap_or_default()
    }
}

fn main() {
    // Initialize logger
    env_logger::init();
    
    // Load .env file
    dotenvy::dotenv().ok();
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, ask_claude, get_env, ask_perplexity, stream_perplexity])
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

fn load_perplexity_api_key() -> Result<String, String> {
    // First try environment variable
    if let Ok(key) = std::env::var("PERPLEXITY_API_KEY") {
        return Ok(key);
    }
    
    // Then try config file
    let config_path = get_config_path();
    if let Ok(contents) = fs::read_to_string(config_path) {
        for line in contents.lines() {
            if let Some(key) = line.strip_prefix("PERPLEXITY_API_KEY=") {
                return Ok(key.to_string());
            }
        }
    }
    
    Err("Perplexity API key not found in environment or config file".to_string())
}

#[tauri::command]
async fn ask_claude(prompt: String) -> Result<String, String> {
    info!("Starting ask_claude with prompt: {}", prompt);
    
    let client = reqwest::Client::new();
    
    // Load API key from environment or config file
    let api_key = load_api_key()?;
    info!("Successfully loaded API key");
    
    let request = ClaudeRequest {
        model: "claude-3-7-sonnet-20250219".to_string(),
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

// Perplexity API

#[derive(Deserialize)]
struct PerplexityResponse {
    choices: Vec<PerplexityChoice>,
}

#[derive(Deserialize)]
struct PerplexityChoice {
    message: PerplexityMessage,
}

#[derive(Deserialize)]
struct PerplexityMessage {
    content: String,
}

// Streaming response structures
#[derive(Deserialize, Debug)]
struct PerplexityStreamResponse {
    choices: Vec<PerplexityStreamChoice>,
}

#[derive(Deserialize, Debug)]
struct PerplexityStreamChoice {
    delta: PerplexityStreamDelta,
}

#[derive(Deserialize, Debug)]
struct PerplexityStreamDelta {
    content: Option<String>,
}

#[tauri::command]
async fn ask_perplexity(api_key: String, request_body: String) -> Result<String, String> {
    info!("Starting ask_perplexity with request body");
    
    let client = reqwest::Client::new();
    
    info!("Sending request to Perplexity API...");
    let response = match client
        .post("https://api.perplexity.ai/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(request_body)
        .send()
        .await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let status = resp.status();
                    let error_text = resp.text().await.unwrap_or_else(|_| "Could not read error response".to_string());
                    error!("API request failed with status {}: {}", status, error_text);
                    return Err(format!("API request failed: {} - {}", status, error_text));
                }
                info!("Received response from Perplexity API with status: {}", resp.status());
                resp
            },
            Err(e) => {
                error!("Failed to send request to Perplexity API: {}", e);
                return Err(e.to_string());
            }
        };

    let perplexity_response: PerplexityResponse = match response
        .json::<PerplexityResponse>()
        .await {
            Ok(resp) => {
                info!("Successfully parsed Perplexity response");
                resp
            },
            Err(e) => {
                error!("Failed to parse Perplexity API response: {}", e);
                return Err(format!("Failed to parse response: {}", e));
            }
        };

    if perplexity_response.choices.is_empty() {
        error!("Perplexity response choices array is empty");
        return Err("Empty response from Perplexity API".to_string());
    }

    info!("Returning response from Perplexity");
    Ok(perplexity_response.choices[0].message.content.clone())
}

#[tauri::command]
async fn stream_perplexity(window: tauri::Window, api_key: String, request_body: String) -> Result<(), String> {
    info!("Starting stream_perplexity with request body");
    
    let mut request_data: serde_json::Value = serde_json::from_str(&request_body)
        .map_err(|e| format!("Failed to parse request body: {}", e))?;
    
    // Add stream parameter
    request_data["stream"] = serde_json::Value::Bool(true);
    
    let client = reqwest::Client::new();
    
    info!("Sending streaming request to Perplexity API...");
    let response = match client
        .post("https://api.perplexity.ai/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()
        .await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let status = resp.status();
                    let error_text = resp.text().await.unwrap_or_else(|_| "Could not read error response".to_string());
                    error!("API request failed with status {}: {}", status, error_text);
                    return Err(format!("API request failed: {} - {}", status, error_text));
                }
                info!("Received streaming response from Perplexity API with status: {}", resp.status());
                resp
            },
            Err(e) => {
                error!("Failed to send request to Perplexity API: {}", e);
                return Err(e.to_string());
            }
        };
    
    let mut stream = response.bytes_stream();
    let mut full_response = String::new();
    
    use futures_util::StreamExt;
    
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                let chunk_str = String::from_utf8_lossy(&chunk);
                
                // Process each line in the chunk
                for line in chunk_str.lines() {
                    if line.is_empty() || line == "data: [DONE]" {
                        continue;
                    }
                    
                    if let Some(data) = line.strip_prefix("data: ") {
                        match serde_json::from_str::<PerplexityStreamResponse>(data) {
                            Ok(stream_response) => {
                                if !stream_response.choices.is_empty() {
                                    if let Some(content) = &stream_response.choices[0].delta.content {
                                        full_response.push_str(content);
                                        
                                        // Emit event to frontend with the new content
                                        if let Err(e) = window.emit("perplexity-stream", content) {
                                            error!("Failed to emit perplexity-stream event: {}", e);
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                error!("Failed to parse stream chunk: {}, data: {}", e, data);
                            }
                        }
                    }
                }
            },
            Err(e) => {
                error!("Error reading stream: {}", e);
                return Err(format!("Error reading stream: {}", e));
            }
        }
    }
    
    // Emit completion event
    if let Err(e) = window.emit("perplexity-stream-done", full_response) {
        error!("Failed to emit perplexity-stream-done event: {}", e);
    }
    
    info!("Perplexity streaming completed");
    Ok(())
}
