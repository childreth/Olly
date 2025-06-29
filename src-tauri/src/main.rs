// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

use log::{info, error};
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tauri::Manager;

// API Key Management Module
mod api_keys {
    use super::*;
    use serde::{Deserialize, Serialize};
    use keyring::Entry;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct ApiKeyEntry {
        pub provider: String,
        pub display_name: String,
        pub created_at: DateTime<Utc>,
        pub last_used: Option<DateTime<Utc>>,
        pub is_active: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ApiKeyStore {
        pub providers: HashMap<String, ApiKeyEntry>,
    }

    const KEYRING_SERVICE: &str = "com.olly.app";
    const KEYRING_USER: &str = "api_keys";
    const STORE_KEY: &str = "api_key_store";

    impl ApiKeyStore {
        pub fn new() -> Self {
            Self {
                providers: HashMap::new(),
            }
        }

        pub fn load(_app: &tauri::AppHandle) -> Result<Self, String> {
            match Entry::new(KEYRING_SERVICE, STORE_KEY) {
                Ok(entry) => {
                    match entry.get_password() {
                        Ok(data) => {
                            serde_json::from_str(&data)
                                .map_err(|e| format!("Failed to parse stored API key data: {}", e))
                        }
                        Err(_) => {
                            info!("No existing API key store found, creating new");
                            Ok(Self::new())
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to create keyring entry: {}", e);
                    Ok(Self::new())
                }
            }
        }

        pub fn save(&self, _app: &tauri::AppHandle) -> Result<(), String> {
            let data = serde_json::to_string(self)
                .map_err(|e| format!("Failed to serialize API key store: {}", e))?;
            
            let entry = Entry::new(KEYRING_SERVICE, STORE_KEY)
                .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
            
            entry.set_password(&data)
                .map_err(|e| format!("Failed to save API key store to keyring: {}", e))
        }

        pub fn store_key(&mut self, provider: &str, display_name: &str, api_key: &str, app: &tauri::AppHandle) -> Result<(), String> {
            let key_id = format!("{}_key", provider);
            
            // Store the actual API key in keyring
            let entry = Entry::new(KEYRING_SERVICE, &key_id)
                .map_err(|e| format!("Failed to create keyring entry for {}: {}", provider, e))?;
            
            entry.set_password(api_key)
                .map_err(|e| format!("Failed to store API key for {}: {}", provider, e))?;

            // Store metadata in the store
            let entry = ApiKeyEntry {
                provider: provider.to_string(),
                display_name: display_name.to_string(),
                created_at: Utc::now(),
                last_used: None,
                is_active: true,
            };

            self.providers.insert(provider.to_string(), entry);
            self.save(app)?;

            info!("Successfully stored API key for provider: {}", provider);
            Ok(())
        }

        pub fn get_key(&mut self, provider: &str, app: &tauri::AppHandle) -> Result<Option<String>, String> {
            if !self.providers.contains_key(provider) {
                return Ok(None);
            }

            let key_id = format!("{}_key", provider);
            let entry = Entry::new(KEYRING_SERVICE, &key_id)
                .map_err(|e| format!("Failed to create keyring entry for {}: {}", provider, e))?;
            
            match entry.get_password() {
                Ok(api_key) => {
                    // Update last_used timestamp
                    if let Some(entry) = self.providers.get_mut(provider) {
                        entry.last_used = Some(Utc::now());
                        let _ = self.save(app); // Continue even if save fails
                    }
                    Ok(Some(api_key))
                }
                Err(_) => Ok(None),
            }
        }

        pub fn delete_key(&mut self, provider: &str, app: &tauri::AppHandle) -> Result<(), String> {
            let key_id = format!("{}_key", provider);
            
            // Delete the actual API key from keyring
            if let Ok(entry) = Entry::new(KEYRING_SERVICE, &key_id) {
                if let Err(e) = entry.delete_credential() {
                    error!("Failed to delete API key from keyring for {}: {}", provider, e);
                }
            }

            // Remove metadata from store
            self.providers.remove(provider);
            self.save(app)?;

            info!("Successfully deleted API key for provider: {}", provider);
            Ok(())
        }

        pub fn list_providers(&self) -> Vec<String> {
            self.providers.keys().cloned().collect()
        }

        pub fn get_provider_info(&self, provider: &str) -> Option<&ApiKeyEntry> {
            self.providers.get(provider)
        }

        pub fn migrate_from_config_file(&mut self, app: &tauri::AppHandle) -> Result<(), String> {
            info!("Starting API key migration from config file");
            
            let config_path = get_config_path();
            if let Ok(contents) = fs::read_to_string(&config_path) {
                for line in contents.lines() {
                    if let Some(key) = line.strip_prefix("CLAUDE_API_KEY=") {
                        if !self.providers.contains_key("claude") {
                            info!("Migrating Claude API key from config file to secure storage");
                            self.store_key("claude", "Claude API", key.trim(), app)?;
                        }
                    }
                    // Add more providers as needed
                    if let Some(key) = line.strip_prefix("PERPLEXITY_API_KEY=") {
                        if !self.providers.contains_key("perplexity") {
                            info!("Migrating Perplexity API key from config file to secure storage");
                            self.store_key("perplexity", "Perplexity API", key.trim(), app)?;
                        }
                    }
                }
                info!("Migration completed successfully");
            } else {
                info!("No config file found for migration");
            }
            
            Ok(())
        }
    }
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
fn get_env(name: &str) -> String {
    std::env::var(name).unwrap_or_default()
}

// API Key Management Commands
#[tauri::command]
async fn store_api_key(app: tauri::AppHandle, provider: String, display_name: String, api_key: String) -> Result<(), String> {
    info!("Storing API key for provider: {}", provider);
    
    let mut store = api_keys::ApiKeyStore::load(&app)?;
    store.store_key(&provider, &display_name, &api_key, &app)?;
    
    Ok(())
}

#[tauri::command]
async fn get_api_key(app: tauri::AppHandle, provider: String) -> Result<Option<String>, String> {
    info!("Retrieving API key for provider: {}", provider);
    
    let mut store = api_keys::ApiKeyStore::load(&app)?;
    store.get_key(&provider, &app)
}

#[tauri::command]
async fn list_api_key_providers(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    info!("Listing API key providers");
    
    let store = api_keys::ApiKeyStore::load(&app)?;
    Ok(store.list_providers())
}

#[tauri::command]
async fn delete_api_key(app: tauri::AppHandle, provider: String) -> Result<(), String> {
    info!("Deleting API key for provider: {}", provider);
    
    let mut store = api_keys::ApiKeyStore::load(&app)?;
    store.delete_key(&provider, &app)?;
    
    Ok(())
}

#[tauri::command]
async fn get_provider_info(app: tauri::AppHandle, provider: String) -> Result<Option<api_keys::ApiKeyEntry>, String> {
    info!("Getting provider info for: {}", provider);
    
    let store = api_keys::ApiKeyStore::load(&app)?;
    Ok(store.get_provider_info(&provider).cloned())
}

#[tauri::command]
async fn migrate_api_keys(app: tauri::AppHandle) -> Result<(), String> {
    info!("Starting API key migration");
    
    let mut store = api_keys::ApiKeyStore::load(&app)?;
    store.migrate_from_config_file(&app)?;
    
    Ok(())
}

#[tauri::command]
async fn validate_api_key(provider: String, api_key: String) -> Result<bool, String> {
    info!("Validating API key for provider: {}", provider);
    
    let client = reqwest::Client::new();
    
    match provider.as_str() {
        "claude" => {
            // Test Claude API with a simple request
            let response = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&serde_json::json!({
                    "model": "claude-3-7-sonnet-20250219",
                    "max_tokens": 1,
                    "messages": [{"role": "user", "content": "hi"}]
                }))
                .send()
                .await;
            
            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        info!("Claude API key validation successful");
                        Ok(true)
                    } else if resp.status() == 401 {
                        info!("Claude API key validation failed - unauthorized");
                        Ok(false)
                    } else {
                        let status = resp.status();
                        let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        error!("Claude API validation failed with status {}: {}", status, error_text);
                        Err(format!("API validation failed: {} - {}", status, error_text))
                    }
                }
                Err(e) => {
                    error!("Failed to connect to Claude API for validation: {}", e);
                    Err(format!("Connection failed: {}", e))
                }
            }
        }
        "perplexity" => {
            // Test Perplexity API
            let response = client
                .post("https://api.perplexity.ai/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({
                    "model": "llama-3.1-sonar-small-128k-online",
                    "messages": [{"role": "user", "content": "hi"}],
                    "max_tokens": 1
                }))
                .send()
                .await;
            
            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        info!("Perplexity API key validation successful");
                        Ok(true)
                    } else if resp.status() == 401 {
                        info!("Perplexity API key validation failed - unauthorized");
                        Ok(false)
                    } else {
                        let status = resp.status();
                        let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        error!("Perplexity API validation failed with status {}: {}", status, error_text);
                        Err(format!("API validation failed: {} - {}", status, error_text))
                    }
                }
                Err(e) => {
                    error!("Failed to connect to Perplexity API for validation: {}", e);
                    Err(format!("Connection failed: {}", e))
                }
            }
        }
        "openai" => {
            // Test OpenAI API
            let response = client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({
                    "model": "gpt-3.5-turbo",
                    "messages": [{"role": "user", "content": "hi"}],
                    "max_tokens": 1
                }))
                .send()
                .await;
            
            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        info!("OpenAI API key validation successful");
                        Ok(true)
                    } else if resp.status() == 401 {
                        info!("OpenAI API key validation failed - unauthorized");
                        Ok(false)
                    } else {
                        let status = resp.status();
                        let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        error!("OpenAI API validation failed with status {}: {}", status, error_text);
                        Err(format!("API validation failed: {} - {}", status, error_text))
                    }
                }
                Err(e) => {
                    error!("Failed to connect to OpenAI API for validation: {}", e);
                    Err(format!("Connection failed: {}", e))
                }
            }
        }
        _ => {
            error!("Unknown provider for validation: {}", provider);
            Err(format!("Unsupported provider: {}", provider))
        }
    }
}

fn main() {
    // Initialize logger
    env_logger::init();
    
    // Load .env file
    dotenvy::dotenv().ok();
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, ask_claude, stream_claude, get_env, store_api_key, get_api_key, list_api_key_providers, delete_api_key, get_provider_info, migrate_api_keys, validate_api_key])
        .setup(|app| {
            // Auto-migrate API keys on startup
            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = auto_migrate_keys(&app_handle).await {
                    error!("Failed to auto-migrate API keys on startup: {}", e);
                } else {
                    info!("Auto-migration check completed successfully");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn auto_migrate_keys(app: &tauri::AppHandle) -> Result<(), String> {
    info!("Checking for API keys to migrate from config file");
    
    let config_path = get_config_path();
    if !config_path.exists() {
        info!("No config file found, skipping migration");
        return Ok(());
    }
    
    let contents = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    let mut store = api_keys::ApiKeyStore::load(app)?;
    let mut config_modified = false;
    let mut new_config_lines = Vec::new();
    let mut migrated_keys = Vec::new();
    
    for line in contents.lines() {
        if let Some(key) = line.strip_prefix("CLAUDE_API_KEY=") {
            if !store.providers.contains_key("claude") && !key.trim().is_empty() {
                info!("Migrating Claude API key from config file to secure storage");
                match store.store_key("claude", "Claude API", key.trim(), app) {
                    Ok(_) => {
                        info!("Successfully migrated Claude API key to secure storage");
                        migrated_keys.push("Claude");
                        // Don't add this line to new config - effectively removing it
                        config_modified = true;
                        continue;
                    }
                    Err(e) => {
                        error!("Failed to migrate Claude API key: {}", e);
                        // Keep the line in config if migration failed
                    }
                }
            } else if store.providers.contains_key("claude") {
                info!("Claude API key already in secure storage, removing from config file");
                config_modified = true;
                continue;
            }
        }
        
        // Add all other lines (including failed migrations) to new config
        new_config_lines.push(line);
    }
    
    // Write back the modified config file if we removed any keys
    if config_modified {
        let new_contents = new_config_lines.join("\n");
        if let Err(e) = fs::write(&config_path, new_contents) {
            error!("Failed to update config file after migration: {}", e);
        } else {
            info!("Updated config file to remove migrated keys");
        }
    }
    
    // Notify frontend about migration
    if !migrated_keys.is_empty() {
        let message = format!("Migrated {} API key(s) to secure storage: {}", 
                             migrated_keys.len(), 
                             migrated_keys.join(", "));
        
        if let Err(e) = app.emit_all("api-keys-migrated", message) {
            error!("Failed to emit migration notification: {}", e);
        }
    }
    
    Ok(())
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
    #[serde(skip_serializing_if = "Option::is_none")]
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
#[serde(tag = "type")]
enum ClaudeStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: serde_json::Value },
    #[serde(rename = "content_block_start")]
    ContentBlockStart { index: u32, content_block: serde_json::Value },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: u32, delta: ClaudeStreamDelta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: u32 },
    #[serde(rename = "message_delta")]
    MessageDelta { delta: serde_json::Value, usage: serde_json::Value },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Deserialize, Debug)]
struct ClaudeStreamDelta {
    #[serde(rename = "type")]
    delta_type: String,
    text: String,
}

fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".olly").join("config.env")
}

fn load_api_key(app: &tauri::AppHandle) -> Result<String, String> {
    // First try secure storage
    if let Ok(mut store) = api_keys::ApiKeyStore::load(app) {
        if let Ok(Some(key)) = store.get_key("claude", app) {
            info!("Loaded Claude API key from secure storage");
            return Ok(key);
        }
    }
    
    // Fallback to environment variable
    if let Ok(key) = std::env::var("CLAUDE_API_KEY") {
        info!("Loaded Claude API key from environment variable");
        return Ok(key);
    }
    
    // Fallback to config file
    let config_path = get_config_path();
    if let Ok(contents) = fs::read_to_string(config_path) {
        for line in contents.lines() {
            if let Some(key) = line.strip_prefix("CLAUDE_API_KEY=") {
                info!("Loaded Claude API key from config file");
                return Ok(key.to_string());
            }
        }
    }
    
    Err("API key not found in secure storage, environment, or config file".to_string())
}

#[tauri::command]
async fn ask_claude(app: tauri::AppHandle, prompt: String) -> Result<String, String> {
    info!("Starting ask_claude with prompt: {}", prompt);
    
    let client = reqwest::Client::new();
    
    // Load API key from secure storage, environment, or config file
    let api_key = load_api_key(&app)?;
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
async fn stream_claude(window: tauri::Window, app: tauri::AppHandle, prompt: String) -> Result<(), String> {
    info!("Starting stream_claude with prompt: {}", prompt);
    
    let client = reqwest::Client::new();
    
    // Load API key from secure storage, environment, or config file
    let api_key = load_api_key(&app)?;
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
    let mut buffer = String::new();
    
    while let Some(item) = stream.next().await {
        match item {
            Ok(bytes) => {
                let chunk = String::from_utf8_lossy(&bytes);
                info!("Received chunk from Claude: {}", chunk);
                
                // Add chunk to buffer to handle split JSON objects
                buffer.push_str(&chunk);
                
                // Process complete lines from buffer
                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].to_string();
                    buffer = buffer[line_end + 1..].to_string();
                    
                    if line.is_empty() || line == "data: [DONE]" {
                        continue;
                    }
                    
                    // Remove the "data: " prefix if present
                    let json_str = if line.starts_with("data: ") {
                        &line[6..]
                    } else {
                        &line
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
                    match serde_json::from_str::<ClaudeStreamEvent>(json_str) {
                        Ok(event) => {
                            match event {
                                ClaudeStreamEvent::ContentBlockDelta { delta, .. } => {
                                    if !delta.text.is_empty() {
                                        let content = &delta.text;
                                        info!("Parsed content from Claude delta: {}", content);
                                        full_response.push_str(content);
                                        
                                        // Emit event to frontend
                                        if let Err(e) = window.emit("claude-stream", content) {
                                            error!("Failed to emit claude-stream event: {}", e);
                                        }
                                    }
                                }
                                ClaudeStreamEvent::MessageStart { .. } => {
                                    info!("Claude message started");
                                }
                                ClaudeStreamEvent::ContentBlockStart { .. } => {
                                    info!("Claude content block started");
                                }
                                ClaudeStreamEvent::ContentBlockStop { .. } => {
                                    info!("Claude content block stopped");
                                }
                                ClaudeStreamEvent::MessageStop => {
                                    info!("Claude message stopped");
                                }
                                ClaudeStreamEvent::MessageDelta { .. } => {
                                    info!("Claude message delta received");
                                }
                                ClaudeStreamEvent::Ping => {
                                    info!("Claude ping received");
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
    
    // Process any remaining content in buffer
    if !buffer.trim().is_empty() {
        info!("Processing remaining buffer content: {}", buffer);
        
        for line in buffer.lines() {
            if line.is_empty() || line == "data: [DONE]" {
                continue;
            }
            
            let json_str = if line.starts_with("data: ") {
                &line[6..]
            } else {
                line
            };
            
            if line.starts_with("event:") || json_str.trim().is_empty() || !json_str.trim().starts_with('{') {
                continue;
            }
            
            match serde_json::from_str::<ClaudeStreamEvent>(json_str) {
                Ok(event) => {
                    if let ClaudeStreamEvent::ContentBlockDelta { delta, .. } = event {
                        if !delta.text.is_empty() {
                            full_response.push_str(&delta.text);
                            if let Err(e) = window.emit("claude-stream", &delta.text) {
                                error!("Failed to emit claude-stream event from buffer: {}", e);
                            }
                        }
                    }
                },
                Err(_) => {
                    // Try salvage operation for remaining buffer
                    if let Some(content_start) = json_str.find("\"text\": \"") {
                        if let Some(content_end) = json_str[content_start + 9..].find("\"") {
                            let content = &json_str[content_start + 9..content_start + 9 + content_end];
                            full_response.push_str(content);
                            if let Err(e) = window.emit("claude-stream", content) {
                                error!("Failed to emit salvaged content from buffer: {}", e);
                            }
                        }
                    }
                }
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
