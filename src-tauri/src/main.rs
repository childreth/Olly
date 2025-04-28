// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

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
use futures_util::stream::StreamExt;

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
    stream: Option<bool>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct ClaudeResponse {
    content: Vec<Content>,
}

#[derive(Deserialize, Debug)]
struct Content {
    text: String,
}

// Streaming response structures for Claude
#[derive(Deserialize, Debug)]
struct ClaudeStreamDelta {
    text: String,
}

#[derive(Deserialize, Debug)]
struct ClaudeStreamResponse {
    #[serde(rename = "response_type")]
    response_type: Option<String>,
    delta: ClaudeStreamDelta,
    content: Vec<Content>,
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
        model: "claude-3-7-sonnet-20250219".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
        max_tokens: 1024,
        temperature: 0.0,
        stream: None,
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

#[tauri::command]
async fn stream_claude(window: tauri::Window, prompt: String) -> Result<(), String> {
    info!("Starting stream_claude with prompt: {}", prompt);
    
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
        stream: Some(true),
    };

    info!("Sending streaming request to Claude API...");
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
                info!("Received streaming response from Claude API with status: {}", resp.status());
                resp
            },
            Err(e) => {
                error!("Failed to send request to Claude API: {}", e);
                return Err(e.to_string());
            }
        };
    
    let mut stream = response.bytes_stream();
    let mut full_response = String::new();
    
    while let Some(item) = stream.next().await {
        match item {
            Ok(bytes) => {
                let chunk = String::from_utf8_lossy(&bytes);
                info!("Received chunk from Claude: {}", chunk);
                
                for line in chunk.lines() {
                    if line.is_empty() || line == "data: [DONE]" {
                        continue;
                    }
                    
                    // Remove the "data: " prefix if present
                    let json_str = if line.starts_with("data: ") {
                        &line[6..]
                    } else {
                        line
                    };
                    
                    // Skip if line is an event message or empty/invalid JSON
                    if line.starts_with("event:") {
                        info!("Received Claude event: {}", line);
                        continue;
                    }
                    
                    // Skip if the JSON string is empty or obviously invalid
                    if json_str.trim().is_empty() || !json_str.trim().starts_with('{') {
                        error!("Skipping invalid JSON from Claude: {}", json_str);
                        continue;
                    }
                    
                    // Parse the JSON
                    match serde_json::from_str::<ClaudeStreamResponse>(json_str) {
                        Ok(parsed) => {
                            if let Some(response_type) = &parsed.response_type {
                                if response_type == "content_block_delta" {
                                    if !parsed.delta.text.is_empty() {
                                        let content = &parsed.delta.text;
                                        info!("Parsed content from Claude delta: {}", content);
                                        full_response.push_str(content);
                                        
                                        // Emit event to frontend
                                        if let Err(e) = window.emit("claude-stream", content) {
                                            error!("Failed to emit claude-stream event: {}", e);
                                        }
                                    }
                                } else if response_type == "content_block_start" {
                                    info!("Claude content block started");
                                }
                            } else if !parsed.content.is_empty() && !parsed.content[0].text.is_empty() {
                                let content = &parsed.content[0].text;
                                info!("Parsed content from Claude content array: {}", content);
                                
                                // Emit event to frontend
                                if let Err(e) = window.emit("claude-stream", content) {
                                    error!("Failed to emit claude-stream event: {}", e);
                                }
                            }
                        },
                        Err(e) => {
                            error!("Failed to parse JSON from Claude chunk: {} - Error: {}", json_str, e);
                            // Try to salvage any content by looking for text pattern
                            if let Some(content_start) = json_str.find("\"text\": \"") {
                                if let Some(content_end) = json_str[content_start + 9..].find("\"") {
                                    let content = &json_str[content_start + 9..content_start + 9 + content_end];
                                    info!("Salvaged content from Claude: {}", content);
                                    full_response.push_str(content);
                                    
                                    // Emit event to frontend with salvaged content
                                    if let Err(e) = window.emit("claude-stream", content) {
                                        error!("Failed to emit claude-stream event with salvaged content: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Err(e) => {
                error!("Error reading from Claude stream: {}", e);
                return Err(format!("Error reading from stream: {}", e));
            }
        }
    }
    
    info!("Streaming completed from Claude. Full response: {}", full_response);
    
    // Emit completion event with the full response
    if let Err(e) = window.emit("claude-stream-done", full_response) {
        error!("Failed to emit claude-stream-done event: {}", e);
    }
    
    Ok(())
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
    info!("Starting stream_perplexity with request body: {}", request_body);
    
    let client = reqwest::Client::new();
    
    // Add stream: true to the request body
    let mut request_json: serde_json::Value = serde_json::from_str(&request_body)
        .map_err(|e| format!("Failed to parse request body: {}", e))?;
    
    request_json["stream"] = serde_json::Value::Bool(true);
    
    let stream_request_body = serde_json::to_string(&request_json)
        .map_err(|e| format!("Failed to serialize request body: {}", e))?;
    
    info!("Sending streaming request to Perplexity API with body: {}", stream_request_body);
    
    let response = match client
        .post("https://api.perplexity.ai/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(stream_request_body)
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
    
    while let Some(item) = stream.next().await {
        match item {
            Ok(bytes) => {
                let chunk = String::from_utf8_lossy(&bytes);
                info!("Received chunk: {}", chunk);
                
                // Split by lines (each line is a separate JSON object)
                for line in chunk.lines() {
                    if line.is_empty() || line == "data: [DONE]" {
                        continue;
                    }
                    
                    // Remove the "data: " prefix if present
                    let json_str = if line.starts_with("data: ") {
                        &line[6..]
                    } else {
                        line
                    };
                    
                    // Skip if the JSON string is empty or obviously invalid
                    if json_str.trim().is_empty() || !json_str.trim().starts_with('{') {
                        error!("Skipping invalid JSON: {}", json_str);
                        continue;
                    }
                    
                    // Parse the JSON
                    match serde_json::from_str::<PerplexityStreamResponse>(json_str) {
                        Ok(parsed) => {
                            // Extract content from the first choice's delta if available
                            if let Some(choice) = parsed.choices.first() {
                                if let Some(content) = &choice.delta.content {
                                    info!("Parsed content: {}", content);
                                    full_response.push_str(content);
                                    
                                    // Emit event to frontend
                                    if let Err(e) = window.emit("perplexity-stream", content) {
                                        error!("Failed to emit perplexity-stream event: {}", e);
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            error!("Failed to parse JSON from chunk: {} - Error: {}", json_str, e);
                            // Try to salvage any content by looking for delta.content pattern
                            if let Some(content_start) = json_str.find("\"content\": \"") {
                                if let Some(content_end) = json_str[content_start + 12..].find("\"") {
                                    let content = &json_str[content_start + 12..content_start + 12 + content_end];
                                    info!("Salvaged content: {}", content);
                                    full_response.push_str(content);
                                    
                                    // Emit event to frontend with salvaged content
                                    if let Err(e) = window.emit("perplexity-stream", content) {
                                        error!("Failed to emit perplexity-stream event with salvaged content: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Err(e) => {
                error!("Error reading from stream: {}", e);
                return Err(format!("Error reading from stream: {}", e));
            }
        }
    }
    
    info!("Streaming completed. Full response: {}", full_response);
    
    // Emit completion event with the full response
    if let Err(e) = window.emit("perplexity-stream-done", full_response) {
        error!("Failed to emit perplexity-stream-done event: {}", e);
    }
    
    Ok(())
}
