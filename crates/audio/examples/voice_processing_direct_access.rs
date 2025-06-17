// Example demonstrating direct access to voice processing features
// This shows how to use VoiceProcessingMicInput directly without the generic AudioInput wrapper

use audio::voice_processing_direct::*;
use futures_util::StreamExt;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎙️ Voice Processing Direct Access Examples");
    println!("==========================================");

    // Example 1: Basic VoiceProcessingMicInput (CoreAudio-based)
    println!("\n1. VoiceProcessingMicInput (CoreAudio-based):");
    
    let voice_mic = create_voice_processing_mic()?;
    let mut stream = voice_mic.stream()?;
    
    println!("   ✅ Created VoiceProcessingMicInput at {}Hz", stream.sample_rate());
    println!("   📝 Note: Uses CoreAudio for format compatibility with speaker tap");
    
    // Collect a few samples to verify it works
    let mut sample_count = 0;
    let timeout = tokio::time::timeout(Duration::from_secs(1), async {
        while let Some(_sample) = stream.next().await {
            sample_count += 1;
            if sample_count >= 100 {
                break;
            }
        }
    });
    
    match timeout.await {
        Ok(_) => println!("   ✅ Successfully captured {} samples", sample_count),
        Err(_) => println!("   ⚠️ Timeout - may need microphone permissions"),
    }

    // Example 2: AppleVoiceProcessingInput (Full AudioUnit-based voice processing)
    println!("\n2. AppleVoiceProcessingInput (Full AudioUnit voice processing):");
    
    // Create with all features enabled
    let apple_voice = create_apple_voice_processing()?;
    let apple_stream = apple_voice.stream()?;
    
    println!("   ✅ Created AppleVoiceProcessingInput with AGC, Noise Suppression, Echo Cancellation");
    println!("   🔧 Sample rate: {}Hz", apple_stream.sample_rate());
    
    // Example 3: AppleVoiceProcessingInput with custom configuration
    println!("\n3. AppleVoiceProcessingInput with custom configuration:");
    
    let custom_voice = create_apple_voice_processing_with_config(
        48000, // 48kHz sample rate
        true,  // AGC enabled
        true,  // Noise suppression enabled  
        false, // Echo cancellation disabled (no speaker reference)
    )?;
    let custom_stream = custom_voice.stream()?;
    
    println!("   ✅ Created custom AppleVoiceProcessingInput:");
    println!("      • Sample rate: {}Hz", custom_stream.sample_rate());
    println!("      • AGC: enabled");
    println!("      • Noise suppression: enabled");
    println!("      • Echo cancellation: disabled");

    // Example 4: AppleVoiceProcessingInput with speaker reference for echo cancellation
    println!("\n4. AppleVoiceProcessingInput with speaker reference:");
    
    let (speaker_ref, _speaker_producer) = create_speaker_reference();
    let voice_with_echo_cancel = create_apple_voice_processing_with_sample_rate(16000)?;
    let echo_cancel_stream = voice_with_echo_cancel.stream_with_speaker_reference(speaker_ref)?;
    
    println!("   ✅ Created AppleVoiceProcessingInput with speaker reference");
    println!("   🔇 Echo cancellation will use speaker output as reference");
    println!("   📊 Sample rate: {}Hz", echo_cancel_stream.sample_rate());

    // Example 5: IntegratedVoiceProcessing (combines mic + speaker)
    println!("\n5. IntegratedVoiceProcessing (integrated mic + speaker):");
    
    let integrated = create_integrated_voice_processing()?;
    let integrated_stream = integrated.stream()?;
    
    println!("   ✅ Created IntegratedVoiceProcessing");
    println!("   🔄 Automatically handles speaker reference for optimal echo cancellation");
    println!("   📊 Sample rate: {}Hz", integrated_stream.sample_rate());

    println!("\n📋 Summary of direct access options:");
    println!("   • VoiceProcessingMicInput: CoreAudio-based, format-compatible");
    println!("   • AppleVoiceProcessingInput: Full AudioUnit voice processing");
    println!("   • IntegratedVoiceProcessing: Combines mic + speaker for best echo cancellation");
    println!("\n💡 All implementations provide AsyncSource trait for streaming audio data");

    Ok(())
}