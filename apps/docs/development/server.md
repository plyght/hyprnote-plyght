# Server Development

This guide covers developing the Hyprnote server components, including both the backend services and web frontend.

## 🏗️ Server Architecture

Hyprnote's server architecture consists of multiple components:

### Backend Services
- **Main Server** (`apps/app/server/`) - Core API server built with Rust
- **Restate Services** (`apps/restate/`) - Durable workflow services
- **Native Libraries** (`crates/`) - Core functionality (audio, AI, etc.)

### Frontend Applications  
- **Web App** (`apps/app/`) - Browser-based interface
- **Desktop App** (`apps/desktop/`) - Tauri-based native application

## 📋 Prerequisites

Before starting server development, ensure you have these tools installed:

### Required Tools
```bash
# Task runner for build automation
brew install go-task/tap/go-task

# Bacon for Rust hot reloading
cargo install bacon

# pnpm for JavaScript package management
npm install -g pnpm

# Turbo for monorepo management
npm install -g turbo
```

### Rust Dependencies
```bash
# Rust toolchain
curl https://sh.rustup.rs -sSf | sh

# macOS-specific dependencies for audio/AI
brew install libomp cmake
xcode-select --install
```

### Database Setup
Hyprnote uses SQLite for local storage and Turso for cloud sync:

```bash
# Install Turso CLI (optional, for cloud development)
curl -sSfL https://get.tur.so/install.sh | bash
```

## 🚀 Quick Start

### 1. Clone and Install
```bash
git clone https://github.com/fastrepl/hyprnote.git
cd hyprnote
pnpm install
```

### 2. Development Database Setup
```bash
# Initialize local SQLite databases
turbo -F @hypr/db-user db:migrate
turbo -F @hypr/db-admin db:migrate
```

### 3. Start Development Servers

#### Backend Development
```bash
# Start the main API server with hot reloading
task bacon app-backend

# Alternative: Run without hot reloading
turbo -F @hypr/app dev:server
```

#### Frontend Development
```bash
# Start the web frontend
task bacon app-frontend

# Alternative: Standard development server
turbo -F @hypr/app dev
```

#### Full Stack Development
```bash
# Run both backend and frontend concurrently
turbo -F @hypr/app dev:full
```

## 🔧 Development Workflow

### File Structure
```
apps/app/
├── server/               # Rust backend server
│   ├── src/
│   │   ├── main.rs      # Server entry point
│   │   ├── middleware.rs # Auth, CORS, etc.
│   │   ├── state.rs     # Application state
│   │   ├── native/      # Native API routes
│   │   └── web/         # Web API routes
│   └── Cargo.toml
├── src/                 # Frontend React app
│   ├── components/
│   ├── routes/
│   └── types/
├── package.json
└── vite.config.ts
```

### Backend Development

#### Adding New API Endpoints
1. **Define route in appropriate module** (`native/` or `web/`)
2. **Add route to main server** in `main.rs`
3. **Update OpenAPI schema** with proper types
4. **Generate TypeScript types** for frontend

Example new endpoint:
```rust
// In server/src/native/example.rs
use axum::{Json, extract::State};
use crate::state::AppState;

pub async fn get_example(
    State(state): State<AppState>,
) -> Result<Json<ExampleResponse>, AppError> {
    // Implementation here
    Ok(Json(ExampleResponse { message: "Hello".to_string() }))
}
```

#### Working with Database
```bash
# Create new migration
echo "CREATE TABLE example (id INTEGER PRIMARY KEY);" > crates/db-user/src/example_migration.sql

# Run migrations
turbo -F @hypr/db-user db:migrate

# Reset database (caution: deletes all data)
turbo -F @hypr/db-user db:reset
```

### Frontend Development

#### API Integration
The frontend automatically generates TypeScript types from the OpenAPI schema:

```bash
# Regenerate types after backend changes
turbo -F @hypr/app codegen
```

#### Component Development
```typescript
// Example API integration
import { client } from '@/client'

export function ExampleComponent() {
  const { data, error } = useSWR('/api/example', () => 
    client.GET('/api/example')
  )
  
  if (error) return <div>Error loading example</div>
  if (!data) return <div>Loading...</div>
  
  return <div>{data.message}</div>
}
```

## 🌐 API Development

### OpenAPI Schema
Hyprnote uses OpenAPI for API documentation and type generation:

```bash
# Generate OpenAPI schema
turbo -F @hypr/app openapi:generate

# View API documentation
open http://localhost:3000/docs
```

### Authentication
The server supports multiple authentication methods:

- **OAuth** (Google, Microsoft) for web users
- **API Keys** for programmatic access
- **Session-based** for web sessions

### CORS Configuration
CORS is configured in `server/src/middleware.rs`:

```rust
// Development: Allow all origins
let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_headers(Any)
    .allow_methods(Any);

// Production: Restrict to known origins
let cors = CorsLayer::new()
    .allow_origin("https://app.hyprnote.com".parse::<HeaderValue>().unwrap())
    .allow_headers([CONTENT_TYPE, AUTHORIZATION])
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]);
```

## 🧪 Testing

### Backend Testing
```bash
# Run all Rust tests
cargo test

# Run specific crate tests
cargo test -p db-user

# Run with output
cargo test -- --nocapture
```

### Frontend Testing
```bash
# Run React component tests
turbo -F @hypr/app test

# Run E2E tests (if configured)
turbo -F @hypr/app test:e2e
```

### Integration Testing
```bash
# Test full API integration
curl -X GET http://localhost:3000/api/health
curl -X POST http://localhost:3000/api/sessions \
  -H "Content-Type: application/json" \
  -d '{"title": "Test Session"}'
```

## 📊 Monitoring & Debugging

### Development Logging
```bash
# Enable debug logging
RUST_LOG=debug task bacon app-backend

# Enable trace logging for specific module
RUST_LOG=hyprnote_server::native=trace task bacon app-backend
```

### Performance Profiling
```bash
# Profile server performance
cargo install flamegraph
cargo flamegraph --bin hyprnote-server

# Profile frontend performance
# Use browser DevTools for React profiling
```

### Database Inspection
```bash
# Open SQLite database
sqlite3 ~/.hyprnote/user.db
.tables
.schema sessions
SELECT * FROM sessions LIMIT 10;
```

## 🔌 Plugin Development

Hyprnote's server can be extended with plugins for additional functionality:

### Creating a Server Plugin
1. **Add new crate** in `crates/my-plugin/`
2. **Implement plugin trait** for server integration
3. **Register plugin** in main server configuration
4. **Add API routes** if needed

Example plugin structure:
```rust
// crates/my-plugin/src/lib.rs
pub struct MyPlugin {
    config: MyPluginConfig,
}

impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }
    
    fn initialize(&mut self) -> Result<(), PluginError> {
        // Plugin initialization
        Ok(())
    }
    
    fn routes(&self) -> Vec<Route> {
        // Return API routes
        vec![]
    }
}
```

## 🌍 Environment Configuration

### Development Environment
```bash
# .env.development
DATABASE_URL=sqlite://~/.hyprnote/dev.db
LOG_LEVEL=debug
CORS_ORIGINS=*
```

### Production Environment
```bash
# .env.production
DATABASE_URL=libsql://your-turso-db.turso.io
LOG_LEVEL=info
CORS_ORIGINS=https://app.hyprnote.com
```

### Environment Variables
| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | Database connection string | SQLite local |
| `LOG_LEVEL` | Logging verbosity | `info` |
| `PORT` | Server port | `3000` |
| `CORS_ORIGINS` | Allowed CORS origins | `*` (dev) |

## 🚨 Common Issues

### Port Already in Use
```bash
# Find process using port 3000
lsof -ti:3000 | xargs kill -9

# Or use a different port
PORT=3001 task bacon app-backend
```

### Database Migration Errors
```bash
# Reset database and re-run migrations
rm ~/.hyprnote/user.db
turbo -F @hypr/db-user db:migrate
```

### Rust Compilation Issues
```bash
# Clean cargo cache
cargo clean

# Update Rust toolchain
rustup update

# Rebuild with clean cache
cargo build
```

### Frontend Build Issues
```bash
# Clear node_modules and reinstall
rm -rf node_modules
pnpm install

# Clear Vite cache
rm -rf node_modules/.vite
```

## 📚 Additional Resources

### Documentation
- [Axum Web Framework](https://docs.rs/axum) - Backend framework
- [Vite](https://vitejs.dev/) - Frontend build tool
- [Tauri](https://tauri.app/) - Desktop app framework
- [Turso](https://turso.tech/) - Database platform

### Related Guides
- [Plugin Development](./plugin.md) - Creating custom plugins
- [Contributing](./contributing.md) - Contributing guidelines
- [Mobile Development](./mobile.md) - iOS/Android development

### Getting Help
- [Discord Community](https://hyprnote.com/discord) - Community support
- [GitHub Issues](https://github.com/fastrepl/hyprnote/issues) - Bug reports
- [GitHub Discussions](https://github.com/fastrepl/hyprnote/discussions) - Feature requests

---

**Ready to start server development?** Follow the quick start guide above and join our [Discord](https://hyprnote.com/discord) if you need help! 🚀
