# Contributing to Hyprnote

Thank you for your interest in contributing to Hyprnote! We welcome contributions from everyone, whether you're fixing a bug, adding a feature, improving documentation, or helping with community support.

## 🚀 Quick Start

### Prerequisites

Before you begin, ensure you have the following installed:

```bash
# Rust toolchain (required for Tauri and backend libs)
curl https://sh.rustup.rs -sSf | sh

# macOS-specific dependencies
brew install libomp       # Required for llama-cpp
brew install cmake        # Required for whisper-rs
xcode-select --install    # Required for cidre (audio capture)
xcodebuild -runFirstLaunch

# Node.js package manager and build tools
npm install -g pnpm turbo
```

### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/fastrepl/hyprnote.git
   cd hyprnote
   ```

2. **Install dependencies**
   ```bash
   pnpm install
   ```

3. **Start development server**
   ```bash
   turbo -F @hypr/desktop tauri:dev
   ```

## 🏗️ Project Structure

```
hyprnote/
├── apps/
│   ├── desktop/          # Main Tauri desktop application
│   ├── app/              # Web application
│   └── docs/             # Documentation site
├── crates/               # Rust libraries and utilities
├── plugins/              # Tauri plugins for extended functionality
└── packages/             # Shared TypeScript packages
```

### Key Directories

- **`apps/desktop/`** - Main desktop application (Tauri + React)
- **`crates/`** - Core Rust libraries (audio, AI models, etc.)
- **`plugins/`** - Tauri plugins for system integration
- **`apps/docs/`** - Documentation and guides

## 🐛 Reporting Issues

### Before Creating an Issue

1. **Search existing issues** to avoid duplicates
2. **Check the documentation** at [docs.hyprnote.com](https://docs.hyprnote.com)
3. **Gather system information** using our diagnostic script:

```bash
curl -s https://raw.githubusercontent.com/fastrepl/hyprnote/refs/heads/main/scripts/info.sh | bash
```

### Issue Template

When creating an issue, please include:

- **Hyprnote version** (from diagnostic script above)
- **Operating system** and version
- **Steps to reproduce** the problem
- **Expected behavior** vs actual behavior
- **Screenshots or logs** if applicable

## 💡 Contributing Code

### Development Workflow

1. **Fork** the repository
2. **Create a feature branch** from `main`
3. **Make your changes** following our coding standards
4. **Test thoroughly** on your local environment
5. **Submit a pull request** with clear description

### Branch Naming

Use descriptive branch names:
- `feature/audio-enhancement` - New features
- `fix/transcription-bug` - Bug fixes
- `docs/setup-guide` - Documentation updates
- `refactor/plugin-system` - Code refactoring

### Commit Messages

Follow conventional commit format:
```
type(scope): brief description

More detailed explanation if needed
```

Examples:
- `feat(audio): add noise reduction filter`
- `fix(ui): resolve settings panel layout issue`
- `docs(readme): update installation instructions`

## 🧪 Testing

### Running Tests

```bash
# Run all tests
turbo test

# Run specific package tests
turbo -F @hypr/desktop test

# Run Rust tests
cargo test
```

### Manual Testing

Before submitting a PR, please test:
- Core recording functionality
- AI transcription and enhancement
- Settings and preferences
- Plugin integrations (if applicable)

## 🎨 Code Style

### Formatting

We use [dprint](https://dprint.dev/) for consistent code formatting:

```bash
# Format all code
dprint fmt

# Check formatting
dprint check
```

### Rust Guidelines

- Follow standard Rust conventions
- Use `cargo clippy` for linting
- Document public APIs with rustdoc comments
- Prefer explicit error handling over panics

### TypeScript Guidelines

- Use TypeScript strict mode
- Prefer functional components with hooks
- Follow existing naming conventions
- Use descriptive variable and function names

## 🏗️ Architecture Guidelines

### Core Principles

1. **Privacy First** - No data should leave the user's device
2. **Offline Capable** - Core functionality must work without internet
3. **Plugin Architecture** - New features should be pluggable when possible
4. **Performance** - Optimize for low latency and resource usage

### Adding New Features

When adding features:
1. Consider if it should be a plugin
2. Ensure offline functionality
3. Maintain privacy guarantees
4. Add appropriate tests and documentation

## 🔧 Platform-Specific Notes

### macOS Development

- Ensure proper entitlements in `src-tauri/Entitlements.plist`
- Test on both Apple Silicon and Intel Macs
- Verify microphone and calendar permissions work correctly

### Cross-Platform Considerations

- Use platform-agnostic Rust crates when possible
- Abstract platform-specific code into separate modules
- Test on all supported platforms before merging

## 📖 Documentation

### What to Document

- New features and APIs
- Configuration options
- Plugin development guides
- Breaking changes

### Documentation Style

- Write clear, concise instructions
- Include code examples
- Use screenshots for UI changes
- Test all code examples

## 🚨 Common Issues & Solutions

### Build Failures

**Architecture/OS Problems**
```bash
# Specify your architecture explicitly
CARGO_BUILD_TARGET=aarch64-apple-darwin pnpm exec turbo -F @hypr/desktop tauri:dev
```

**Supported targets:**
- macOS Apple Silicon: `aarch64-apple-darwin`
- macOS Intel: `x86_64-apple-darwin`
- Windows: `x86_64-pc-windows-msvc`

**macOS Version Warnings**
If you see version compatibility warnings, you can update these files locally (don't commit):
- `crates/tcc/build.rs`
- `apps/desktop/src-tauri/tauri.conf.json`
- `crates/tcc/swift-lib/Package.swift`

### Performance Issues

- Profile with `cargo flamegraph` for Rust code
- Use browser dev tools for frontend performance
- Monitor memory usage during long recordings

## 🤝 Community

### Getting Help

- **Discord**: [Join our community](https://hyprnote.com/discord)
- **GitHub Discussions**: For feature requests and general questions
- **GitHub Issues**: For bug reports and specific problems

### Code of Conduct

Be respectful, inclusive, and constructive in all interactions. We're building software to help people be more productive, and our community should reflect those values.

## 🎯 Contribution Ideas

Looking for ways to contribute? Here are some areas where we'd love help:

### High Priority
- Windows and Linux platform support
- Performance optimizations
- Accessibility improvements
- Plugin development framework

### Medium Priority
- UI/UX enhancements
- Additional language support
- Integration with more calendar providers
- Documentation improvements

### Getting Started
- Fix typos in documentation
- Add missing type annotations
- Improve error messages
- Write unit tests

## 📋 Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

1. Update version numbers
2. Update CHANGELOG.md
3. Test on all supported platforms
4. Create GitHub release with detailed notes
5. Update documentation if needed

## 🙏 Recognition

Contributors are recognized in:
- GitHub contributor graph
- Release notes for significant contributions
- Special mentions in Discord announcements

## 📞 Contact

- **Maintainers**: [GitHub team page](https://github.com/orgs/fastrepl/teams)
- **General Questions**: [Discord community](https://hyprnote.com/discord)
- **Security Issues**: security@hyprnote.com

---

Thank you for contributing to Hyprnote! Every contribution, no matter how small, helps make meeting notes better for everyone.