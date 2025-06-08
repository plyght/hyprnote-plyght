![README banner](https://github.com/user-attachments/assets/44ebe793-004e-4313-8e04-2491ab076bea)

<div align="center">

# Hyprnote *(Public Beta)*

**AI-powered meeting notes that work offline**

[![Discord](https://img.shields.io/static/v1?label=Join%20our&message=Discord&color=blue&logo=Discord)](https://hyprnote.com/discord)
[![X (formerly Twitter)](https://img.shields.io/static/v1?label=Follow%20us%20on&message=X&color=black&logo=x)](https://x.com/tryhyprnote)
[![License](https://img.shields.io/github/license/fastrepl/hyprnote)](./LICENSE)
[![Release](https://img.shields.io/github/v/release/fastrepl/hyprnote)](https://github.com/fastrepl/hyprnote/releases)

[Download](https://hyprnote.com/download) • [Documentation](https://docs.hyprnote.com) • [Discord](https://hyprnote.com/discord) • [Website](https://hyprnote.com)

</div>

## 🎯 What is Hyprnote?

Hyprnote is a local-first AI meeting assistant that transforms your conversations into structured, actionable notes. Unlike cloud-based alternatives, Hyprnote runs entirely on your device, ensuring complete privacy and offline functionality.

### ✨ Key Features

- **🎙️ Real-time transcription** - Accurate speech-to-text using Whisper
- **🤖 AI-powered summaries** - Generate structured notes with Llama models
- **🔒 Privacy-first** - All processing happens locally on your device
- **📱 Cross-platform** - Native desktop applications for macOS (Windows/Linux coming soon)
- **🔌 Extensible** - Plugin architecture for custom integrations
- **📅 Calendar integration** - Sync with Google Calendar, Outlook, and Apple Calendar
- **🌐 Works offline** - No internet connection required for core functionality

## 📦 Quick Start

### macOS Installation

```bash
brew tap fastrepl/hyprnote && brew install hyprnote --cask
```

**Alternative download options:**
- [Direct download](https://hyprnote.com/download) - Latest stable release
- [GitHub releases](https://github.com/fastrepl/hyprnote/releases) - All versions

### Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| **macOS** | ✅ Available | Public beta, Apple Silicon & Intel |
| **Windows** | 🚧 Coming soon | [Track progress](https://github.com/fastrepl/hyprnote/issues/66) |
| **Linux** | 🤔 Maybe | [Community interest](https://github.com/fastrepl/hyprnote/issues/67) |

## 🚀 Features Overview

### Intelligent Note Enhancement
Transform casual meeting notes into professional summaries with AI assistance.

<img src="https://github.com/user-attachments/assets/1615a9f0-7a30-44c1-b142-0d1774a84e89" width="720" alt="Note enhancement demo" />

### Complete Privacy & Offline Operation
Your conversations never leave your device. Work confidently in sensitive environments.

<img src="https://github.com/user-attachments/assets/e5014024-3f6a-457a-8f1c-3b183883b782" width="720" alt="Privacy and offline features" />

## 🏗️ Architecture

Hyprnote is built with a modern, extensible architecture:

- **Desktop App**: Tauri-based native application (Rust + TypeScript)
- **Backend Services**: Rust microservices for AI processing and data management
- **Plugin System**: Extensible architecture for custom integrations
- **Local AI Models**: Whisper for speech recognition, Llama for text generation
- **Storage**: Local SQLite databases with optional cloud sync

## 🤝 Contributing

We welcome contributions! See our [Contributing Guide](./CONTRIBUTING.md) for details on:

- Setting up the development environment
- Code style and formatting guidelines
- Submitting pull requests
- Reporting issues

## 📄 License

This project is licensed under the terms specified in the [LICENSE](./LICENSE) file.

## 🔒 Privacy

Your privacy is our priority. Read our [Privacy Policy](./PRIVACY.md) to understand how we handle your data (spoiler: we don't collect it).

## 🆘 Support

- **Documentation**: [docs.hyprnote.com](https://docs.hyprnote.com)
- **Discord Community**: [Join our Discord](https://hyprnote.com/discord)
- **Issues**: [GitHub Issues](https://github.com/fastrepl/hyprnote/issues)
- **Feature Requests**: [GitHub Discussions](https://github.com/fastrepl/hyprnote/discussions)

## ⭐ Star History

If you find Hyprnote useful, please consider giving it a star! It helps us understand the community interest and motivates continued development.

[![Star History Chart](https://api.star-history.com/svg?repos=fastrepl/hyprnote&type=Date)](https://star-history.com/#fastrepl/hyprnote&Date)

---

<div align="center">

Made with ❤️ by the Hyprnote team

</div>