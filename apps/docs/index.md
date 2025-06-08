<script setup>
    const quickStartCards = [
        {
            title: "🚀 Quickstart",
            url: "/quickstart",
            body: "No signup required. Get started in minutes with our comprehensive getting started guide."
        },
        {
            title: "🎯 First Recording",
            url: "/tutorials/first-recording",
            body: "Record your first meeting and learn the basics of AI-powered note taking."
        },
        {
            title: "🤖 AI Enhancement",
            url: "/tutorials/ai-enhancement",
            body: "Transform rough notes into polished summaries with AI assistance."
        }
    ]

    const developmentCards = [
        {
            title: "🛠️ Contributing",
            url: "/development/contributing",
            body: "Join our community. Comprehensive guide for new contributors."
        },
        {
            title: "🖥️ Server Development",
            url: "/development/server",
            body: "Build and extend Hyprnote's backend services and web frontend."
        },
        {
            title: "🔌 Plugin Development",
            url: "/development/plugin",
            body: "Create custom plugins to extend Hyprnote's functionality."
        },
        {
            title: "📱 Mobile Development",
            url: "/development/mobile",
            body: "Develop Hyprnote for iOS and Android with Tauri mobile."
        }
    ]

    const featuresCards = [
        {
            title: "🔗 Plugins",
            url: "/plugins",
            body: "Explore available plugins for calendar, AI, storage, and more."
        },
        {
            title: "📚 Tutorials",
            url: "/tutorials",
            body: "Step-by-step guides for mastering Hyprnote's features."
        },
        {
            title: "🎨 Templates",
            url: "/tutorials/templates-workflows",
            body: "Create efficient workflows with custom note templates."
        }
    ]
</script>

<h1 class="flex items-center gap-2 font-mono"><div class="i-heroicons-bolt-20-solid h-8 w-8 bg-yellow-500"></div> Hyprnote</h1>

_**AI-powered meeting assistant that works offline.** `Open source`, `local-first`, and `extensible`._

Hyprnote transforms your meetings into structured, actionable notes using AI. Record conversations, get real-time transcriptions, and enhance your notes with powerful AI models—all while keeping your data completely private on your device.

## ✨ Key Features

🎙️ **Real-time transcription** with Whisper  
🤖 **AI-powered summaries** using local Llama models  
🔒 **Complete privacy** - your data never leaves your device  
📅 **Calendar integration** with Google, Outlook, and Apple Calendar  
🔌 **Extensible plugin system** for custom integrations  
🌐 **Works offline** - no internet required for core functionality  

## 🚀 Getting Started

Perfect for first-time users who want to start taking better meeting notes immediately.

<div class="grid grid-cols-1 md:grid-cols-3 gap-4 my-8">
  <Card v-for="card in quickStartCards" :key="card.title" :title="card.title" :url="card.url" :body="card.body"/>
</div>

## 🛠️ Development

For developers who want to contribute, extend, or customize Hyprnote.

<div class="grid grid-cols-1 md:grid-cols-2 gap-4 my-8">
  <Card v-for="card in developmentCards" :key="card.title" :title="card.title" :url="card.url" :body="card.body"/>
</div>

## 📖 Features & Guides

Explore advanced features and learn how to get the most out of Hyprnote.

<div class="grid grid-cols-1 md:grid-cols-3 gap-4 my-8">
  <Card v-for="card in featuresCards" :key="card.title" :title="card.title" :url="card.url" :body="card.body"/>
</div>

## 🎯 Popular Use Cases

### For Remote Teams
- **Daily standups** with automatic action item extraction
- **Sprint planning** with AI-generated summaries
- **Client calls** with professional meeting minutes
- **All-hands meetings** with structured note taking

### For Individuals
- **1:1 meetings** with managers and direct reports
- **Interview recordings** with candidate evaluation notes
- **Lecture notes** with AI-enhanced study materials
- **Personal brainstorming** with idea organization

### For Organizations
- **Board meetings** with formal minute generation
- **Project reviews** with decision tracking
- **Training sessions** with structured learning outcomes
- **Customer feedback** sessions with insight extraction

## 🔧 Technical Highlights

### Privacy-First Architecture
- **Local AI models** run entirely on your device
- **No cloud dependencies** for core functionality
- **Encrypted local storage** protects your data
- **Optional cloud sync** for team collaboration

### Modern Technology Stack
- **Rust backend** for performance and security
- **Tauri framework** for cross-platform native apps
- **React frontend** with modern web technologies
- **Plugin system** built on secure foundations

### Offline Capabilities
- **Speech recognition** works without internet
- **AI enhancement** using local models
- **Note organization** and search
- **Export functionality** in multiple formats

## 🆘 Need Help?

### Quick Support
- **[Troubleshooting Guide](./tutorials/troubleshooting.md)** - Solve common issues
- **[Performance Tips](./tutorials/performance-tips.md)** - Optimize your setup
- **[Keyboard Shortcuts](./tutorials/shortcuts.md)** - Work faster

### Community
- **[Discord Server](https://hyprnote.com/discord)** - Get help from the community
- **[GitHub Discussions](https://github.com/fastrepl/hyprnote/discussions)** - Feature requests and ideas
- **[GitHub Issues](https://github.com/fastrepl/hyprnote/issues)** - Report bugs and problems

### Stay Updated
- **[Follow on X](https://x.com/tryhyprnote)** - Latest news and updates
- **[Release Notes](https://github.com/fastrepl/hyprnote/releases)** - What's new in each version
- **[Roadmap](https://github.com/fastrepl/hyprnote/projects)** - Planned features and improvements

---

**Ready to transform your meeting notes?** [Download Hyprnote](https://hyprnote.com/download) and start with our [Quickstart Guide](./quickstart.md)!

