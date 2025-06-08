# Privacy Policy

*Last updated: December 2024*

## Our Privacy Promise

**TL;DR: We don't collect, store, or transmit your meeting data. Everything stays on your device.**

Hyprnote is built with privacy as a core principle. Unlike cloud-based meeting tools, your conversations, transcriptions, and notes never leave your device.

## 🔐 Data Processing

### What Stays Local
- **Meeting recordings** - Processed locally using Whisper
- **Transcriptions** - Generated and stored only on your device
- **AI-generated summaries** - Created using local Llama models
- **Meeting notes and annotations** - Stored in local SQLite databases
- **Calendar data** - Cached locally for offline access

### What We Never See
- Your meeting content
- Personal conversations
- Business discussions
- Meeting participants
- Calendar information
- File contents

## 📊 Analytics & Telemetry

To improve Hyprnote, we collect minimal, non-personal analytics data:

### What We Collect
- **Usage statistics** - Feature usage patterns (anonymized)
- **Performance metrics** - App performance and crash reports
- **Device information** - OS version, app version (no personal identifiers)

### What We Don't Collect
- Meeting content or transcriptions
- Personal information
- Location data
- Keystroke data
- Screen recordings

### Opting Out

You can disable all analytics from the Settings menu:

```
Settings → Privacy → Analytics → Disable
```

You can also verify what analytics code exists in the application:

```bash
# Search for analytics events in the codebase
grep -r ".event(" apps/desktop/src
grep -r ".event(" apps/desktop/src-tauri/src
```

## 🌐 Network Activity

### When Hyprnote Connects to the Internet

1. **Initial setup** - Downloading AI models (Whisper, Llama)
2. **Software updates** - Checking for new versions
3. **Optional integrations** - Calendar sync (if enabled)
4. **Analytics** - Anonymous usage data (if enabled)

### When It Doesn't

- **During meetings** - All processing is local
- **While transcribing** - No data transmitted
- **When generating summaries** - Entirely offline
- **In airplane mode** - Full functionality available

## 🔗 Third-Party Integrations

### Calendar Services
When you connect calendar services (Google, Outlook, Apple), Hyprnote:
- Requests read-only access to calendar events
- Caches event data locally for offline access
- Does not transmit calendar data to our servers

### Cloud Storage (Optional)
If you enable cloud backup:
- Data is encrypted before transmission
- You control the storage provider
- We cannot decrypt your data

## 🛡️ Security Measures

- **Local processing** - AI models run entirely on your device
- **Encrypted storage** - Local data is encrypted at rest
- **No cloud dependencies** - Core functionality works offline
- **Open source** - Code is auditable and transparent

## 📱 Platform-Specific Privacy

### macOS
- Requires microphone permission for recording
- Requires calendar access (if integration enabled)
- All permissions can be revoked via System Preferences

### Windows (Coming Soon)
- Will follow Windows privacy guidelines
- Granular permission controls

## 🏢 Enterprise & Organizations

For enterprise users:
- All privacy protections apply equally
- No backdoors or administrative overrides
- Local deployment options available
- SOC 2 compliance (planned)

## 👨‍💼 Data Controller

Since all data processing happens locally on your device, **you are the data controller** of your information. Hyprnote acts only as a tool that processes data on your behalf.

## 📞 Contact & Transparency

### Questions About Privacy?
- Email: privacy@hyprnote.com
- Discord: [Join our community](https://hyprnote.com/discord)
- GitHub: [Open an issue](https://github.com/fastrepl/hyprnote/issues)

### Transparency Report
- No data breaches to report
- No government data requests received
- No user data ever transmitted to our servers

### Source Code
Hyprnote is open source. You can audit our privacy practices:
- **Repository**: [github.com/fastrepl/hyprnote](https://github.com/fastrepl/hyprnote)
- **Releases**: [All versions available](https://github.com/fastrepl/hyprnote/releases)

## 📝 Changes to This Policy

We will notify users of any material changes to this privacy policy through:
- In-app notifications
- GitHub repository updates
- Discord announcements

This policy is effective as of the date listed above and will remain in effect except with respect to any changes in its provisions in the future.

---

*Your privacy is not just a feature—it's our foundation.*