# Mobile Development

Hyprnote's mobile development is powered by **Tauri v2**, enabling the same Rust + TypeScript codebase to run on iOS and Android devices with native performance.

## 🏗️ Mobile Architecture

### Cross-Platform Strategy
- **Shared Core**: Rust business logic runs on all platforms
- **Native UI**: Tauri WebView with platform-specific optimizations
- **Plugin System**: Mobile-specific plugins for device integration
- **Offline-First**: Core functionality works without internet

### Platform Support Status
| Platform | Status | Notes |
|----------|--------|-------|
| **iOS** | 🚧 In Development | iPhone/iPad support coming soon |
| **Android** | 🤔 Planned | Following iOS completion |

## 📱 iOS Development

### Prerequisites

#### Development Environment
```bash
# Xcode Command Line Tools
xcode-select --install

# Xcode (from App Store)
# Required for iOS simulator and device deployment

# Rust iOS targets
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim
rustup target add x86_64-apple-ios
```

#### iOS Dependencies
```bash
# CocoaPods for iOS dependencies
sudo gem install cocoapods

# iOS development tools
brew install ios-sim ios-deploy

# Verify iOS targets
rustup target list --installed | grep ios
```

### Project Setup

#### Initialize iOS Support
```bash
# Add iOS capability to Tauri project
npm run tauri add ios

# Generate iOS project
npm run tauri ios init
```

#### iOS Project Structure
```
src-tauri/
├── gen/
│   └── apple/              # Generated iOS project
│       ├── hyprnote.xcodeproj
│       ├── Sources/
│       └── hyprnote_iOS/
├── capabilities/
│   └── ios.json           # iOS-specific permissions
└── tauri.conf.json        # iOS configuration
```

### Development Workflow

#### Running on Simulator
```bash
# List available simulators
npm run tauri ios list

# Run on specific simulator
npm run tauri ios dev --target "iPhone 15"

# Run with verbose logging
RUST_LOG=debug npm run tauri ios dev
```

#### Building for Device
```bash
# Build for iOS device
npm run tauri ios build

# Build for specific target
npm run tauri ios build --target aarch64-apple-ios

# Build for distribution
npm run tauri ios build --release
```

### iOS-Specific Configuration

#### Capabilities and Permissions
```json
// src-tauri/capabilities/ios.json
{
  "identifier": "ios-capabilities",
  "description": "iOS-specific capabilities",
  "permissions": [
    "microphone:allow-record",
    "notification:allow-send",
    "calendar:allow-read",
    "storage:allow-read-write"
  ]
}
```

#### Info.plist Customization
```xml
<!-- src-tauri/gen/apple/hyprnote_iOS/Info.plist -->
<dict>
    <!-- Microphone permission -->
    <key>NSMicrophoneUsageDescription</key>
    <string>Hyprnote needs microphone access for recording meetings</string>
    
    <!-- Calendar permission -->
    <key>NSCalendarsUsageDescription</key>
    <string>Access your calendar to link notes with meetings</string>
    
    <!-- Background audio -->
    <key>UIBackgroundModes</key>
    <array>
        <string>audio</string>
        <string>background-processing</string>
    </array>
</dict>
```

### iOS Testing

#### Simulator Testing
```bash
# Test on multiple simulators
npm run tauri ios dev --target "iPhone 15"
npm run tauri ios dev --target "iPad Pro (12.9-inch)"

# Test different iOS versions
npm run tauri ios dev --target "iPhone 14" --ios-version 16.0
```

#### Device Testing
```bash
# Deploy to connected device
npm run tauri ios dev --device

# Install on specific device
npm run tauri ios build --device "John's iPhone"
```

## 🤖 Android Development

### Prerequisites

#### Development Environment
```bash
# Android Studio (download from official site)
# https://developer.android.com/studio

# Android SDK and NDK via Android Studio
# SDK Manager > SDK Tools > 
# - Android SDK Build-Tools
# - Android NDK (Side by side)
# - CMake
```

#### Environment Variables
```bash
# Add to ~/.zshrc or ~/.bashrc
export ANDROID_HOME=$HOME/Library/Android/sdk
export NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653
export PATH=$PATH:$ANDROID_HOME/tools:$ANDROID_HOME/platform-tools
```

#### Rust Android Targets
```bash
# Add Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

### Project Setup

#### Initialize Android Support
```bash
# Add Android capability
npm run tauri add android

# Generate Android project
npm run tauri android init
```

#### Android Project Structure
```
src-tauri/
├── gen/
│   └── android/            # Generated Android project
│       ├── app/
│       ├── gradle/
│       └── build.gradle
├── capabilities/
│   └── android.json       # Android-specific permissions
└── tauri.conf.json        # Android configuration
```

### Development Workflow

#### Running on Emulator
```bash
# List available emulators
npm run tauri android list

# Start emulator
npm run tauri android emulator --device "Pixel_7_API_33"

# Run on emulator
npm run tauri android dev
```

#### Building APK
```bash
# Debug build
npm run tauri android build

# Release build
npm run tauri android build --release

# Build for specific architecture
npm run tauri android build --target aarch64-linux-android
```

### Android-Specific Configuration

#### Permissions and Capabilities
```json
// src-tauri/capabilities/android.json
{
  "identifier": "android-capabilities", 
  "description": "Android-specific capabilities",
  "permissions": [
    "android:RECORD_AUDIO",
    "android:WRITE_EXTERNAL_STORAGE",
    "android:READ_CALENDAR",
    "android:WAKE_LOCK"
  ]
}
```

#### AndroidManifest.xml
```xml
<!-- src-tauri/gen/android/app/src/main/AndroidManifest.xml -->
<manifest>
    <!-- Permissions -->
    <uses-permission android:name="android.permission.RECORD_AUDIO" />
    <uses-permission android:name="android.permission.READ_CALENDAR" />
    <uses-permission android:name="android.permission.WRITE_CALENDAR" />
    
    <!-- Features -->
    <uses-feature android:name="android.hardware.microphone" android:required="true" />
    
    <application>
        <!-- Background services -->
        <service android:name=".RecordingService" 
                 android:foregroundServiceType="microphone" />
    </application>
</manifest>
```

## 📱 Mobile-Specific Features

### Audio Recording
```rust
// Mobile-optimized audio recording
#[cfg(mobile)]
#[command]
pub async fn start_mobile_recording() -> Result<(), String> {
    // Request microphone permission
    if !check_microphone_permission().await? {
        request_microphone_permission().await?;
    }
    
    // Configure for mobile constraints
    let config = AudioConfig {
        sample_rate: 16000,  // Lower for mobile
        channels: 1,         // Mono for efficiency
        format: AudioFormat::S16LE,
    };
    
    start_recording_with_config(config).await
}
```

### Battery Optimization
```rust
// Battery-conscious processing
#[cfg(mobile)]
pub struct MobileOptimizedProcessor {
    batch_size: usize,
    processing_interval: Duration,
}

impl MobileOptimizedProcessor {
    pub fn new() -> Self {
        Self {
            batch_size: 512,  // Smaller batches
            processing_interval: Duration::from_millis(100),
        }
    }
    
    pub async fn process_efficiently(&self, audio: &[f32]) {
        // Batch processing to reduce CPU usage
        for chunk in audio.chunks(self.batch_size) {
            self.process_chunk(chunk).await;
            tokio::time::sleep(self.processing_interval).await;
        }
    }
}
```

### Storage Management
```typescript
// Mobile storage optimization
interface MobileStorageConfig {
  maxCacheSize: number
  compressionLevel: number
  offlineRetention: number
}

export class MobileStorage {
  private config: MobileStorageConfig = {
    maxCacheSize: 100 * 1024 * 1024, // 100MB
    compressionLevel: 6,
    offlineRetention: 7, // days
  }
  
  async optimizeStorage(): Promise<void> {
    await this.cleanupOldSessions()
    await this.compressInactiveSessions()
    await this.clearTemporaryFiles()
  }
}
```

## 🔧 Mobile UI Considerations

### Responsive Design
```css
/* Mobile-first CSS */
@media (max-width: 768px) {
  .desktop-sidebar {
    display: none;
  }
  
  .mobile-nav {
    display: flex;
    position: fixed;
    bottom: 0;
    width: 100%;
  }
}

/* Touch-friendly interactions */
.touch-target {
  min-height: 44px;
  min-width: 44px;
  padding: 12px;
}
```

### Mobile Components
```typescript
// Mobile-optimized components
export function MobileRecordingButton() {
  const [isRecording, setIsRecording] = useState(false)
  
  return (
    <TouchableButton
      size="large"
      onPress={() => toggleRecording()}
      hapticFeedback="medium"
      className="recording-button"
    >
      {isRecording ? <StopIcon /> : <RecordIcon />}
    </TouchableButton>
  )
}

export function SwipeableNoteCard({ note, onSwipeLeft, onSwipeRight }) {
  return (
    <GestureHandler
      onSwipeLeft={() => onSwipeLeft(note)}
      onSwipeRight={() => onSwipeRight(note)}
    >
      <NoteCard note={note} />
    </GestureHandler>
  )
}
```

## 🧪 Mobile Testing

### Automated Testing
```bash
# iOS testing
npm run test:ios:unit
npm run test:ios:integration

# Android testing  
npm run test:android:unit
npm run test:android:integration

# Cross-platform mobile tests
npm run test:mobile
```

### Manual Testing Checklist
- [ ] **Recording Quality** - Test in various environments
- [ ] **Battery Usage** - Monitor power consumption
- [ ] **Memory Usage** - Check for memory leaks
- [ ] **Offline Functionality** - Test without internet
- [ ] **Permissions** - Verify permission requests
- [ ] **Background Processing** - Test app backgrounding
- [ ] **Device Rotation** - Test orientation changes
- [ ] **Different Screen Sizes** - Test on various devices

### Performance Testing
```rust
// Mobile performance benchmarks
#[cfg(test)]
mod mobile_benchmarks {
    use super::*;
    
    #[test]
    fn benchmark_audio_processing() {
        let audio_data = generate_test_audio(16000, 1.0); // 1 second
        let start = Instant::now();
        
        process_audio_mobile(&audio_data);
        
        let duration = start.elapsed();
        assert!(duration < Duration::from_millis(100), "Processing too slow for mobile");
    }
}
```

## 📦 Mobile Distribution

### iOS App Store
```bash
# Build for App Store
npm run tauri ios build --release --target aarch64-apple-ios

# Archive for distribution
xcodebuild archive \
  -project src-tauri/gen/apple/hyprnote.xcodeproj \
  -scheme hyprnote \
  -archivePath hyprnote.xcarchive

# Upload to App Store Connect
xcodebuild -exportArchive \
  -archivePath hyprnote.xcarchive \
  -exportPath . \
  -exportOptionsPlist ExportOptions.plist
```

### Android Play Store
```bash
# Generate signed APK
npm run tauri android build --release --bundle aab

# Upload to Play Console
# Use Android Studio or Play Console web interface
```

## 🔍 Debugging Mobile Apps

### iOS Debugging
```bash
# View iOS logs
npm run tauri ios dev --verbose

# Debug with Xcode
open src-tauri/gen/apple/hyprnote.xcodeproj

# Safari Web Inspector (for WebView debugging)
# Safari > Develop > [Device] > Hyprnote
```

### Android Debugging
```bash
# View Android logs
adb logcat | grep hyprnote

# Chrome DevTools (for WebView debugging)
# chrome://inspect in Chrome browser

# Android Studio debugging
npm run tauri android dev --open
```

### Remote Debugging
```typescript
// Remote debugging setup
if (process.env.NODE_ENV === 'development') {
  // Connect to desktop dev tools
  import('eruda').then(eruda => eruda.default.init())
}
```

## 🚨 Common Mobile Issues

### iOS Issues
```bash
# Codesigning errors
security find-identity -v -p codesigning

# Provisioning profile issues
npm run tauri ios build --device --provisioning-profile "Development Profile"

# Simulator not found
xcrun simctl list devices
```

### Android Issues
```bash
# NDK version mismatch
# Update NDK_HOME in environment variables

# Gradle build failures
cd src-tauri/gen/android && ./gradlew clean

# Device not recognized
adb devices
adb kill-server && adb start-server
```

### Performance Issues
```rust
// Optimize for mobile constraints
#[cfg(mobile)]
const MOBILE_CONFIG: ProcessingConfig = ProcessingConfig {
    max_concurrent_tasks: 2,
    chunk_size: 512,
    enable_gpu_acceleration: false, // Often unavailable on mobile
    power_save_mode: true,
};
```

## 📚 Mobile Resources

### Documentation
- [Tauri Mobile Guide](https://tauri.app/v2/guides/building/mobile/) - Official mobile development guide
- [iOS Development](https://developer.apple.com/ios/) - Apple's iOS development resources
- [Android Development](https://developer.android.com/) - Google's Android development resources

### Tools
- [iOS Simulator](https://developer.apple.com/documentation/xcode/running-your-app-in-simulator) - Testing iOS apps
- [Android Emulator](https://developer.android.com/studio/run/emulator) - Testing Android apps
- [React Native Debugger](https://github.com/jhen0409/react-native-debugger) - Mobile debugging tools

### Community
- [Discord #mobile-development](https://hyprnote.com/discord) - Mobile development discussions
- [Tauri Mobile Examples](https://github.com/tauri-apps/tauri/tree/dev/examples) - Reference implementations

## 🎯 Roadmap

### Short Term (Q1 2024)
- [ ] iOS app alpha release
- [ ] Basic recording functionality
- [ ] Calendar integration
- [ ] App Store submission

### Medium Term (Q2-Q3 2024)
- [ ] Android app development
- [ ] Cross-platform feature parity
- [ ] Performance optimizations
- [ ] Advanced mobile features

### Long Term (Q4 2024+)
- [ ] Mobile-specific AI models
- [ ] Offline speech recognition
- [ ] Widget support
- [ ] Wear OS/watchOS integration

---

**Ready to start mobile development?** Check out the [iOS prerequisites](https://tauri.app/start/prerequisites/#ios) and join our [Discord](https://hyprnote.com/discord) for mobile development support! 📱
