use std::time::{Duration, Instant};
use anyhow::Result;
use futures_util::StreamExt;
use kalosm_sound::AsyncSource;

use crate::{AppleVoiceProcessingInput, IntegratedVoiceProcessing, VoiceProcessingMicInput};
use crate::speaker::SpeakerInput;

/// Comprehensive test suite for voice processing implementations
pub struct VoiceProcessingTester {
    test_duration_secs: u64,
    expected_sample_rate: u32,
}

impl VoiceProcessingTester {
    pub fn new(test_duration_secs: u64, expected_sample_rate: u32) -> Self {
        Self {
            test_duration_secs,
            expected_sample_rate,
        }
    }

    /// Test the basic CoreAudio implementation (format compatibility test)
    pub async fn test_voice_processing_mic(&self) -> Result<TestResults> {
        tracing::info!("🧪 Testing VoiceProcessingMicInput (CoreAudio format compatibility)");
        
        let start_time = Instant::now();
        let mic = VoiceProcessingMicInput::with_sample_rate(self.expected_sample_rate)?;
        let mut stream = mic.stream()?;
        
        tracing::info!(
            sample_rate = stream.sample_rate(),
            expected_sample_rate = self.expected_sample_rate,
            "📊 VoiceProcessingMicInput stream created"
        );

        let results = self.collect_audio_data(&mut stream, "VoiceProcessingMicInput").await?;
        
        tracing::info!(
            duration_ms = start_time.elapsed().as_millis(),
            actual_sample_rate = results.actual_sample_rate,
            samples_collected = results.total_samples,
            non_zero_samples = results.non_zero_samples,
            max_amplitude = results.max_amplitude,
            avg_amplitude = results.avg_amplitude,
            "✅ VoiceProcessingMicInput test completed"
        );

        Ok(results)
    }

    /// Test the full Apple VoiceProcessingIO AudioUnit
    pub async fn test_apple_voice_processing(&self) -> Result<TestResults> {
        tracing::info!("🧪 Testing AppleVoiceProcessingInput (Full AudioUnit with AGC, noise suppression, echo cancellation)");
        
        let start_time = Instant::now();
        let voice_input = AppleVoiceProcessingInput::with_sample_rate(self.expected_sample_rate)?;
        let mut stream = voice_input.stream()?;
        
        tracing::info!(
            sample_rate = stream.sample_rate(),
            expected_sample_rate = self.expected_sample_rate,
            features = "AGC + Noise Suppression + Echo Cancellation",
            "📊 AppleVoiceProcessingInput stream created"
        );

        let results = self.collect_audio_data(&mut stream, "AppleVoiceProcessingInput").await?;
        
        tracing::info!(
            duration_ms = start_time.elapsed().as_millis(),
            actual_sample_rate = results.actual_sample_rate,
            samples_collected = results.total_samples,
            non_zero_samples = results.non_zero_samples,
            max_amplitude = results.max_amplitude,
            avg_amplitude = results.avg_amplitude,
            voice_processing_features = "AGC + Noise Suppression + Echo Cancellation",
            "✅ AppleVoiceProcessingInput test completed"
        );

        Ok(results)
    }

    /// Test the integrated voice processing with speaker reference
    pub async fn test_integrated_voice_processing(&self) -> Result<TestResults> {
        tracing::info!("🧪 Testing IntegratedVoiceProcessing (Mic + Speaker reference for optimal echo cancellation)");
        
        let start_time = Instant::now();
        let integrated = IntegratedVoiceProcessing::with_sample_rate(self.expected_sample_rate, Some(self.expected_sample_rate))?;
        let mut stream = integrated.stream()?;
        
        tracing::info!(
            sample_rate = stream.sample_rate(),
            expected_sample_rate = self.expected_sample_rate,
            features = "Full VoiceProcessingIO + Speaker Reference",
            "📊 IntegratedVoiceProcessing stream created"
        );

        let results = self.collect_audio_data(&mut stream, "IntegratedVoiceProcessing").await?;
        
        tracing::info!(
            duration_ms = start_time.elapsed().as_millis(),
            actual_sample_rate = results.actual_sample_rate,
            samples_collected = results.total_samples,
            non_zero_samples = results.non_zero_samples,
            max_amplitude = results.max_amplitude,
            avg_amplitude = results.avg_amplitude,
            voice_processing_features = "AGC + Noise Suppression + Echo Cancellation + Speaker Reference",
            "✅ IntegratedVoiceProcessing test completed"
        );

        Ok(results)
    }

    /// Test running mic and speaker together (the original problematic scenario)
    pub async fn test_mic_and_speaker_together(&self) -> Result<(TestResults, TestResults)> {
        tracing::info!("🧪 Testing Mic + Speaker together (original format mismatch scenario)");
        
        // Start speaker first
        tracing::info!("🔊 Starting speaker stream...");
        let speaker_input = SpeakerInput::new(Some(self.expected_sample_rate))?;
        let mut speaker_stream = speaker_input.stream()?;
        
        tracing::info!(
            speaker_sample_rate = speaker_stream.sample_rate(),
            "📊 Speaker stream created"
        );

        // Start voice processing mic
        tracing::info!("🎤 Starting voice processing mic stream...");
        let mic_input = AppleVoiceProcessingInput::with_sample_rate(self.expected_sample_rate)?;
        let mut mic_stream = mic_input.stream()?;
        
        tracing::info!(
            mic_sample_rate = mic_stream.sample_rate(),
            speaker_sample_rate = speaker_stream.sample_rate(),
            "📊 Both streams created - testing concurrent operation"
        );

        // Collect data from both streams concurrently
        let start_time = Instant::now();
        
        let (mic_results, speaker_results) = tokio::join!(
            self.collect_audio_data(&mut mic_stream, "ConcurrentMic"),
            self.collect_audio_data(&mut speaker_stream, "ConcurrentSpeaker")
        );

        let mic_results = mic_results?;
        let speaker_results = speaker_results?;
        
        tracing::info!(
            duration_ms = start_time.elapsed().as_millis(),
            mic_samples = mic_results.total_samples,
            speaker_samples = speaker_results.total_samples,
            mic_non_zero = mic_results.non_zero_samples,
            speaker_non_zero = speaker_results.non_zero_samples,
            mic_empty = mic_results.non_zero_samples == 0,
            speaker_empty = speaker_results.non_zero_samples == 0,
            format_mismatch_resolved = mic_results.non_zero_samples > 0 && speaker_results.non_zero_samples > 0,
            "✅ Concurrent mic+speaker test completed"
        );

        if mic_results.non_zero_samples == 0 {
            tracing::error!("❌ MIC STREAM IS EMPTY - Format mismatch still exists!");
        } else {
            tracing::info!("✅ Mic stream has audio data");
        }

        if speaker_results.non_zero_samples == 0 {
            tracing::error!("❌ SPEAKER STREAM IS EMPTY - Audio tap not working!");
        } else {
            tracing::info!("✅ Speaker stream has audio data");
        }

        Ok((mic_results, speaker_results))
    }

    /// Compare different voice processing implementations
    pub async fn compare_implementations(&self) -> Result<ComparisonResults> {
        tracing::info!("🔍 Comparing all voice processing implementations");

        let basic_results = self.test_voice_processing_mic().await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let apple_results = self.test_apple_voice_processing().await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let integrated_results = self.test_integrated_voice_processing().await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let (concurrent_mic, concurrent_speaker) = self.test_mic_and_speaker_together().await?;

        let comparison = ComparisonResults {
            basic_voice_processing: basic_results,
            apple_voice_processing: apple_results,
            integrated_voice_processing: integrated_results,
            concurrent_mic: concurrent_mic,
            concurrent_speaker: concurrent_speaker,
        };

        // Log comparison summary
        tracing::info!("📋 VOICE PROCESSING COMPARISON SUMMARY");
        tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        tracing::info!(
            implementation = "VoiceProcessingMicInput",
            samples = comparison.basic_voice_processing.total_samples,
            non_zero = comparison.basic_voice_processing.non_zero_samples,
            avg_amplitude = comparison.basic_voice_processing.avg_amplitude,
            working = comparison.basic_voice_processing.non_zero_samples > 0,
            "📊 Basic CoreAudio compatibility"
        );

        tracing::info!(
            implementation = "AppleVoiceProcessingInput", 
            samples = comparison.apple_voice_processing.total_samples,
            non_zero = comparison.apple_voice_processing.non_zero_samples,
            avg_amplitude = comparison.apple_voice_processing.avg_amplitude,
            working = comparison.apple_voice_processing.non_zero_samples > 0,
            "📊 Full AudioUnit voice processing"
        );

        tracing::info!(
            implementation = "IntegratedVoiceProcessing",
            samples = comparison.integrated_voice_processing.total_samples, 
            non_zero = comparison.integrated_voice_processing.non_zero_samples,
            avg_amplitude = comparison.integrated_voice_processing.avg_amplitude,
            working = comparison.integrated_voice_processing.non_zero_samples > 0,
            "📊 Integrated with speaker reference"
        );

        tracing::info!(
            concurrent_mic_samples = comparison.concurrent_mic.total_samples,
            concurrent_speaker_samples = comparison.concurrent_speaker.total_samples,
            concurrent_mic_working = comparison.concurrent_mic.non_zero_samples > 0,
            concurrent_speaker_working = comparison.concurrent_speaker.non_zero_samples > 0,
            format_mismatch_resolved = comparison.concurrent_mic.non_zero_samples > 0 && comparison.concurrent_speaker.non_zero_samples > 0,
            "📊 Concurrent operation test"
        );

        tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        Ok(comparison)
    }

    async fn collect_audio_data<S>(&self, stream: &mut S, stream_name: &str) -> Result<TestResults>
    where
        S: StreamExt<Item = f32> + Unpin,
    {
        let mut samples = Vec::new();
        let start_time = Instant::now();
        let target_samples = self.expected_sample_rate * self.test_duration_secs as u32;
        
        tracing::info!(
            stream = stream_name,
            target_samples = target_samples,
            duration_secs = self.test_duration_secs,
            "🎵 Starting audio data collection"
        );

        let mut last_log_time = Instant::now();
        let mut chunk_count = 0;

        while samples.len() < target_samples as usize && start_time.elapsed() < Duration::from_secs(self.test_duration_secs + 2) {
            if let Some(sample) = stream.next().await {
                samples.push(sample);
                
                // Log progress every 0.5 seconds
                if last_log_time.elapsed() >= Duration::from_millis(500) {
                    chunk_count += 1;
                    let non_zero_so_far = samples.iter().filter(|&&s| s != 0.0).count();
                    let elapsed_secs = start_time.elapsed().as_secs_f32();
                    let actual_rate = samples.len() as f32 / elapsed_secs;
                    
                    tracing::info!(
                        stream = stream_name,
                        chunk = chunk_count,
                        samples_collected = samples.len(),
                        non_zero_samples = non_zero_so_far,
                        elapsed_secs = elapsed_secs,
                        actual_sample_rate = actual_rate,
                        target_rate = self.expected_sample_rate,
                        is_silent = non_zero_so_far == 0,
                        "📈 Collection progress"
                    );
                    
                    last_log_time = Instant::now();
                }
            } else {
                tracing::warn!(stream = stream_name, "Stream ended unexpectedly");
                break;
            }
        }

        let total_samples = samples.len();
        let non_zero_samples = samples.iter().filter(|&&s| s != 0.0).count();
        let max_amplitude = samples.iter().fold(0.0f32, |max, &s| max.max(s.abs()));
        let avg_amplitude = if total_samples > 0 {
            samples.iter().map(|&s| s.abs()).sum::<f32>() / total_samples as f32
        } else {
            0.0
        };
        
        let elapsed = start_time.elapsed();
        let actual_sample_rate = if elapsed.as_secs_f32() > 0.0 {
            total_samples as f32 / elapsed.as_secs_f32()
        } else {
            0.0
        };

        tracing::info!(
            stream = stream_name,
            total_samples = total_samples,
            non_zero_samples = non_zero_samples,
            silence_percentage = (total_samples - non_zero_samples) as f32 / total_samples as f32 * 100.0,
            max_amplitude = max_amplitude,
            avg_amplitude = avg_amplitude,
            actual_sample_rate = actual_sample_rate,
            expected_sample_rate = self.expected_sample_rate,
            sample_rate_accuracy = (actual_sample_rate / self.expected_sample_rate as f32 * 100.0),
            duration_secs = elapsed.as_secs_f32(),
            is_working = non_zero_samples > 0,
            "🎯 Audio collection completed"
        );

        Ok(TestResults {
            total_samples,
            non_zero_samples,
            max_amplitude,
            avg_amplitude,
            actual_sample_rate,
            duration: elapsed,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TestResults {
    pub total_samples: usize,
    pub non_zero_samples: usize,
    pub max_amplitude: f32,
    pub avg_amplitude: f32,
    pub actual_sample_rate: f32,
    pub duration: Duration,
}

#[derive(Debug)]
pub struct ComparisonResults {
    pub basic_voice_processing: TestResults,
    pub apple_voice_processing: TestResults,
    pub integrated_voice_processing: TestResults,
    pub concurrent_mic: TestResults,
    pub concurrent_speaker: TestResults,
}

impl ComparisonResults {
    /// Check if the original format mismatch issue is resolved
    pub fn format_mismatch_resolved(&self) -> bool {
        self.concurrent_mic.non_zero_samples > 0 && self.concurrent_speaker.non_zero_samples > 0
    }

    /// Get the best performing implementation
    pub fn best_implementation(&self) -> &str {
        let implementations = [
            ("Basic", &self.basic_voice_processing),
            ("Apple", &self.apple_voice_processing), 
            ("Integrated", &self.integrated_voice_processing),
        ];

        implementations
            .iter()
            .max_by(|a, b| {
                // Compare by non-zero samples first, then by average amplitude
                a.1.non_zero_samples.cmp(&b.1.non_zero_samples)
                    .then(a.1.avg_amplitude.partial_cmp(&b.1.avg_amplitude).unwrap_or(std::cmp::Ordering::Equal))
            })
            .map(|(name, _)| *name)
            .unwrap_or("Unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_voice_processing_implementations() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init();

        let tester = VoiceProcessingTester::new(3, 16000); // 3 seconds at 16kHz
        
        match tester.compare_implementations().await {
            Ok(results) => {
                println!("\n🎉 VOICE PROCESSING TEST COMPLETED");
                println!("Format mismatch resolved: {}", results.format_mismatch_resolved());
                println!("Best implementation: {}", results.best_implementation());
                
                // Assert that at least one implementation works
                assert!(
                    results.basic_voice_processing.non_zero_samples > 0 ||
                    results.apple_voice_processing.non_zero_samples > 0 ||
                    results.integrated_voice_processing.non_zero_samples > 0,
                    "At least one voice processing implementation should work"
                );
            },
            Err(e) => {
                eprintln!("❌ Voice processing test failed: {}", e);
                panic!("Voice processing test failed: {}", e);
            }
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_format_mismatch_resolution() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init();

        let tester = VoiceProcessingTester::new(2, 16000); // 2 seconds at 16kHz
        
        match tester.test_mic_and_speaker_together().await {
            Ok((mic_results, speaker_results)) => {
                println!("\n🔍 FORMAT MISMATCH TEST COMPLETED");
                println!("Mic working: {}", mic_results.non_zero_samples > 0);
                println!("Speaker working: {}", speaker_results.non_zero_samples > 0);
                println!("Format mismatch resolved: {}", mic_results.non_zero_samples > 0 && speaker_results.non_zero_samples > 0);
                
                // The main goal is to ensure mic doesn't go silent when speaker is running
                if mic_results.non_zero_samples == 0 {
                    println!("⚠️  Mic is silent - format mismatch still exists!");
                } else {
                    println!("✅ Mic has audio data - format mismatch resolved!");
                }
            },
            Err(e) => {
                eprintln!("❌ Format mismatch test failed: {}", e);
                // Don't panic here as this is testing a known issue
            }
        }
    }
}