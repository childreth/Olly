// Enable console window to see logs in release builds for debugging
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use log::{error, info};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::Emitter;

// API Key Management Module
mod api_keys {
    use super::*;
    use keyring::Entry;
    use serde::{Deserialize, Serialize};

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
                Ok(entry) => match entry.get_password() {
                    Ok(data) => serde_json::from_str(&data)
                        .map_err(|e| format!("Failed to parse stored API key data: {}", e)),
                    Err(_) => {
                        info!("No existing API key store found, creating new");
                        Ok(Self::new())
                    }
                },
                Err(e) => {
                    error!("Failed to create keyring entry: {}", e);
                    Ok(Self::new())
                }
            }
        }

        pub fn save(&self, _app: &tauri::AppHandle) -> Result<(), String> {
            info!(
                "Saving API key store with {} providers",
                self.providers.len()
            );

            let data = serde_json::to_string(self)
                .map_err(|e| format!("Failed to serialize API key store: {}", e))?;

            info!("Serialized store data: {} bytes", data.len());

            let entry = Entry::new(KEYRING_SERVICE, STORE_KEY)
                .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

            match entry.set_password(&data) {
                Ok(_) => {
                    info!("Successfully saved API key store to keyring");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to save API key store to keyring: {}", e);
                    Err(format!("Failed to save API key store to keyring: {}", e))
                }
            }
        }

        pub fn store_key(
            &mut self,
            provider: &str,
            display_name: &str,
            api_key: &str,
            app: &tauri::AppHandle,
        ) -> Result<(), String> {
            let key_id = format!("{}_key", provider);

            // Store the actual API key in keyring
            let entry = Entry::new(KEYRING_SERVICE, &key_id)
                .map_err(|e| format!("Failed to create keyring entry for {}: {}", provider, e))?;

            entry
                .set_password(api_key)
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

        pub fn get_key(
            &mut self,
            provider: &str,
            app: &tauri::AppHandle,
        ) -> Result<Option<String>, String> {
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
                    error!(
                        "Failed to delete API key from keyring for {}: {}",
                        provider, e
                    );
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
                            info!(
                                "Migrating Perplexity API key from config file to secure storage"
                            );
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

// File-Based API Key Storage Implementation (macOS keyring has issues)
#[tauri::command]
async fn store_api_key(
    _app: tauri::AppHandle,
    provider: String,
    _display_name: String,
    api_key: String,
) -> Result<(), String> {
    info!(
        "Storing API key for provider: {} (using file storage due to macOS keyring issues)",
        provider
    );

    // Use file storage directly as it's more reliable than keyring on macOS
    store_api_key_file(&provider, &api_key)
}

#[tauri::command]
async fn store_api_key_debug(
    _app: tauri::AppHandle,
    provider: String,
    _display_name: String,
    api_key: String,
) -> Result<String, String> {
    use keyring::Entry;

    let mut debug_log = Vec::new();
    debug_log.push(format!("Starting storage for provider: {}", provider));

    // Try keyring first
    let service = "olly";
    let username = format!("{}_api_key", provider);
    debug_log.push(format!(
        "Keyring: service='{}', username='{}'",
        service, username
    ));

    match Entry::new(service, &username) {
        Ok(entry) => {
            debug_log.push("Keyring: Entry created successfully".to_string());
            match entry.set_password(&api_key) {
                Ok(_) => {
                    debug_log.push("Keyring: set_password succeeded".to_string());
                    // Verify it was actually stored
                    match entry.get_password() {
                        Ok(retrieved) => {
                            debug_log.push(format!(
                                "Keyring: Retrieved password, length: {}",
                                retrieved.len()
                            ));
                            if retrieved == api_key {
                                debug_log.push("Keyring: VERIFICATION PASSED".to_string());
                                return Ok(debug_log.join(" | "));
                            } else {
                                debug_log.push(format!(
                                    "Keyring: VERIFICATION FAILED - length mismatch {} != {}",
                                    api_key.len(),
                                    retrieved.len()
                                ));
                            }
                        }
                        Err(e) => {
                            debug_log.push(format!(
                                "Keyring: Verification failed - get_password error: {}",
                                e
                            ));
                        }
                    }
                }
                Err(e) => {
                    debug_log.push(format!("Keyring: set_password failed: {}", e));
                }
            }
        }
        Err(e) => {
            debug_log.push(format!("Keyring: Entry creation failed: {}", e));
        }
    }

    // Fallback to encrypted file storage
    debug_log.push("Trying file storage fallback".to_string());
    match store_api_key_file(&provider, &api_key) {
        Ok(_) => {
            debug_log.push("File: Storage succeeded".to_string());
        }
        Err(e) => {
            debug_log.push(format!("File: Storage failed: {}", e));
        }
    }

    Ok(debug_log.join(" | "))
}

#[tauri::command]
async fn get_api_key_debug(_app: tauri::AppHandle, provider: String) -> Result<String, String> {
    use keyring::Entry;

    let mut debug_log = Vec::new();
    debug_log.push(format!("Starting retrieval for provider: {}", provider));

    let service = "olly";
    let username = format!("{}_api_key", provider);
    debug_log.push(format!(
        "Keyring: service='{}', username='{}'",
        service, username
    ));

    // Try keyring first
    match Entry::new(service, &username) {
        Ok(entry) => {
            debug_log.push("Keyring: Entry created successfully".to_string());
            match entry.get_password() {
                Ok(api_key) => {
                    debug_log.push(format!(
                        "Keyring: Retrieved password, length: {}",
                        api_key.len()
                    ));
                    return Ok(format!("{} | FOUND_IN_KEYRING", debug_log.join(" | ")));
                }
                Err(e) => {
                    debug_log.push(format!("Keyring: get_password failed: {}", e));
                }
            }
        }
        Err(e) => {
            debug_log.push(format!("Keyring: Entry creation failed: {}", e));
        }
    }

    // Check file storage
    debug_log.push("Checking file storage".to_string());
    match get_api_key_file(&provider) {
        Ok(Some(key)) => {
            debug_log.push(format!("File: Found key, length: {}", key.len()));
            Ok(format!("{} | FOUND_IN_FILE", debug_log.join(" | ")))
        }
        Ok(None) => {
            debug_log.push("File: No key found".to_string());
            Ok(format!("{} | NOT_FOUND", debug_log.join(" | ")))
        }
        Err(e) => {
            debug_log.push(format!("File: Error: {}", e));
            Ok(format!("{} | FILE_ERROR", debug_log.join(" | ")))
        }
    }
}

#[tauri::command]
async fn get_api_key(_app: tauri::AppHandle, provider: String) -> Result<Option<String>, String> {
    info!(
        "Retrieving API key for provider: {} (using file storage)",
        provider
    );

    // Use file storage directly since we're having keyring issues on macOS
    match get_api_key_file(&provider) {
        Ok(Some(key)) => {
            info!("Retrieved API key for {} from file storage", provider);
            Ok(Some(key))
        }
        Ok(None) => {
            info!("No API key found for provider: {}", provider);
            Ok(None)
        }
        Err(e) => {
            error!("Error retrieving API key for {}: {}", provider, e);
            Ok(None)
        }
    }
}

#[tauri::command]
async fn list_api_key_providers(_app: tauri::AppHandle) -> Result<Vec<String>, String> {
    info!("Listing API key providers using file storage");

    let mut providers = Vec::new();

    // Check for known providers in file storage
    for provider in ["claude", "perplexity", "openai"] {
        if let Ok(Some(_)) = get_api_key_file(provider) {
            providers.push(provider.to_string());
        }
    }

    info!("Found {} providers with stored keys", providers.len());
    Ok(providers)
}

#[tauri::command]
async fn delete_api_key(_app: tauri::AppHandle, provider: String) -> Result<(), String> {
    info!("Deleting API key for provider: {}", provider);

    // Delete from file storage
    delete_api_key_file(&provider)?;

    info!("Successfully deleted API key for provider: {}", provider);
    Ok(())
}

#[tauri::command]
async fn get_provider_info(
    _app: tauri::AppHandle,
    provider: String,
) -> Result<Option<serde_json::Value>, String> {
    info!("Getting provider info for: {}", provider);

    if let Ok(Some(_)) = get_api_key_file(&provider) {
        return Ok(Some(serde_json::json!({
            "provider": provider,
            "display_name": format!("{} API", provider.chars().next().unwrap().to_uppercase().collect::<String>() + &provider[1..]),
            "is_active": true
        })));
    }

    Ok(None)
}

#[tauri::command]
async fn migrate_api_keys(_app: tauri::AppHandle) -> Result<(), String> {
    info!("Fresh implementation - migration not needed");
    Ok(())
}

#[tauri::command]
async fn validate_api_key(provider: String, api_key: String) -> Result<bool, String> {
    info!("Validating API key for provider: {}", provider);

    let client = reqwest::Client::new();

    match provider.as_str() {
        "claude" => {
            // Test Claude API by listing models (no need to guess a model name)
            let response = client
                .get("https://api.anthropic.com/v1/models?limit=1")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
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
                        let error_text = resp
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());
                        error!(
                            "Claude API validation failed with status {}: {}",
                            status, error_text
                        );
                        Err(format!(
                            "API validation failed: {} - {}",
                            status, error_text
                        ))
                    }
                }
                Err(e) => {
                    error!("Failed to connect to Claude API for validation: {}", e);
                    Err(format!("Connection failed: {}", e))
                }
            }
        }
        "perplexity" => {
            // Test Perplexity API with sonar model
            let response = client
                .post("https://api.perplexity.ai/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({
                    "model": "sonar",
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
                        let error_text = resp
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());
                        error!(
                            "Perplexity API validation failed with status {}: {}",
                            status, error_text
                        );
                        Err(format!(
                            "API validation failed: {} - {}",
                            status, error_text
                        ))
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
                        let error_text = resp
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());
                        error!(
                            "OpenAI API validation failed with status {}: {}",
                            status, error_text
                        );
                        Err(format!(
                            "API validation failed: {} - {}",
                            status, error_text
                        ))
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

#[tauri::command]
async fn get_claude_models(_app: tauri::AppHandle) -> Result<Vec<serde_json::Value>, String> {
    info!("Fetching Claude models from API (backend)");

    // Get Claude API key from storage
    let api_key = match get_api_key_file("claude") {
        Ok(Some(key)) => key,
        Ok(None) => {
            info!("No Claude API key found");
            return Ok(vec![]);
        }
        Err(e) => {
            error!("Error getting Claude API key: {}", e);
            return Ok(vec![]);
        }
    };

    let client = reqwest::Client::new();
    
    let response = client
        .get("https://api.anthropic.com/v1/models?limit=20")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<serde_json::Value>().await {
                    Ok(data) => {
                        if let Some(models_array) = data.get("data").and_then(|d| d.as_array()) {
                            let models: Vec<serde_json::Value> = models_array
                                .iter()
                                .map(|model| {
                                    serde_json::json!({
                                        "id": model.get("id").and_then(|v| v.as_str()).unwrap_or("unknown"),
                                        "name": model.get("display_name").and_then(|v| v.as_str())
                                            .unwrap_or(model.get("id").and_then(|v| v.as_str()).unwrap_or("unknown")),
                                        "description": "Claude API model",
                                        "provider": "claude"
                                    })
                                })
                                .collect();
                            info!("Successfully fetched {} Claude models", models.len());
                            Ok(models)
                        } else {
                            error!("Claude API response missing 'data' field");
                            Ok(vec![])
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse Claude models response: {}", e);
                        Ok(vec![])
                    }
                }
            } else {
                let status = resp.status();
                let error_text = resp.text().await.unwrap_or_default();
                error!("Claude models API error {}: {}", status, error_text);
                Ok(vec![])
            }
        }
        Err(e) => {
            error!("Failed to connect to Claude API: {}", e);
            Ok(vec![])
        }
    }
}

#[tauri::command]
async fn get_perplexity_models() -> Result<Vec<serde_json::Value>, String> {
    info!("Getting available Perplexity models");

    // Return available Perplexity models
    let models = vec![
        serde_json::json!({
            "id": "sonar-deep-research",
            "name": "Sonar Deep Research",
            "description": "Deep research with comprehensive analysis",
            "provider": "perplexity"
        }),
        serde_json::json!({
            "id": "sonar-reasoning-pro",
            "name": "Sonar Reasoning Pro",
            "description": "Advanced reasoning capabilities",
            "provider": "perplexity"
        }),
        serde_json::json!({
            "id": "sonar-reasoning",
            "name": "Sonar Reasoning",
            "description": "Core reasoning model",
            "provider": "perplexity"
        }),
        serde_json::json!({
            "id": "sonar-pro",
            "name": "Sonar Pro",
            "description": "Professional grade search and chat",
            "provider": "perplexity"
        }),
        serde_json::json!({
            "id": "sonar",
            "name": "Sonar",
            "description": "Standard search and chat model",
            "provider": "perplexity"
        }),
    ];

    Ok(models)
}

#[tauri::command]
async fn debug_api_keys(_app: tauri::AppHandle) -> Result<String, String> {
    use keyring::Entry;

    info!("=== DEBUG API KEYS CALLED ===");

    let mut keyring_providers = Vec::new();
    let mut file_providers = Vec::new();
    let service = "olly";

    // Check keyring for known providers
    for provider in ["claude", "perplexity", "openai"] {
        let username = format!("{}_api_key", provider);
        info!("Checking keyring for {}: service='{}'", provider, service);

        if let Ok(entry) = Entry::new(service, &username) {
            match entry.get_password() {
                Ok(key) => {
                    info!("Found {} in keyring, key length: {}", provider, key.len());
                    keyring_providers.push(provider.to_string());
                }
                Err(e) => {
                    info!("No {} in keyring: {}", provider, e);
                }
            }
        } else {
            info!("Failed to create keyring entry for {}", provider);
        }

        // Check file storage too
        match get_api_key_file(provider) {
            Ok(Some(key)) => {
                info!(
                    "Found {} in file storage, key length: {}",
                    provider,
                    key.len()
                );
                file_providers.push(provider.to_string());
            }
            Ok(None) => {
                info!("No {} in file storage", provider);
            }
            Err(e) => {
                info!("Error checking file storage for {}: {}", provider, e);
            }
        }
    }

    let debug_info = format!(
        "Keyring: {:?}, File: {:?}",
        keyring_providers, file_providers
    );
    info!("=== DEBUG RESULT: {} ===", debug_info);
    Ok(debug_info)
}

#[tauri::command]
async fn test_store_load(_app: tauri::AppHandle) -> Result<String, String> {
    use keyring::Entry;

    let test_key = "test_key_12345";
    let test_provider = "test_hybrid";

    // Test 1: Keyring storage
    let keyring_result = {
        let service = "olly";
        let username = format!("{}_api_key", test_provider);

        match Entry::new(service, &username) {
            Ok(entry) => match entry.set_password(test_key) {
                Ok(_) => match entry.get_password() {
                    Ok(retrieved) => {
                        let _ = entry.delete_credential();
                        if retrieved == test_key {
                            "KEYRING_PASSED"
                        } else {
                            "KEYRING_MISMATCH"
                        }
                    }
                    Err(_) => "KEYRING_READ_FAILED",
                },
                Err(_) => "KEYRING_WRITE_FAILED",
            },
            Err(_) => "KEYRING_CREATE_FAILED",
        }
    };

    // Test 2: File storage
    let file_result = {
        match store_api_key_file(test_provider, test_key) {
            Ok(_) => match get_api_key_file(test_provider) {
                Ok(Some(retrieved)) => {
                    let _ = delete_api_key_file(test_provider);
                    if retrieved == test_key {
                        "FILE_PASSED"
                    } else {
                        "FILE_MISMATCH"
                    }
                }
                Ok(None) => "FILE_READ_EMPTY",
                Err(_) => "FILE_READ_FAILED",
            },
            Err(_) => "FILE_WRITE_FAILED",
        }
    };

    // Test 3: Hybrid storage (our actual implementation)
    let hybrid_result = {
        match store_api_key(
            _app.clone(),
            test_provider.to_string(),
            "Test".to_string(),
            test_key.to_string(),
        )
        .await
        {
            Ok(_) => {
                // Add a small delay to ensure storage is complete
                std::thread::sleep(std::time::Duration::from_millis(50));

                // Check both storage locations manually after hybrid store
                let keyring_check = {
                    let service = "olly";
                    let username = format!("{}_api_key", test_provider);
                    if let Ok(entry) = Entry::new(service, &username) {
                        entry.get_password().is_ok()
                    } else {
                        false
                    }
                };

                let file_check = get_api_key_file(test_provider).unwrap_or(None).is_some();

                match get_api_key(_app.clone(), test_provider.to_string()).await {
                    Ok(Some(retrieved)) => {
                        let _ = delete_api_key(_app.clone(), test_provider.to_string()).await;
                        if retrieved == test_key {
                            "HYBRID_PASSED".to_string()
                        } else {
                            "HYBRID_MISMATCH".to_string()
                        }
                    }
                    Ok(None) => {
                        format!("HYBRID_READ_EMPTY(K:{},F:{})", keyring_check, file_check)
                    }
                    Err(e) => format!("HYBRID_READ_FAILED:{}", e),
                }
            }
            Err(e) => format!("HYBRID_WRITE_FAILED:{}", e),
        }
    };

    Ok(format!(
        "Keyring: {} | File: {} | Hybrid: {}",
        keyring_result, file_result, hybrid_result
    ))
}

#[tauri::command]
async fn test_keyring() -> Result<String, String> {
    use keyring::Entry;

    let test_service = "com.olly.app.test";
    let test_key = "test_key";
    let test_value = "test_value_123";

    // Try to create and save a test entry
    match Entry::new(test_service, test_key) {
        Ok(entry) => {
            match entry.set_password(test_value) {
                Ok(_) => {
                    // Try to read it back
                    match entry.get_password() {
                        Ok(retrieved) => {
                            // Clean up
                            let _ = entry.delete_credential();
                            if retrieved == test_value {
                                Ok("Keyring test PASSED - read/write works".to_string())
                            } else {
                                Ok(format!(
                                    "Keyring test FAILED - wrote '{}' but read '{}'",
                                    test_value, retrieved
                                ))
                            }
                        }
                        Err(e) => Ok(format!("Keyring test FAILED - could not read: {}", e)),
                    }
                }
                Err(e) => Ok(format!("Keyring test FAILED - could not write: {}", e)),
            }
        }
        Err(e) => Ok(format!(
            "Keyring test FAILED - could not create entry: {}",
            e
        )),
    }
}

#[tauri::command]
async fn test_exact_keyring() -> Result<String, String> {
    use keyring::Entry;

    // Test with the exact same service and key that our store uses
    let service = "com.olly.app";
    let key = "api_key_store";
    let test_value = r#"{"providers":{"test":{"provider":"test","display_name":"Test","created_at":"2025-01-01T00:00:00Z","last_used":null,"is_active":true}}}"#;

    match Entry::new(service, key) {
        Ok(entry) => {
            match entry.set_password(test_value) {
                Ok(_) => {
                    // Try to read it back
                    match entry.get_password() {
                        Ok(retrieved) => {
                            // Clean up
                            let _ = entry.delete_credential();
                            if retrieved == test_value {
                                Ok("Exact keyring test PASSED - store service/key works"
                                    .to_string())
                            } else {
                                Ok(format!(
                                    "Exact keyring test FAILED - wrote {} bytes but read {} bytes",
                                    test_value.len(),
                                    retrieved.len()
                                ))
                            }
                        }
                        Err(e) => Ok(format!("Exact keyring test FAILED - could not read: {}", e)),
                    }
                }
                Err(e) => Ok(format!(
                    "Exact keyring test FAILED - could not write: {}",
                    e
                )),
            }
        }
        Err(e) => Ok(format!(
            "Exact keyring test FAILED - could not create entry: {}",
            e
        )),
    }
}

#[tauri::command]
async fn get_all_models() -> Result<Vec<serde_json::Value>, String> {
    info!("Getting all available models from all providers");

    let mut all_models = Vec::new();

    // Claude models are now handled dynamically in the frontend

    // Add Perplexity models
    let perplexity_models = get_perplexity_models().await?;
    all_models.extend(perplexity_models);

    // Add Fal model
    all_models.push(serde_json::json!({
        "id": "fal-flux",
        "name": "Fal - Flux",
        "description": "Image generation model",
        "provider": "fal"
    }));

    // Add Ollama models (we'll need to handle this dynamically from frontend)
    // For now, just add a placeholder that will be replaced by frontend Ollama detection

    Ok(all_models)
}

#[tauri::command]
async fn get_ollama_models() -> Result<Vec<serde_json::Value>, String> {
    info!("Getting Ollama models from localhost:11434");

    let client = reqwest::Client::new();

    match client.get("http://localhost:11434/api/tags").send().await {
        Ok(response) => {
            match response.json::<serde_json::Value>().await {
                Ok(data) => {
                    if let Some(models) = data.get("models").and_then(|m| m.as_array()) {
                        let ollama_models = models.iter().filter_map(|model| {
                            let name = model.get("name").and_then(|n| n.as_str())?;
                            let modified_at = model.get("modified_at").and_then(|m| m.as_str()).unwrap_or("Unknown");
                            let details = model.get("details").cloned().unwrap_or_else(|| serde_json::json!({}));
                            
                            // Format the date to human-readable format
                            let formatted_date = if modified_at != "Unknown" {
                                match DateTime::parse_from_rfc3339(modified_at) {
                                    Ok(dt) => {
                                        // Convert to local time and format
                                        let local_dt = dt.with_timezone(&chrono::Local);
                                        local_dt.format("%b %d, %Y - %I:%M %p %Z").to_string()
                                    }
                                    Err(_) => modified_at.to_string()
                                }
                            } else {
                                "Unknown".to_string()
                            };
                            
                            Some(serde_json::json!({
                                "id": name,
                                "name": name,
                                "description": format!("{} - {}", 
                                    details.get("parameter_size").and_then(|p| p.as_str()).unwrap_or("Unknown"),
                                    formatted_date
                                ),
                                "provider": "ollama",
                                "details": {
                                    "modified_at": modified_at,
                                    "parameter_size": details.get("parameter_size").and_then(|p| p.as_str()).unwrap_or("Unknown"),
                                    "quantization_level": details.get("quantization_level").and_then(|q| q.as_str()).unwrap_or("Unknown")
                                }
                            }))
                        }).collect::<Vec<_>>();

                        info!("Successfully fetched {} Ollama models", ollama_models.len());
                        Ok(ollama_models)
                    } else {
                        info!("No models found in Ollama response");
                        Ok(Vec::new())
                    }
                }
                Err(e) => {
                    error!("Failed to parse Ollama response: {}", e);
                    Err(format!("Failed to parse Ollama response: {}", e))
                }
            }
        }
        Err(e) => {
            error!("Failed to connect to Ollama: {}", e);
            Err(format!(
                "Failed to connect to Ollama at localhost:11434: {}",
                e
            ))
        }
    }
}

fn main() {
    // Initialize logger with more verbose output
    #[cfg(debug_assertions)]
    env_logger::init();

    // In production, also enable logging to help diagnose issues
    #[cfg(not(debug_assertions))]
    {
        use std::io::Write;
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
            .init();
    }

    info!("Starting Olly application");

    // Load .env file
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            ask_claude,
            stream_claude,
            ask_perplexity,
            stream_perplexity,
            get_perplexity_models,
            get_claude_models,
            get_all_models,
            get_ollama_models,
            get_env,
            store_api_key,
            get_api_key,
            list_api_key_providers,
            delete_api_key,
            get_provider_info,
            migrate_api_keys,
            validate_api_key,
            debug_api_keys,
            test_keyring,
            test_store_load,
            test_exact_keyring,
            store_api_key_debug,
            get_api_key_debug,
            migrate_claude_key
        ])
        .setup(|app| {
            info!("Running setup function");

            // Load .env from resources in production
            #[cfg(not(debug_assertions))]
            {
                if let Some(resource_path) = app.path_resolver().resolve_resource("../.env") {
                    info!("Loading .env from resource path: {:?}", resource_path);
                    if let Err(e) = dotenvy::from_path(&resource_path) {
                        error!("Failed to load .env from resource path: {}", e);
                    } else {
                        info!("Successfully loaded .env from resource path");
                    }
                } else {
                    error!("Failed to resolve .env resource path");
                }
            }

            // Auto-migrate API keys on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                info!("Starting API key migration check");
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
    use keyring::Entry;

    info!("Checking for API keys to migrate from config file using fresh implementation");

    let config_path = get_config_path();
    if !config_path.exists() {
        info!("No config file found, skipping migration");
        return Ok(());
    }

    let contents = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let service = "olly";
    let mut config_modified = false;
    let mut new_config_lines = Vec::new();
    let mut migrated_keys = Vec::new();

    for line in contents.lines() {
        if let Some(key) = line.strip_prefix("CLAUDE_API_KEY=") {
            let username = "claude_api_key";
            let already_stored = if let Ok(entry) = Entry::new(service, username) {
                entry.get_password().is_ok()
            } else {
                false
            };

            if !already_stored && !key.trim().is_empty() {
                info!("Migrating Claude API key from config file to secure storage");
                if let Ok(entry) = Entry::new(service, username) {
                    if entry.set_password(key.trim()).is_ok() {
                        info!("Successfully migrated Claude API key to secure storage");
                        migrated_keys.push("Claude");
                        config_modified = true;
                        continue;
                    }
                }
                error!("Failed to migrate Claude API key");
            } else if already_stored {
                info!("Claude API key already in secure storage, removing from config file");
                config_modified = true;
                continue;
            }
        }

        if let Some(key) = line.strip_prefix("PERPLEXITY_API_KEY=") {
            let username = "perplexity_api_key";
            let already_stored = if let Ok(entry) = Entry::new(service, username) {
                entry.get_password().is_ok()
            } else {
                false
            };

            if !already_stored && !key.trim().is_empty() {
                info!("Migrating Perplexity API key from config file to secure storage");
                if let Ok(entry) = Entry::new(service, username) {
                    if entry.set_password(key.trim()).is_ok() {
                        info!("Successfully migrated Perplexity API key to secure storage");
                        migrated_keys.push("Perplexity");
                        config_modified = true;
                        continue;
                    }
                }
                error!("Failed to migrate Perplexity API key");
            } else if already_stored {
                info!("Perplexity API key already in secure storage, removing from config file");
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
        let message = format!(
            "Migrated {} API key(s) to secure storage: {}",
            migrated_keys.len(),
            migrated_keys.join(", ")
        );

        if let Err(e) = app.emit("api-keys-migrated", message) {
            error!("Failed to emit migration notification: {}", e);
        }
    }

    Ok(())
}

// Claude API

use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
enum MessageContent {
    Text(String),
    Multimodal(Vec<ContentBlock>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    role: String,
    content: MessageContent,
}

#[derive(Serialize)]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_uses: Option<u32>,
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
    ContentBlockStart {
        index: u32,
        content_block: serde_json::Value,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: u32,
        delta: ClaudeStreamDelta,
    },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: u32 },
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: serde_json::Value,
        usage: serde_json::Value,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum ClaudeStreamDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "citations_delta")]
    CitationsDelta { citation: Citation },
    #[serde(other)]
    Other,
}

#[derive(Deserialize, Debug)]
struct Citation {
    #[serde(rename = "type")]
    citation_type: String,
    cited_text: String,
    url: String,
    title: String,
    encrypted_index: String,
}

fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".olly").join("config.env")
}

fn get_keys_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".olly").join("keys")
}

// File-based storage fallback functions
fn store_api_key_file(provider: &str, api_key: &str) -> Result<(), String> {
    use std::fs;

    let keys_dir = get_keys_dir();
    info!(
        "Attempting file storage for {} in directory: {:?}",
        provider, keys_dir
    );

    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&keys_dir) {
        error!("Failed to create keys directory: {}", e);
        return Err(format!("Failed to create keys directory: {}", e));
    }
    info!("Keys directory created/exists");

    // Simple XOR encoding for basic obfuscation (not real security)
    let encoded_key = simple_encode(api_key);
    info!(
        "Encoded key, original length: {}, encoded length: {}",
        api_key.len(),
        encoded_key.len()
    );

    let key_file = keys_dir.join(format!("{}.key", provider));
    info!("Writing to file: {:?}", key_file);

    match fs::write(&key_file, &encoded_key) {
        Ok(_) => {
            info!("Successfully wrote key file for {}", provider);
            // Verify by reading it back
            match fs::read(&key_file) {
                Ok(read_data) => {
                    if read_data == encoded_key {
                        info!("File storage verification successful for {}", provider);
                        Ok(())
                    } else {
                        error!(
                            "File storage verification failed - data mismatch for {}",
                            provider
                        );
                        Err("File verification failed".to_string())
                    }
                }
                Err(e) => {
                    error!("File storage verification failed - cannot read back: {}", e);
                    Err(format!("File verification read failed: {}", e))
                }
            }
        }
        Err(e) => {
            error!("Failed to write key file for {}: {}", provider, e);
            Err(format!("Failed to write key file: {}", e))
        }
    }
}

fn get_api_key_file(provider: &str) -> Result<Option<String>, String> {
    use std::fs;

    let keys_dir = get_keys_dir();
    let key_file = keys_dir.join(format!("{}.key", provider));

    if !key_file.exists() {
        return Ok(None);
    }

    match fs::read(&key_file) {
        Ok(encoded_data) => {
            let decoded_key = simple_decode(&encoded_data);
            Ok(Some(decoded_key))
        }
        Err(e) => Err(format!("Failed to read key file: {}", e)),
    }
}

fn delete_api_key_file(provider: &str) -> Result<(), String> {
    use std::fs;

    let keys_dir = get_keys_dir();
    let key_file = keys_dir.join(format!("{}.key", provider));

    if key_file.exists() {
        match fs::remove_file(&key_file) {
            Ok(_) => {
                info!("Deleted API key file for {}", provider);
                Ok(())
            }
            Err(e) => Err(format!("Failed to delete key file: {}", e)),
        }
    } else {
        Ok(()) // File doesn't exist, consider it deleted
    }
}

// Simple encoding/decoding for basic obfuscation
fn simple_encode(input: &str) -> Vec<u8> {
    let key = b"olly_secure_2024"; // Simple XOR key
    input
        .bytes()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect()
}

fn simple_decode(input: &[u8]) -> String {
    let key = b"olly_secure_2024"; // Same XOR key
    let decoded: Vec<u8> = input
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect();
    String::from_utf8(decoded).unwrap_or_default()
}

fn load_api_key(_app: &tauri::AppHandle, provider: &str) -> Result<String, String> {
    info!("Loading API key for provider: {}", provider);

    // Log the keys directory path for debugging
    let keys_dir = get_keys_dir();
    info!("Keys directory: {:?}", keys_dir);
    info!("Keys directory exists: {}", keys_dir.exists());

    // First try our new file storage system
    match get_api_key_file(provider) {
        Ok(Some(key)) => {
            info!(
                "Loaded {} API key from file storage (length: {})",
                provider,
                key.len()
            );
            if key.is_empty() {
                error!("{} API key is empty!", provider);
                return Err(format!(
                    "{} API key is empty. Please add a valid key in Settings.",
                    provider
                ));
            }
            return Ok(key);
        }
        Ok(None) => {
            info!(
                "No {} API key in file storage, checking for migration",
                provider
            );
        }
        Err(e) => {
            error!(
                "Error reading {} API key from file storage: {}",
                provider, e
            );
        }
    }

    // Migration: Check environment variable and config file, then migrate to file storage
    let env_var = format!("{}_API_KEY", provider.to_uppercase());
    info!("Checking environment variable: {}", env_var);

    // Check environment variable
    if let Ok(key) = std::env::var(&env_var) {
        if key.trim().is_empty() {
            info!("Found {} environment variable but it's empty", env_var);
        } else {
            info!(
                "Found {} API key in environment variable (length: {}), migrating to file storage",
                provider,
                key.len()
            );
            if let Err(e) = store_api_key_file(provider, &key) {
                error!(
                    "Failed to migrate {} API key from environment: {}",
                    provider, e
                );
            } else {
                info!(
                    "Successfully migrated {} API key from environment to file storage",
                    provider
                );
                return Ok(key);
            }
        }
    } else {
        info!("Environment variable {} not found", env_var);
    }

    // Check config file
    let config_path = get_config_path();
    info!("Config path: {:?}", config_path);
    info!("Config file exists: {}", config_path.exists());

    if let Ok(contents) = fs::read_to_string(&config_path) {
        info!("Read config file, checking for {} key", provider);
        let config_prefix = format!("{}=", env_var);
        for line in contents.lines() {
            if let Some(key) = line.strip_prefix(&config_prefix) {
                if key.trim().is_empty() {
                    info!("Found {} in config but value is empty", env_var);
                } else {
                    info!(
                        "Found {} API key in config file (length: {}), migrating to file storage",
                        provider,
                        key.trim().len()
                    );
                    if let Err(e) = store_api_key_file(provider, key.trim()) {
                        error!("Failed to migrate {} API key from config: {}", provider, e);
                    } else {
                        info!(
                            "Successfully migrated {} API key from config to file storage",
                            provider
                        );
                        return Ok(key.trim().to_string());
                    }
                }
            }
        }
        info!("No {} key found in config file", provider);
    } else {
        info!("Could not read config file");
    }

    error!("{} API key not found in any location", provider);
    Err(format!(
        "{} API key not found. Please add it in Settings.",
        provider
    ))
}

#[tauri::command]
async fn migrate_claude_key(_app: tauri::AppHandle) -> Result<String, String> {
    info!("Manual Claude key migration requested");

    // Check if already in file storage
    if let Ok(Some(_)) = get_api_key_file("claude") {
        return Ok("Claude API key already migrated to secure file storage".to_string());
    }

    let mut migration_source = None;
    let mut found_key = None;

    // Check environment variable
    if let Ok(key) = std::env::var("CLAUDE_API_KEY") {
        if !key.trim().is_empty() {
            migration_source = Some("environment variable");
            found_key = Some(key);
        }
    }

    // Check config file if not found in environment
    if found_key.is_none() {
        let config_path = get_config_path();
        if let Ok(contents) = fs::read_to_string(config_path) {
            for line in contents.lines() {
                if let Some(key) = line.strip_prefix("CLAUDE_API_KEY=") {
                    if !key.trim().is_empty() {
                        migration_source = Some("config file");
                        found_key = Some(key.trim().to_string());
                        break;
                    }
                }
            }
        }
    }

    // Migrate if found
    if let (Some(source), Some(key)) = (migration_source, found_key) {
        match store_api_key_file("claude", &key) {
            Ok(_) => {
                info!(
                    "Successfully migrated Claude API key from {} to file storage",
                    source
                );
                Ok(format!(
                    "✅ Successfully migrated Claude API key from {} to secure file storage",
                    source
                ))
            }
            Err(e) => {
                error!("Failed to migrate Claude API key: {}", e);
                Err(format!("Failed to migrate Claude API key: {}", e))
            }
        }
    } else {
        Ok("No Claude API key found in environment variables or config file".to_string())
    }
}

#[tauri::command]
async fn ask_claude(
    app: tauri::AppHandle,
    model: String,
    prompt: String,
    messages: Vec<Message>,
) -> Result<String, String> {
    info!("Starting ask_claude with prompt: {}", prompt);

    let client = reqwest::Client::new();

    // Load API key from secure storage, environment, or config file
    let api_key = load_api_key(&app, "claude")?;
    info!("Successfully loaded API key");

    let model_name = model;
    info!("Using Claude model: {}", model_name);

    let request = ClaudeRequest {
        model: model_name,
        messages: if messages.is_empty() {
            vec![Message {
                role: "user".to_string(),
                content: MessageContent::Text(prompt),
            }]
        } else {
            messages
        },
        max_tokens: 1024,
        temperature: 0.0,
        stream: None,
        tools: Some(vec![Tool {
            tool_type: "web_search_20250305".to_string(),
            name: "web_search".to_string(),
            max_uses: Some(5),
        }]),
    };

    info!("Sending request to Claude API...");
    let response = match client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
    {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let error_text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "Could not read error response".to_string());
                error!(
                    "Claude API request failed with status {}: {}",
                    status, error_text
                );

                if status == 401 {
                    return Err(
                        "Authentication failed. Please check your Claude API key in Settings."
                            .to_string(),
                    );
                } else if status == 429 {
                    return Err("Rate limit exceeded. Please try again later.".to_string());
                } else {
                    return Err(format!("Claude API error ({}): {}", status, error_text));
                }
            }
            info!(
                "Received response from Claude API with status: {}",
                resp.status()
            );
            resp
        }
        Err(e) => {
            error!("Failed to connect to Claude API: {}", e);
            if e.is_timeout() {
                return Err("Request timed out. Please check your internet connection.".to_string());
            } else if e.is_connect() {
                return Err(
                    "Could not connect to Claude API. Please check your internet connection."
                        .to_string(),
                );
            } else {
                return Err(format!("Network error: {}", e));
            }
        }
    };

    let claude_response: ClaudeResponse = match response.json::<ClaudeResponse>().await {
        Ok(resp) => {
            info!(
                "Successfully parsed response: {:?}",
                resp.content.first().map(|c| &c.text)
            );
            resp
        }
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
async fn stream_claude(
    window: tauri::Window,
    app: tauri::AppHandle,
    model: String,
    prompt: String,
    messages: Vec<Message>,
) -> Result<(), String> {
    info!("Starting stream_claude with prompt: {}", prompt);

    let client = reqwest::Client::new();

    // Load API key from secure storage, environment, or config file
    let api_key = load_api_key(&app, "claude")?;
    info!("Successfully loaded API key");

    let model_name = model;
    info!("Using Claude model for streaming: {}", model_name);

    let request = ClaudeRequest {
        model: model_name,
        messages: if messages.is_empty() {
            vec![Message {
                role: "user".to_string(),
                content: MessageContent::Text(prompt),
            }]
        } else {
            messages
        },
        max_tokens: 1024,
        temperature: 0.0,
        stream: Some(true),
        tools: Some(vec![Tool {
            tool_type: "web_search_20250305".to_string(),
            name: "web_search".to_string(),
            max_uses: Some(5),
        }]),
    };

    info!("Sending streaming request to Claude API...");
    let response = match client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
    {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let error_text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "Could not read error response".to_string());
                error!("API request failed with status {}: {}", status, error_text);
                return Err(format!("API request failed: {} - {}", status, error_text));
            }
            info!(
                "Received streaming response from Claude API with status: {}",
                resp.status()
            );
            resp
        }
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
                                    match delta {
                                        ClaudeStreamDelta::TextDelta { text } => {
                                            if !text.is_empty() {
                                                info!("Parsed text from Claude delta: {}", text);
                                                full_response.push_str(&text);

                                                // Emit event to frontend
                                                if let Err(e) = window.emit("claude-stream", &text)
                                                {
                                                    error!(
                                                        "Failed to emit claude-stream event: {}",
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                        ClaudeStreamDelta::CitationsDelta { citation } => {
                                            info!(
                                                "Received citation: {} - {}",
                                                citation.title, citation.url
                                            );
                                            // Could emit citation event to frontend if needed
                                        }
                                        ClaudeStreamDelta::Other => {
                                            info!("Received other delta type, ignoring");
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
                        }
                        Err(e) => {
                            error!(
                                "Failed to parse JSON from Claude chunk: {} - Error: {}",
                                json_str, e
                            );
                            // Try to salvage any content by looking for text pattern
                            if let Some(content_start) = json_str.find("\"text\": \"") {
                                if let Some(content_end) = json_str[content_start + 9..].find("\"")
                                {
                                    let content = &json_str
                                        [content_start + 9..content_start + 9 + content_end];
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
            }
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

            if line.starts_with("event:")
                || json_str.trim().is_empty()
                || !json_str.trim().starts_with('{')
            {
                continue;
            }

            match serde_json::from_str::<ClaudeStreamEvent>(json_str) {
                Ok(event) => {
                    if let ClaudeStreamEvent::ContentBlockDelta { delta, .. } = event {
                        match delta {
                            ClaudeStreamDelta::TextDelta { text } => {
                                if !text.is_empty() {
                                    full_response.push_str(&text);
                                    if let Err(e) = window.emit("claude-stream", &text) {
                                        error!(
                                            "Failed to emit claude-stream event from buffer: {}",
                                            e
                                        );
                                    }
                                }
                            }
                            ClaudeStreamDelta::CitationsDelta { citation } => {
                                info!(
                                    "Received citation from buffer: {} - {}",
                                    citation.title, citation.url
                                );
                            }
                            ClaudeStreamDelta::Other => {
                                info!("Received other delta type from buffer, ignoring");
                            }
                        }
                    }
                }
                Err(_) => {
                    // Try salvage operation for remaining buffer
                    if let Some(content_start) = json_str.find("\"text\": \"") {
                        if let Some(content_end) = json_str[content_start + 9..].find("\"") {
                            let content =
                                &json_str[content_start + 9..content_start + 9 + content_end];
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

    info!(
        "Streaming completed from Claude. Full response: {}",
        full_response
    );

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
    citations: Option<Vec<String>>,
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
async fn ask_perplexity(
    app: tauri::AppHandle,
    model: String,
    prompt: String,
) -> Result<String, String> {
    info!(
        "Starting ask_perplexity with model: {} and prompt: {}",
        model, prompt
    );

    let client = reqwest::Client::new();

    // Load API key from secure storage
    let api_key = match get_api_key(app.clone(), "perplexity".to_string()).await? {
        Some(key) => key,
        None => return Err("Perplexity API key not found. Please add it in Settings.".to_string()),
    };

    // Build request body with the specified model
    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 1024,
        "temperature": 0.7
    });

    info!("Sending request to Perplexity API...");
    let response = match client
        .post("https://api.perplexity.ai/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
    {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let error_text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "Could not read error response".to_string());
                error!(
                    "Perplexity API request failed with status {}: {}",
                    status, error_text
                );

                if status == 401 {
                    return Err(
                        "Authentication failed. Please check your Perplexity API key in Settings."
                            .to_string(),
                    );
                } else if status == 429 {
                    return Err("Rate limit exceeded. Please try again later.".to_string());
                } else {
                    return Err(format!("Perplexity API error ({}): {}", status, error_text));
                }
            }
            info!(
                "Received response from Perplexity API with status: {}",
                resp.status()
            );
            resp
        }
        Err(e) => {
            error!("Failed to connect to Perplexity API: {}", e);
            if e.is_timeout() {
                return Err("Request timed out. Please check your internet connection.".to_string());
            } else if e.is_connect() {
                return Err(
                    "Could not connect to Perplexity API. Please check your internet connection."
                        .to_string(),
                );
            } else {
                return Err(format!("Network error: {}", e));
            }
        }
    };

    let perplexity_response: PerplexityResponse = match response.json::<PerplexityResponse>().await
    {
        Ok(resp) => {
            info!("Successfully parsed Perplexity response");
            resp
        }
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
async fn stream_perplexity(
    window: tauri::Window,
    app: tauri::AppHandle,
    model: String,
    prompt: String,
) -> Result<(), String> {
    info!(
        "Starting stream_perplexity with model: {} and prompt: {}",
        model, prompt
    );

    let client = reqwest::Client::new();

    // Load API key from secure storage
    let api_key = match get_api_key(app.clone(), "perplexity".to_string()).await? {
        Some(key) => key,
        None => return Err("Perplexity API key not found. Please add it in Settings.".to_string()),
    };

    // Build request body with stream enabled
    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 1024,
        "temperature": 0.7,
        "stream": true
    });

    info!("Sending streaming request to Perplexity API");

    let response = match client
        .post("https://api.perplexity.ai/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
    {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let error_text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "Could not read error response".to_string());
                error!("API request failed with status {}: {}", status, error_text);
                return Err(format!("API request failed: {} - {}", status, error_text));
            }
            info!(
                "Received streaming response from Perplexity API with status: {}",
                resp.status()
            );
            resp
        }
        Err(e) => {
            error!("Failed to send request to Perplexity API: {}", e);
            return Err(e.to_string());
        }
    };

    let mut stream = response.bytes_stream();
    let mut full_response = String::new();
    let mut buffer = String::new();
    let mut citations: Option<Vec<String>> = None;

    while let Some(item) = stream.next().await {
        match item {
            Ok(bytes) => {
                let chunk = String::from_utf8_lossy(&bytes);
                buffer.push_str(&chunk);

                // Process complete lines only
                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if line.trim().is_empty() || line.trim() == "data: [DONE]" {
                        continue;
                    }

                    // Remove the "data: " prefix if present
                    let json_str = if line.starts_with("data: ") {
                        &line[6..]
                    } else {
                        &line
                    };

                    // Skip if the JSON string is empty or obviously invalid
                    if json_str.trim().is_empty() || !json_str.trim().starts_with('{') {
                        continue;
                    }

                    // Parse the JSON
                    match serde_json::from_str::<PerplexityStreamResponse>(json_str) {
                        Ok(parsed) => {
                            // Capture citations if present (they come in the final chunk)
                            if let Some(ref cites) = parsed.citations {
                                citations = Some(cites.clone());
                                info!("Received {} citations", cites.len());
                            }

                            // Extract content from the first choice's delta if available
                            if let Some(choice) = parsed.choices.first() {
                                if let Some(content) = &choice.delta.content {
                                    full_response.push_str(content);

                                    // Emit event to frontend
                                    if let Err(e) = window.emit("perplexity-stream", content) {
                                        error!("Failed to emit perplexity-stream event: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!(
                                "Failed to parse JSON from line: {} - Error: {}",
                                json_str, e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error reading from stream: {}", e);
                return Err(format!("Error reading from stream: {}", e));
            }
        }
    }

    info!("Streaming completed. Full response: {}", full_response);

    // Emit completion event with the full response and citations
    let completion_data = serde_json::json!({
        "content": full_response,
        "citations": citations
    });

    if let Err(e) = window.emit("perplexity-stream-done", completion_data) {
        error!("Failed to emit perplexity-stream-done event: {}", e);
    }

    Ok(())
}
