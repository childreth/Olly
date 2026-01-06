# Security Audit Report

**Date:** 2025-05-23
**Auditor:** Jules (AI Agent)
**Target:** Olly (Tauri + Svelte + Rust)

## Executive Summary

A security review of the Olly application identified **2 Critical** and **1 High** severity vulnerabilities. The most significant risk is a Remote Code Execution (RCE) vulnerability stemming from unsanitized Markdown rendering combined with a missing Content Security Policy (CSP).

## Vulnerability Findings

### 1. Cross-Site Scripting (XSS) leading to RCE (Critical)

**Location:** `src/routes/+page.svelte`

**Description:**
The application uses the `marked` library to parse Markdown responses from the LLM and renders them using Svelte's `{@html ...}` tag without any sanitization.
```javascript
// src/routes/+page.svelte
responseMarked = marked.parse(streamedGreeting);
// ...
{@html responseMarked}
```
`marked` does not sanitize HTML by default. If an LLM is tricked (via prompt injection) or malfunctions and outputs malicious HTML/JavaScript (e.g., `<img src=x onerror=alert(1)>`), the script will execute in the context of the application.

**Impact:**
In a Tauri application, the frontend has access to the `window.__TAURI__` object. An attacker can use XSS to invoke Rust backend commands, read/write files, or execute shell commands (if enabled), leading to full system compromise.

**Remediation:**
1.  Install `dompurify` and `isomorphic-dompurify`.
2.  Sanitize the HTML before rendering:
    ```javascript
    import DOMPurify from 'isomorphic-dompurify';
    // ...
    responseMarked = DOMPurify.sanitize(marked.parse(streamedGreeting));
    ```

### 2. Missing Content Security Policy (CSP) (Critical)

**Location:** `src-tauri/tauri.conf.json`

**Description:**
The `security.csp` configuration is explicitly set to `null`.
```json
"security": {
  "csp": null
}
```

**Impact:**
This disables the browser's security mechanism that restricts the sources from which content (scripts, styles, images) can be loaded. It makes the XSS vulnerability described above trivial to exploit and prevents defense-in-depth.

**Remediation:**
Configure a strict CSP in `tauri.conf.json`. For example:
```json
"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: https://* data: blob:; connect-src 'self' http://localhost:11434 https://api.anthropic.com https://api.perplexity.ai https://api.openai.com https://api.weather.gov https://nominatim.openstreetmap.org;"
```

### 3. Path Traversal in API Key Storage (High)

**Location:** `src-tauri/src/main.rs` (`get_api_key_file`, `store_api_key_file`)

**Description:**
The application constructs file paths using the `provider` string input directly from the frontend without validation.
```rust
let key_file = keys_dir.join(format!("{}.key", provider));
```
While `PathBuf::join` handles some cases, if `provider` contains `..` or starts with `/`, it might allow accessing files outside the intended directory.

**Impact:**
An attacker (or compromised frontend) could write arbitrary files to the user's system or overwrite critical files by manipulating the `provider` argument.

**Remediation:**
Validate the `provider` string to ensure it contains only alphanumeric characters and no path separators before using it.

### 4. Weak Encryption for API Keys (Medium)

**Location:** `src-tauri/src/main.rs` (`simple_encode`)

**Description:**
API keys are obfuscated using a simple XOR with a hardcoded static key (`olly_secure_2024`).
```rust
let key = b"olly_secure_2024";
```

**Impact:**
This offers no real cryptographic protection. Anyone with access to the file system (or the source code) can trivially recover the keys. It protects only against casual observation.

**Remediation:**
Use the OS native keyring (which was commented out due to issues) or use a proper encryption library (like `age` or `ring`) with a key derived from a user password or a machine-specific secret.

### 5. Overly Permissive Network Scope (Low)

**Location:** `src-tauri/capabilities/migrated.json`

**Description:**
The capability allowlist includes wildcards for all HTTP/HTTPS traffic:
```json
{ "url": "http://**/" },
{ "url": "https://**/" }
```

**Impact:**
This allows the frontend to contact any server. While necessary for some features (like browsing), it increases the attack surface if the frontend is compromised.

**Remediation:**
Restrict the scope to known API endpoints if possible, or maintain this risk if the feature set requires "browsing" capabilities.

## Conclusion

The application currently has critical security flaws that should be addressed immediately before any distribution or extensive use, particularly the XSS/RCE vulnerability.
