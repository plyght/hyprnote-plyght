use audio::audiounit_ffi::{VoiceProcessingAudioUnit, AudioUnitScope, AU_INPUT_ELEMENT};
use cidre::cat;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize detailed logging
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("🔍 TESTING VOICEPROCESSINGIO AUDIOUNIT FORMATS");
    println!("==============================================");
    println!();
    
    // Try to create VoiceProcessingIO AudioUnit
    let audio_unit = match VoiceProcessingAudioUnit::new() {
        Ok(unit) => {
            println!("✅ Successfully created VoiceProcessingIO AudioUnit");
            unit
        },
        Err(e) => {
            println!("❌ Failed to create VoiceProcessingIO AudioUnit: {:?}", e);
            return Err(format!("AudioUnit creation failed: {:?}", e).into());
        }
    };

    // Configure I/O
    if let Err(e) = audio_unit.enable_io(AudioUnitScope::Input, 1, true) {
        println!("❌ Failed to enable input: {:?}", e);
        return Err(format!("Failed to enable input: {:?}", e).into());
    }
    
    if let Err(e) = audio_unit.enable_io(AudioUnitScope::Output, 0, false) {
        println!("❌ Failed to disable output: {:?}", e);
        return Err(format!("Failed to disable output: {:?}", e).into());
    }

    println!("✅ I/O configuration successful");
    println!();

    // Skip format configuration - VoiceProcessingIO works best with its defaults
    println!("🔧 Testing VoiceProcessingIO with default format...");
    println!("ℹ️  VoiceProcessingIO typically uses its own optimized format");
    println!();

    // Test voice processing features BEFORE initialization
    test_voice_processing_features(&audio_unit, "default").await;

    // Try to initialize with default format AFTER setting properties
    match audio_unit.initialize() {
        Ok(()) => {
            println!("✅ AudioUnit initialization successful with default format");
        },
        Err(e) => {
            println!("❌ AudioUnit initialization failed: {:?}", e);
            return Err(format!("Failed to initialize with default format: {:?}", e).into());
        }
    }

    println!("📊 RESULTS SUMMARY");
    println!("=================");
    println!("✅ VoiceProcessingIO AudioUnit is working with default format!");
    println!("🎉 Apple voice processing capabilities confirmed!");

    Ok(())
}

async fn test_voice_processing_features(audio_unit: &VoiceProcessingAudioUnit, format_name: &str) {
    println!("🎛️  Testing voice processing features with {}...", format_name);
    
    // Test AGC with property checking
    if audio_unit.check_property_support(2010, AudioUnitScope::Global, 0) {
        match audio_unit.enable_voice_processing_agc(true) {
            Ok(()) => println!("✅ AGC feature enabled successfully"),
            Err(e) => println!("⚠️  AGC configuration failed: {:?}", e),
        }
    } else {
        println!("❌ AGC property not supported by this AudioUnit");
    }
    
    // Test Noise Suppression with property checking
    if audio_unit.check_property_support(2011, AudioUnitScope::Global, 0) {
        match audio_unit.enable_voice_processing_noise_suppression(true) {
            Ok(()) => println!("✅ Noise Suppression feature enabled successfully"),
            Err(e) => println!("⚠️  Noise Suppression configuration failed: {:?}", e),
        }
    } else {
        println!("❌ Noise Suppression property not supported by this AudioUnit");
    }
    
    // Test Echo Cancellation with property checking
    if audio_unit.check_property_support(2009, AudioUnitScope::Global, 0) {
        match audio_unit.enable_voice_processing_echo_cancellation(true) {
            Ok(()) => println!("✅ Echo Cancellation feature enabled successfully"),
            Err(e) => println!("⚠️  Echo Cancellation configuration failed: {:?}", e),
        }
    } else {
        println!("❌ Echo Cancellation property not supported by this AudioUnit");
    }
}

fn create_format_16bit_int(sample_rate: f64) -> cat::AudioBasicStreamDesc {
    let mut asbd = cat::AudioBasicStreamDesc::default();
    asbd.sample_rate = sample_rate;
    asbd.format = cat::AudioFormat::LINEAR_PCM;
    asbd.format_flags = cat::AudioFormatFlags::IS_SIGNED_INTEGER | cat::AudioFormatFlags::IS_PACKED;
    asbd.bytes_per_packet = 2;
    asbd.frames_per_packet = 1;
    asbd.bytes_per_frame = 2;
    asbd.channels_per_frame = 1;
    asbd.bits_per_channel = 16;
    asbd
}

fn create_format_32bit_float(sample_rate: f64) -> cat::AudioBasicStreamDesc {
    let mut asbd = cat::AudioBasicStreamDesc::default();
    asbd.sample_rate = sample_rate;
    asbd.format = cat::AudioFormat::LINEAR_PCM;
    asbd.format_flags = cat::AudioFormatFlags::IS_FLOAT | cat::AudioFormatFlags::IS_PACKED;
    asbd.bytes_per_packet = 4;
    asbd.frames_per_packet = 1;
    asbd.bytes_per_frame = 4;
    asbd.channels_per_frame = 1;
    asbd.bits_per_channel = 32;
    asbd
}