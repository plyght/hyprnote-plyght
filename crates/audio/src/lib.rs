mod errors;
mod mic;
#[cfg(target_os = "macos")]
pub mod audiounit_ffi;
#[cfg(target_os = "macos")]
mod apple_voice_processing;
#[cfg(target_os = "macos")]
mod integrated_voice_processing;
#[cfg(target_os = "macos")]
mod voice_processing_mic;
#[cfg(target_os = "macos")]
mod voice_processing_test;
mod norm;
mod speaker;
mod stream;

pub use errors::*;
pub use mic::*;
#[cfg(target_os = "macos")]
pub use apple_voice_processing::*;
#[cfg(target_os = "macos")]
pub use integrated_voice_processing::*;
#[cfg(target_os = "macos")]
pub use voice_processing_mic::*;
#[cfg(target_os = "macos")]
pub use voice_processing_test::*;
pub use norm::*;
pub use speaker::*;
pub use stream::*;

pub use cpal;

use futures_util::Stream;
pub use kalosm_sound::AsyncSource;
use anyhow::Result;

pub struct AudioOutput {}

impl AudioOutput {
    pub fn to_speaker(bytes: &'static [u8]) -> std::sync::mpsc::Sender<()> {
        use rodio::{Decoder, OutputStream, Sink};
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            if let Ok((_, stream)) = OutputStream::try_default() {
                let file = std::io::Cursor::new(bytes);
                if let Ok(source) = Decoder::new(file) {
                    let sink = Sink::try_new(&stream).unwrap();
                    sink.append(source);

                    let _ = rx.recv_timeout(std::time::Duration::from_secs(3600));
                    sink.stop();
                }
            }
        });

        tx
    }

    pub fn silence() -> std::sync::mpsc::Sender<()> {
        use rodio::{source::Zero, OutputStream, Sink};
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            if let Ok((_, stream)) = OutputStream::try_default() {
                let sink = Sink::try_new(&stream).unwrap();
                sink.append(Zero::<f32>::new(1, 16000));

                let _ = rx.recv();
                sink.stop();
            }
        });

        tx
    }
}

pub enum AudioSource {
    RealtimeMic,
    #[cfg(target_os = "macos")]
    VoiceProcessingMic,
    #[cfg(target_os = "macos")]
    AppleVoiceProcessing,
    #[cfg(target_os = "macos")]
    IntegratedVoiceProcessing,
    RealtimeSpeaker,
    Recorded,
}

pub struct AudioInput {
    source: AudioSource,
    mic: Option<MicInput>,
    #[cfg(target_os = "macos")]
    voice_processing_mic: Option<VoiceProcessingMicInput>,
    #[cfg(target_os = "macos")]
    apple_voice_processing: Option<AppleVoiceProcessingInput>,
    #[cfg(target_os = "macos")]
    integrated_voice_processing: Option<IntegratedVoiceProcessing>,
    speaker: Option<SpeakerInput>,
    data: Option<Vec<u8>>,
}

impl AudioInput {
    pub fn from_mic() -> Self {
        Self {
            source: AudioSource::RealtimeMic,
            mic: Some(MicInput::default()),
            #[cfg(target_os = "macos")]
            voice_processing_mic: None,
            #[cfg(target_os = "macos")]
            apple_voice_processing: None,
            #[cfg(target_os = "macos")]
            integrated_voice_processing: None,
            speaker: None,
            data: None,
        }
    }

    #[cfg(target_os = "macos")]
    pub fn from_voice_processing_mic() -> Result<Self, anyhow::Error> {
        Ok(Self {
            source: AudioSource::VoiceProcessingMic,
            mic: None,
            voice_processing_mic: Some(VoiceProcessingMicInput::new()?),
            apple_voice_processing: None,
            integrated_voice_processing: None,
            speaker: None,
            data: None,
        })
    }

    /// Create AudioInput using VoiceProcessingMicInput directly for full voice processing control
    /// This provides direct access to the VoiceProcessingMicInput without generic wrapping
    #[cfg(target_os = "macos")]
    pub fn from_voice_processing_mic_direct() -> Result<VoiceProcessingMicInput, anyhow::Error> {
        VoiceProcessingMicInput::new()
    }

    /// Create AudioInput using VoiceProcessingMicInput with custom sample rate
    #[cfg(target_os = "macos")]
    pub fn from_voice_processing_mic_with_sample_rate(sample_rate: u32) -> Result<VoiceProcessingMicInput, anyhow::Error> {
        VoiceProcessingMicInput::with_sample_rate(sample_rate)
    }

    /// Create AudioInput using Apple's VoiceProcessingIO AudioUnit with full voice processing features
    /// This provides AGC, noise suppression, and echo cancellation
    #[cfg(target_os = "macos")]
    pub fn from_apple_voice_processing() -> Result<Self, anyhow::Error> {
        Ok(Self {
            source: AudioSource::AppleVoiceProcessing,
            mic: None,
            voice_processing_mic: None,
            apple_voice_processing: Some(AppleVoiceProcessingInput::new()?),
            integrated_voice_processing: None,
            speaker: None,
            data: None,
        })
    }

    /// Create AppleVoiceProcessingInput directly for full voice processing control
    /// This provides direct access with AGC, noise suppression, and echo cancellation
    #[cfg(target_os = "macos")]
    pub fn from_apple_voice_processing_direct() -> Result<AppleVoiceProcessingInput, anyhow::Error> {
        AppleVoiceProcessingInput::new()
    }

    /// Create AppleVoiceProcessingInput with custom sample rate
    #[cfg(target_os = "macos")]
    pub fn from_apple_voice_processing_with_sample_rate(sample_rate: u32) -> Result<AppleVoiceProcessingInput, anyhow::Error> {
        AppleVoiceProcessingInput::with_sample_rate(sample_rate)
    }

    /// Create AppleVoiceProcessingInput with full configuration control
    #[cfg(target_os = "macos")]
    pub fn from_apple_voice_processing_with_config(
        sample_rate: u32,
        enable_agc: bool,
        enable_noise_suppression: bool,
        enable_echo_cancellation: bool,
    ) -> Result<AppleVoiceProcessingInput, anyhow::Error> {
        AppleVoiceProcessingInput::with_config(sample_rate, enable_agc, enable_noise_suppression, enable_echo_cancellation)
    }

    /// Create AudioInput using integrated voice processing that combines mic and speaker
    /// for optimal echo cancellation along with AGC and noise suppression
    #[cfg(target_os = "macos")]
    pub fn from_integrated_voice_processing() -> Result<Self, anyhow::Error> {
        Ok(Self {
            source: AudioSource::IntegratedVoiceProcessing,
            mic: None,
            voice_processing_mic: None,
            apple_voice_processing: None,
            integrated_voice_processing: Some(IntegratedVoiceProcessing::new()?),
            speaker: None,
            data: None,
        })
    }

    /// Create IntegratedVoiceProcessing directly for full integrated voice processing control
    /// This combines microphone input with speaker output reference for optimal echo cancellation
    #[cfg(target_os = "macos")]
    pub fn from_integrated_voice_processing_direct() -> Result<IntegratedVoiceProcessing, anyhow::Error> {
        IntegratedVoiceProcessing::new()
    }

    /// Create IntegratedVoiceProcessing with custom sample rates
    #[cfg(target_os = "macos")]
    pub fn from_integrated_voice_processing_with_sample_rate(
        sample_rate: u32,
        speaker_sample_rate_override: Option<u32>,
    ) -> Result<IntegratedVoiceProcessing, anyhow::Error> {
        IntegratedVoiceProcessing::with_sample_rate(sample_rate, speaker_sample_rate_override)
    }

    pub fn from_speaker(sample_rate_override: Option<u32>) -> Self {
        Self {
            source: AudioSource::RealtimeSpeaker,
            mic: None,
            #[cfg(target_os = "macos")]
            voice_processing_mic: None,
            #[cfg(target_os = "macos")]
            apple_voice_processing: None,
            #[cfg(target_os = "macos")]
            integrated_voice_processing: None,
            speaker: Some(SpeakerInput::new(sample_rate_override).unwrap()),
            data: None,
        }
    }

    pub fn from_recording(data: Vec<u8>) -> Self {
        Self {
            source: AudioSource::Recorded,
            mic: None,
            #[cfg(target_os = "macos")]
            voice_processing_mic: None,
            #[cfg(target_os = "macos")]
            apple_voice_processing: None,
            #[cfg(target_os = "macos")]
            integrated_voice_processing: None,
            speaker: None,
            data: Some(data),
        }
    }

    pub fn stream(&mut self) -> Result<AudioStream, anyhow::Error> {
        match &mut self.source {
            AudioSource::RealtimeMic => Ok(AudioStream::RealtimeMic {
                mic: self.mic.as_ref().unwrap().stream(),
            }),
            #[cfg(target_os = "macos")]
            AudioSource::VoiceProcessingMic => Ok(AudioStream::VoiceProcessingMic {
                stream: self.voice_processing_mic.take().unwrap().stream()?,
            }),
            #[cfg(target_os = "macos")]
            AudioSource::AppleVoiceProcessing => Ok(AudioStream::AppleVoiceProcessing {
                stream: self.apple_voice_processing.take().unwrap().stream()?,
            }),
            #[cfg(target_os = "macos")]
            AudioSource::IntegratedVoiceProcessing => Ok(AudioStream::IntegratedVoiceProcessing {
                stream: self.integrated_voice_processing.take().unwrap().stream()?,
            }),
            AudioSource::RealtimeSpeaker => Ok(AudioStream::RealtimeSpeaker {
                speaker: self.speaker.take().unwrap().stream().unwrap(),
            }),
            AudioSource::Recorded => Ok(AudioStream::Recorded {
                data: self.data.as_ref().unwrap().clone(),
                position: 0,
            }),
        }
    }
}

pub enum AudioStream {
    RealtimeMic { mic: MicStream },
    #[cfg(target_os = "macos")]
    VoiceProcessingMic { stream: VoiceProcessingMicStream },
    #[cfg(target_os = "macos")]
    AppleVoiceProcessing { stream: AppleVoiceProcessingStream },
    #[cfg(target_os = "macos")]
    IntegratedVoiceProcessing { stream: IntegratedVoiceProcessingStream },
    RealtimeSpeaker { speaker: SpeakerStream },
    Recorded { data: Vec<u8>, position: usize },
}

impl Stream for AudioStream {
    type Item = f32;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use futures_util::StreamExt;
        use std::task::Poll;

        match &mut *self {
            AudioStream::RealtimeMic { mic } => mic.poll_next_unpin(cx),
            #[cfg(target_os = "macos")]
            AudioStream::VoiceProcessingMic { stream } => stream.poll_next_unpin(cx),
            #[cfg(target_os = "macos")]
            AudioStream::AppleVoiceProcessing { stream } => stream.poll_next_unpin(cx),
            #[cfg(target_os = "macos")]
            AudioStream::IntegratedVoiceProcessing { stream } => stream.poll_next_unpin(cx),
            AudioStream::RealtimeSpeaker { speaker } => speaker.poll_next_unpin(cx),
            // assume pcm_s16le, without WAV header
            AudioStream::Recorded { data, position } => {
                if *position + 2 <= data.len() {
                    let bytes = [data[*position], data[*position + 1]];
                    let sample = i16::from_le_bytes(bytes) as f32 / 32768.0;
                    *position += 2;

                    std::thread::sleep(std::time::Duration::from_secs_f64(1.0 / 16000.0));
                    Poll::Ready(Some(sample))
                } else {
                    Poll::Ready(None)
                }
            }
        }
    }
}

impl kalosm_sound::AsyncSource for AudioStream {
    fn as_stream(&mut self) -> impl Stream<Item = f32> + '_ {
        self
    }

    fn sample_rate(&self) -> u32 {
        match self {
            AudioStream::RealtimeMic { mic } => mic.sample_rate(),
            #[cfg(target_os = "macos")]
            AudioStream::VoiceProcessingMic { stream } => stream.sample_rate(),
            #[cfg(target_os = "macos")]
            AudioStream::AppleVoiceProcessing { stream } => stream.sample_rate(),
            #[cfg(target_os = "macos")]
            AudioStream::IntegratedVoiceProcessing { stream } => stream.sample_rate(),
            AudioStream::RealtimeSpeaker { speaker } => speaker.sample_rate(),
            AudioStream::Recorded { .. } => 16000,
        }
    }
}

/// Direct access utilities for voice processing features
/// 
/// This module provides direct access to voice processing implementations without the generic
/// AudioInput wrapper. This allows for more fine-grained control and access to specific features.
/// 
/// # Available Voice Processing Options
/// 
/// ## 1. VoiceProcessingMicInput (CoreAudio-based)
/// - Uses CoreAudio framework directly
/// - Format-compatible with speaker tap system
/// - Foundation for voice processing integration
/// - Currently provides hardware audio input without processing
/// - Future extension point for AudioUnit integration
/// 
/// ## 2. AppleVoiceProcessingInput (Full AudioUnit voice processing)  
/// - Uses Apple's VoiceProcessingIO AudioUnit
/// - Hardware-accelerated processing on Apple Silicon
/// - Provides AGC, noise suppression, and echo cancellation
/// - Configurable features (can enable/disable individual processing)
/// - Supports speaker reference for echo cancellation
/// 
/// ## 3. IntegratedVoiceProcessing (Mic + Speaker integration)
/// - Combines microphone input with speaker output reference
/// - Automatic speaker reference handling for optimal echo cancellation
/// - Built on VoiceProcessingIO AudioUnit
/// - Best choice for applications with both input and output
/// 
/// # Usage Examples
/// 
/// ```rust,no_run
/// use audio::voice_processing_direct::*;
/// use futures_util::StreamExt;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Basic voice processing mic (CoreAudio-based)
/// let voice_mic = create_voice_processing_mic()?;
/// let mut stream = voice_mic.stream()?;
/// 
/// // Full Apple voice processing with all features
/// let apple_voice = create_apple_voice_processing()?;
/// let mut apple_stream = apple_voice.stream()?;
/// 
/// // Custom configuration
/// let custom_voice = create_apple_voice_processing_with_config(
///     48000, // sample rate
///     true,  // AGC
///     true,  // noise suppression  
///     false, // echo cancellation (no speaker reference)
/// )?;
/// let mut custom_stream = custom_voice.stream()?;
/// 
/// // With speaker reference for echo cancellation
/// let (speaker_ref, _speaker_producer) = create_speaker_reference();
/// let voice_with_echo = create_apple_voice_processing()?;
/// let mut echo_stream = voice_with_echo.stream_with_speaker_reference(speaker_ref)?;
/// 
/// // Integrated processing (handles speaker reference automatically)
/// let integrated = create_integrated_voice_processing()?;
/// let mut integrated_stream = integrated.stream()?;
/// 
/// // All streams implement AsyncSource and Stream<Item = f32>
/// while let Some(sample) = apple_stream.next().await {
///     // Process audio sample...
/// }
/// # Ok(())
/// # }
/// ```
/// 
/// # When to Use Each Option
/// 
/// - **VoiceProcessingMicInput**: When you need CoreAudio compatibility or plan to add
///   voice processing later
/// - **AppleVoiceProcessingInput**: When you need specific voice processing features with
///   fine-grained control
/// - **IntegratedVoiceProcessing**: When you have both microphone input and speaker output
///   and want automatic echo cancellation
#[cfg(target_os = "macos")]
pub mod voice_processing_direct {
    use super::*;
    use std::sync::{Arc, Mutex};
    use ringbuf::{HeapCons, HeapProd};

    /// Create a speaker reference consumer and producer pair for echo cancellation
    /// This allows voice processing to access speaker output for better echo cancellation
    pub fn create_speaker_reference() -> (Arc<Mutex<HeapCons<f32>>>, HeapProd<f32>) {
        crate::apple_voice_processing::create_speaker_reference_for_voice_processing()
    }

    /// Create a VoiceProcessingMicInput with default settings (16kHz, CoreAudio-based)
    pub fn create_voice_processing_mic() -> Result<VoiceProcessingMicInput, anyhow::Error> {
        VoiceProcessingMicInput::new()
    }

    /// Create a VoiceProcessingMicInput with custom sample rate
    pub fn create_voice_processing_mic_with_sample_rate(sample_rate: u32) -> Result<VoiceProcessingMicInput, anyhow::Error> {
        VoiceProcessingMicInput::with_sample_rate(sample_rate)
    }

    /// Create an AppleVoiceProcessingInput with full AudioUnit-based voice processing
    pub fn create_apple_voice_processing() -> Result<AppleVoiceProcessingInput, anyhow::Error> {
        AppleVoiceProcessingInput::new()
    }

    /// Create an AppleVoiceProcessingInput with custom sample rate  
    pub fn create_apple_voice_processing_with_sample_rate(sample_rate: u32) -> Result<AppleVoiceProcessingInput, anyhow::Error> {
        AppleVoiceProcessingInput::with_sample_rate(sample_rate)
    }

    /// Create an AppleVoiceProcessingInput with full configuration control
    pub fn create_apple_voice_processing_with_config(
        sample_rate: u32,
        enable_agc: bool,
        enable_noise_suppression: bool,
        enable_echo_cancellation: bool,
    ) -> Result<AppleVoiceProcessingInput, anyhow::Error> {
        AppleVoiceProcessingInput::with_config(sample_rate, enable_agc, enable_noise_suppression, enable_echo_cancellation)
    }

    /// Create an IntegratedVoiceProcessing that combines microphone and speaker for optimal echo cancellation
    pub fn create_integrated_voice_processing() -> Result<IntegratedVoiceProcessing, anyhow::Error> {
        IntegratedVoiceProcessing::new()
    }

    /// Create an IntegratedVoiceProcessing with custom sample rates
    pub fn create_integrated_voice_processing_with_sample_rate(
        sample_rate: u32,
        speaker_sample_rate_override: Option<u32>,
    ) -> Result<IntegratedVoiceProcessing, anyhow::Error> {
        IntegratedVoiceProcessing::with_sample_rate(sample_rate, speaker_sample_rate_override)
    }
}
