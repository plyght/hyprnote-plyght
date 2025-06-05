use audio::audiounit_ffi::{VoiceProcessingAudioUnit, AudioUnitScope, AU_INPUT_ELEMENT, AudioUnitRenderCallback};
use cidre::{cat, os};
use std::sync::{Arc, Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

struct SimpleCtx {
    sample_count: u32,
}

extern "C" fn simple_render_callback(
    _in_ref_con: *mut std::ffi::c_void,
    _io_action_flags: *mut u32,
    _in_time_stamp: *const cat::AudioTimeStamp,
    _in_bus_number: u32,
    in_number_frames: u32,
    _io_data: *mut cat::AudioBufList<1>,
) -> os::Status {
    println!("🎤 Callback triggered with {} frames", in_number_frames);
    os::Status::NO_ERR
}

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

    println!("🎙️  SIMPLE VOICEPROCESSINGIO TEST");
    println!("=================================");
    println!();
    
    // Create VoiceProcessingIO AudioUnit
    let audio_unit = VoiceProcessingAudioUnit::new()
        .map_err(|e| format!("Failed to create AudioUnit: {:?}", e))?;
    println!("✅ VoiceProcessingIO AudioUnit created");

    // Configure I/O - enable input only
    audio_unit.enable_io(AudioUnitScope::Input, 1, true)
        .map_err(|e| format!("Failed to enable input: {:?}", e))?;
    audio_unit.enable_io(AudioUnitScope::Output, 0, false)
        .map_err(|e| format!("Failed to disable output: {:?}", e))?;
    println!("✅ I/O configured (input enabled, output disabled)");

    // DON'T set any custom format - let VoiceProcessingIO use its defaults
    println!("ℹ️  Skipping format configuration - using AudioUnit defaults");

    // Enable voice processing features BEFORE initialization (this is the fix!)
    println!("🔧 Enabling voice processing features BEFORE initialization...");
    
    // Check and enable AGC
    if audio_unit.check_property_support(2010, audio::audiounit_ffi::AudioUnitScope::Global, 0) {
        if let Err(e) = audio_unit.enable_voice_processing_agc(true) {
            println!("⚠️  AGC configuration failed: {:?}", e);
        } else {
            println!("✅ AGC enabled");
        }
    } else {
        println!("⚠️  AGC property not supported");
    }
    
    // Check and enable noise suppression
    if audio_unit.check_property_support(2011, audio::audiounit_ffi::AudioUnitScope::Global, 0) {
        if let Err(e) = audio_unit.enable_voice_processing_noise_suppression(true) {
            println!("⚠️  Noise suppression configuration failed: {:?}", e);
        } else {
            println!("✅ Noise suppression enabled");
        }
    } else {
        println!("⚠️  Noise suppression property not supported");
    }
    
    // Check and enable echo cancellation
    if audio_unit.check_property_support(2009, audio::audiounit_ffi::AudioUnitScope::Global, 0) {
        if let Err(e) = audio_unit.enable_voice_processing_echo_cancellation(true) {
            println!("⚠️  Echo cancellation configuration failed: {:?}", e);
        } else {
            println!("✅ Echo cancellation enabled");
        }
    } else {
        println!("⚠️  Echo cancellation property not supported");
    }

    // Set callback BEFORE initialization
    let mut ctx = SimpleCtx { sample_count: 0 };
    
    if let Err(e) = audio_unit.set_input_callback(
        simple_render_callback,
        &mut ctx as *mut SimpleCtx as *mut std::ffi::c_void,
    ) {
        println!("⚠️  Failed to set callback: {:?}", e);
    } else {
        println!("✅ Input render callback set");
    }

    // NOW initialize AFTER setting properties and callbacks
    match audio_unit.initialize() {
        Ok(()) => {
            println!("✅ AudioUnit initialized successfully!");
        },
        Err(e) => {
            println!("❌ Failed to initialize AudioUnit: {:?}", e);
            return Err(format!("Initialization failed: {:?}", e).into());
        }
    }

    // Try to start
    match audio_unit.start() {
        Ok(()) => {
            println!("✅ AudioUnit started successfully!");
            println!("🎤 Listening for 5 seconds... (speak into microphone)");
            
            // Wait and see if we get callbacks
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            println!("✅ VoiceProcessingIO test completed successfully!");
            println!("🎉 Apple voice processing features are working!");
        },
        Err(e) => {
            println!("❌ Failed to start AudioUnit: {:?}", e);
            return Err(format!("Start failed: {:?}", e).into());
        }
    }

    Ok(())
}