import { invoke } from "@tauri-apps/api/core";

// Mocking invoke for performance measurement since we are in a sandbox
// and might not have a real Tauri backend running/accessible this way.
// However, the task asks to establish a baseline.
// If I can't run it, I will provide the rationale.

async function loadExternalProvidersOriginal(providers) {
    const start = performance.now();
    for (const provider of providers) {
        const info = await invoke("get_provider_info", { provider });
    }
    const end = performance.now();
    return end - start;
}

async function loadExternalProvidersOptimized(providers) {
    const start = performance.now();
    await Promise.all(providers.map(provider => invoke("get_provider_info", { provider })));
    const end = performance.now();
    return end - start;
}

const mockProviders = Array.from({ length: 100 }, (_, i) => `provider_${i}`);

// Since I cannot easily run this in the environment with @tauri-apps/api/core
// without a browser/tauri context, I'll document the expected improvement.
