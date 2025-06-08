# Plugin Development

Hyprnote's extensible plugin architecture allows you to add custom functionality to the application. This guide covers creating, developing, and distributing plugins for both desktop and web environments.

## 🏗️ Plugin Architecture

Hyprnote supports two types of plugins:

### Tauri Plugins (Desktop)
- **Native system integration** - Access OS-level APIs
- **Rust backend** with TypeScript frontend
- **Secure permissions model** - Granular capability control
- **Cross-platform support** - macOS, Windows, Linux

### Web Plugins (Browser)
- **JavaScript/TypeScript only** - No native system access
- **Lightweight integration** - Pure web technologies
- **Browser compatibility** - Works in any modern browser
- **Rapid development** - No compilation required

## 🚀 Quick Start

### Creating a Tauri Plugin

#### 1. Generate Plugin Structure
```bash
# Create a new Tauri plugin
npx @tauri-apps/cli plugin new my-plugin \
  --no-example \
  --directory ./plugins/my-plugin

# Navigate to plugin directory
cd plugins/my-plugin
```

#### 2. Plugin Structure
```
plugins/my-plugin/
├── Cargo.toml              # Rust dependencies and metadata
├── build.rs                # Build script for TypeScript generation
├── src/
│   ├── lib.rs             # Main Rust plugin code
│   ├── commands.rs        # Tauri commands (API endpoints)
│   ├── ext.rs             # Plugin extension and initialization
│   └── error.rs           # Error handling
├── js/
│   ├── index.ts           # TypeScript API bindings
│   └── bindings.gen.ts    # Auto-generated TypeScript types
├── permissions/
│   ├── default.toml       # Default permissions
│   └── schemas/           # Permission schemas
└── package.json           # JavaScript package metadata
```

#### 3. Development Workflow
```bash
# Install dependencies
pnpm install

# Build TypeScript bindings
pnpm build

# Test the plugin
cargo test

# Format code
dprint fmt
```

## 🔧 Plugin Development

### Backend Development (Rust)

#### Basic Plugin Structure
```rust
// src/lib.rs
use tauri::{AppHandle, command, generate_handler, plugin::{Builder, TauriPlugin}, Runtime};

#[command]
async fn my_command() -> Result<String, String> {
    Ok("Hello from my plugin!".to_string())
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("my-plugin")
        .invoke_handler(generate_handler![my_command])
        .build()
}
```

#### Adding Commands
```rust
// src/commands.rs
use tauri::command;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MyRequest {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct MyResponse {
    pub result: String,
    pub timestamp: i64,
}

#[command]
pub async fn process_data(request: MyRequest) -> Result<MyResponse, String> {
    // Process the request
    let result = format!("Processed: {}", request.message);
    
    Ok(MyResponse {
        result,
        timestamp: chrono::Utc::now().timestamp(),
    })
}

#[command]
pub async fn get_system_info() -> Result<String, String> {
    // Access system information
    let info = format!("OS: {}", std::env::consts::OS);
    Ok(info)
}
```

#### Error Handling
```rust
// src/error.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum PluginError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Permission denied: {0}")]
    Permission(String),
}

impl From<PluginError> for tauri::Error {
    fn from(err: PluginError) -> Self {
        tauri::Error::PluginInvocation(err.to_string())
    }
}
```

#### Plugin State Management
```rust
// src/ext.rs
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

pub struct PluginState {
    pub config: Mutex<PluginConfig>,
    pub cache: Mutex<HashMap<String, String>>,
}

impl PluginState {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(PluginConfig::default()),
            cache: Mutex::new(HashMap::new()),
        }
    }
}

#[command]
pub async fn set_config(
    state: State<'_, PluginState>,
    config: PluginConfig,
) -> Result<(), String> {
    let mut state_config = state.config.lock().unwrap();
    *state_config = config;
    Ok(())
}
```

### Frontend Development (TypeScript)

#### Generated API Bindings
```typescript
// js/index.ts
import { invoke } from '@tauri-apps/api/core'

export interface MyRequest {
  message: string
}

export interface MyResponse {
  result: string
  timestamp: number
}

// Auto-generated from Rust commands
export async function processData(request: MyRequest): Promise<MyResponse> {
  return await invoke('plugin:my-plugin|process_data', { request })
}

export async function getSystemInfo(): Promise<string> {
  return await invoke('plugin:my-plugin|get_system_info')
}
```

#### React Hook Integration
```typescript
// js/hooks.ts
import { useCallback, useEffect, useState } from 'react'
import { processData, MyRequest, MyResponse } from './index'

export function useMyPlugin() {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const processMessage = useCallback(async (message: string) => {
    setLoading(true)
    setError(null)
    
    try {
      const response = await processData({ message })
      return response
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error')
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  return {
    processMessage,
    loading,
    error,
  }
}
```

### Permissions and Security

#### Permission Configuration
```toml
# permissions/default.toml
[[permission]]
identifier = "allow-process-data"
description = "Allow processing user data"

[[permission]]
identifier = "allow-system-info"
description = "Allow reading system information"

[permission.commands]
allow = ["process_data"]
deny = []

[set.core]
description = "Core plugin functionality"
permissions = ["allow-process-data", "allow-system-info"]
```

#### Permission Schema
```json
{
  "$schema": "https://tauri.app/schemas/permission-schema.json",
  "identifier": "my-plugin",
  "description": "Permissions for my custom plugin",
  "permissions": [
    {
      "identifier": "allow-process-data",
      "description": "Allow the plugin to process user data"
    }
  ]
}
```

## 🔌 Plugin Integration

### Registering Your Plugin

#### In Desktop App
```rust
// In apps/desktop/src-tauri/src/lib.rs
fn main() {
    tauri::Builder::default()
        .plugin(my_plugin::init())
        .plugin(other_plugin::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### In Frontend
```typescript
// In apps/desktop/src/main.tsx
import { useMyPlugin } from '@hypr/plugins/my-plugin'

function MyComponent() {
  const { processMessage, loading, error } = useMyPlugin()
  
  const handleClick = async () => {
    try {
      const result = await processMessage("Hello, plugin!")
      console.log('Result:', result)
    } catch (err) {
      console.error('Plugin error:', err)
    }
  }
  
  return (
    <button onClick={handleClick} disabled={loading}>
      {loading ? 'Processing...' : 'Process Message'}
    </button>
  )
}
```

## 📦 Plugin Examples

### File System Plugin
```rust
// Example: File operations plugin
#[command]
pub async fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(path, content)
        .map_err(|e| e.to_string())
}
```

### HTTP Client Plugin
```rust
// Example: HTTP requests plugin
#[command]
pub async fn fetch_data(url: String) -> Result<String, String> {
    let response = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?;
    
    response.text()
        .await
        .map_err(|e| e.to_string())
}
```

### Database Plugin
```rust
// Example: Database operations plugin
use sqlx::SqlitePool;

#[command]
pub async fn query_database(
    pool: State<'_, SqlitePool>,
    query: String,
) -> Result<Vec<serde_json::Value>, String> {
    let rows = sqlx::query(&query)
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    
    // Convert rows to JSON
    Ok(rows.into_iter().map(|row| {
        // Convert row to JSON
        serde_json::Value::Object(serde_json::Map::new())
    }).collect())
}
```

## 🧪 Testing Plugins

### Unit Testing
```rust
// tests/plugin_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_process_data() {
        let request = MyRequest {
            message: "test".to_string(),
        };
        
        let result = process_data(request).await.unwrap();
        assert_eq!(result.result, "Processed: test");
        assert!(result.timestamp > 0);
    }
    
    #[test]
    fn test_plugin_initialization() {
        let plugin = init::<tauri::Wry>();
        assert_eq!(plugin.name(), "my-plugin");
    }
}
```

### Integration Testing
```typescript
// tests/integration.test.ts
import { describe, it, expect } from 'vitest'
import { processData } from '../js/index'

describe('My Plugin Integration', () => {
  it('should process data correctly', async () => {
    const result = await processData({ message: 'test' })
    
    expect(result.result).toBe('Processed: test')
    expect(result.timestamp).toBeGreaterThan(0)
  })
})
```

## 📚 Advanced Topics

### Plugin Communication
```rust
// Plugin-to-plugin communication
#[command]
pub async fn call_other_plugin(
    app: AppHandle,
    data: String,
) -> Result<String, String> {
    app.emit("plugin-message", data)
        .map_err(|e| e.to_string())?;
    
    Ok("Message sent".to_string())
}
```

### Background Tasks
```rust
// Long-running background tasks
use std::sync::Arc;
use tokio::sync::Mutex;

#[command]
pub async fn start_background_task(
    app: AppHandle,
) -> Result<String, String> {
    let app = Arc::new(app);
    
    tokio::spawn(async move {
        loop {
            // Perform background work
            tokio::time::sleep(Duration::from_secs(10)).await;
            
            // Emit progress updates
            let _ = app.emit("task-progress", "Still working...");
        }
    });
    
    Ok("Background task started".to_string())
}
```

### Plugin Configuration
```rust
// Dynamic plugin configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginConfig {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub timeout: u64,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_key: None,
            timeout: 30,
        }
    }
}
```

## 🚀 Publishing Plugins

### Plugin Metadata
```toml
# Cargo.toml
[package]
name = "hyprnote-my-plugin"
version = "0.1.0"
description = "My awesome Hyprnote plugin"
authors = ["Your Name <your.email@example.com>"]
license = "MIT"
repository = "https://github.com/yourusername/hyprnote-my-plugin"

[package.metadata.hyprnote]
min_version = "0.1.0"
category = "productivity"
tags = ["automation", "integration"]
```

### Distribution
```bash
# Publish to crates.io
cargo publish

# Create GitHub release
gh release create v0.1.0 \
  --title "My Plugin v0.1.0" \
  --notes "Initial release"

# Submit to Hyprnote plugin registry
# (Documentation coming soon)
```

## 🛠️ Development Tools

### VS Code Configuration
```json
// .vscode/settings.json
{
  "rust-analyzer.cargo.features": ["tauri/api"],
  "rust-analyzer.linkedProjects": ["Cargo.toml"],
  "typescript.preferences.includePackageJsonAutoImports": "on"
}
```

### Debugging
```bash
# Debug Rust plugin code
RUST_LOG=debug cargo test -- --nocapture

# Debug TypeScript integration
npm run dev -- --debug
```

## 📋 Best Practices

### Code Quality
1. **Use descriptive command names** - `getUserPreferences` not `getPrefs`
2. **Handle errors gracefully** - Always return meaningful error messages
3. **Validate input parameters** - Check types and ranges
4. **Document your APIs** - Use rustdoc comments and TypeScript types
5. **Follow security principles** - Minimize permission requirements

### Performance
1. **Use async operations** - Don't block the UI thread
2. **Cache frequently accessed data** - Avoid repeated expensive operations
3. **Optimize bundle size** - Only include necessary dependencies
4. **Profile memory usage** - Watch for memory leaks in long-running plugins

### User Experience
1. **Provide loading states** - Show progress for long operations
2. **Graceful degradation** - Handle plugin unavailability
3. **Clear error messages** - Help users understand what went wrong
4. **Consistent UI patterns** - Follow Hyprnote design guidelines

## 🆘 Getting Help

### Documentation
- [Tauri Plugin Guide](https://v2.tauri.app/develop/plugins/) - Official Tauri documentation
- [Rust API Reference](https://docs.rs/tauri/) - Detailed API documentation
- [TypeScript Bindings](https://tauri.app/v1/api/js/) - Frontend integration guide

### Community
- [Discord #plugin-development](https://hyprnote.com/discord) - Get help from the community
- [GitHub Discussions](https://github.com/fastrepl/hyprnote/discussions) - Ask questions and share ideas
- [Plugin Examples Repository](https://github.com/fastrepl/hyprnote-plugins) - Sample plugins

### Troubleshooting
Common issues and solutions:

**Build Errors**
```bash
# Clear cache and rebuild
cargo clean
rm -rf node_modules
pnpm install
pnpm build
```

**Permission Denied**
```toml
# Check permissions/default.toml
[[permission]]
identifier = "allow-my-command"
```

**TypeScript Errors**
```bash
# Regenerate bindings
pnpm build
```

---

**Ready to build your first plugin?** Start with the quick start guide and join our [Discord](https://hyprnote.com/discord) for support! 🚀
