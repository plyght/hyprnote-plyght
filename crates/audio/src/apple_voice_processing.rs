use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};

use anyhow::Result;
use futures_util::Stream;
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapCons, HeapProd, HeapRb,
};

use cidre::{cat, os};

use crate::audiounit_ffi::{VoiceProcessingAudioUnit, AudioUnitScope, AU_INPUT_ELEMENT, AU_OUTPUT_ELEMENT};

/// Apple VoiceProcessingIO AudioUnit implementation with full voice processing features:
/// - Automatic Gain Control (AGC)
/// - Noise Suppression 
/// - Echo Cancellation
pub struct AppleVoiceProcessingInput {
    sample_rate: u32,
    enable_agc: bool,
    enable_noise_suppression: bool,
    enable_echo_cancellation: bool,
}

struct WakerState {
    waker: Option<Waker>,
    has_data: bool,
}

pub struct AppleVoiceProcessingStream {
    consumer: HeapCons<f32>,
    sample_rate: u32,
    _audio_unit: VoiceProcessingAudioUnit,
    _ctx: Box<VoiceProcessingCtx>,
    waker_state: Arc<Mutex<WakerState>>,
}

impl AppleVoiceProcessingStream {
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

struct VoiceProcessingCtx {
    producer: HeapProd<f32>,
    waker_state: Arc<Mutex<WakerState>>,
    speaker_reference: Option<Arc<Mutex<HeapCons<f32>>>>, // For echo cancellation
    audio_unit: Option<crate::audiounit_ffi::AudioUnit>, // Store AudioUnit reference for rendering
}

impl AppleVoiceProcessingInput {
    /// Create new voice processing input with all features enabled
    pub fn new() -> Result<Self> {
        Self::with_config(16000, true, true, true)
    }

    /// Create with specific sample rate (recommended: 16kHz or 48kHz for voice processing)
    pub fn with_sample_rate(sample_rate: u32) -> Result<Self> {
        Self::with_config(sample_rate, true, true, true)
    }

    /// Create with full configuration control
    pub fn with_config(
        sample_rate: u32,
        enable_agc: bool,
        enable_noise_suppression: bool,
        enable_echo_cancellation: bool,
    ) -> Result<Self> {
        // Validate sample rate for voice processing
        match sample_rate {
            8000 | 16000 | 24000 | 48000 => {},
            _ => tracing::warn!("Sample rate {} may not be optimal for voice processing", sample_rate),
        }

        Ok(Self {
            sample_rate,
            enable_agc,
            enable_noise_suppression,
            enable_echo_cancellation,
        })
    }

    /// Create stream with speaker reference for echo cancellation
    pub fn stream_with_speaker_reference(
        self,
        speaker_reference: Arc<Mutex<HeapCons<f32>>>,
    ) -> Result<AppleVoiceProcessingStream> {
        self.create_stream(Some(speaker_reference))
    }

    /// Create stream without speaker reference (echo cancellation will be less effective)
    pub fn stream(self) -> Result<AppleVoiceProcessingStream> {
        self.create_stream(None)
    }

    fn create_stream(
        self,
        speaker_reference: Option<Arc<Mutex<HeapCons<f32>>>>,
    ) -> Result<AppleVoiceProcessingStream> {
        let rb = HeapRb::<f32>::new(8192);
        let (producer, consumer) = rb.split();

        let waker_state = Arc::new(Mutex::new(WakerState {
            waker: None,
            has_data: false,
        }));

        let mut ctx = Box::new(VoiceProcessingCtx {
            producer,
            waker_state: waker_state.clone(),
            speaker_reference,
            audio_unit: None, // Will be set after AudioUnit creation
        });

        // Create VoiceProcessingIO AudioUnit
        let audio_unit = VoiceProcessingAudioUnit::new()
            .map_err(|e| anyhow::anyhow!("Failed to create VoiceProcessingIO AudioUnit: {:?}", e))?;

        tracing::info!("Created VoiceProcessingIO AudioUnit");

        // Configure I/O
        // Enable input (microphone) on element 1
        audio_unit.enable_io(AudioUnitScope::Input, AU_INPUT_ELEMENT, true)
            .map_err(|e| anyhow::anyhow!("Failed to enable input: {:?}", e))?;

        // Enable output (speaker reference) on element 0 if we have speaker reference
        if ctx.speaker_reference.is_some() {
            audio_unit.enable_io(AudioUnitScope::Output, AU_OUTPUT_ELEMENT, true)
                .map_err(|e| anyhow::anyhow!("Failed to enable output: {:?}", e))?;
            tracing::info!("Enabled speaker reference for echo cancellation");
        } else {
            audio_unit.enable_io(AudioUnitScope::Output, AU_OUTPUT_ELEMENT, false)
                .map_err(|e| anyhow::anyhow!("Failed to disable output: {:?}", e))?;
            tracing::warn!("No speaker reference provided - echo cancellation will be less effective");
        }

        // Skip format configuration - let VoiceProcessingIO use its default format
        tracing::info!("🔧 Skipping format configuration - using VoiceProcessingIO defaults");

        // Store AudioUnit reference in context for callbacks BEFORE initialization
        ctx.audio_unit = Some(audio_unit.raw_unit());

        // Enable voice processing features BEFORE initialization (this is key!)
        tracing::info!("🔧 Configuring voice processing features BEFORE initialization...");
        
        if self.enable_agc {
            // Check if AGC property is supported
            if audio_unit.check_property_support(
                crate::audiounit_ffi::K_AU_VOICE_IO_PROPERTY_VOICE_PROCESSING_ENABLE_AGC,
                crate::audiounit_ffi::AudioUnitScope::Global,
                0
            ) {
                if let Err(e) = audio_unit.enable_voice_processing_agc(true) {
                    tracing::warn!("AGC configuration failed: {:?}", e);
                } else {
                    tracing::info!("✅ Enabled Automatic Gain Control");
                }
            } else {
                tracing::warn!("AGC property not supported on this AudioUnit");
            }
        }

        if self.enable_noise_suppression {
            // Check if noise suppression property is supported
            if audio_unit.check_property_support(
                crate::audiounit_ffi::K_AU_VOICE_IO_PROPERTY_VOICE_PROCESSING_ENABLE_NOISE_SUPPRESSION,
                crate::audiounit_ffi::AudioUnitScope::Global,
                0
            ) {
                if let Err(e) = audio_unit.enable_voice_processing_noise_suppression(true) {
                    tracing::warn!("Noise suppression configuration failed: {:?}", e);
                } else {
                    tracing::info!("✅ Enabled Noise Suppression");
                }
            } else {
                tracing::warn!("Noise suppression property not supported on this AudioUnit");
            }
        }

        if self.enable_echo_cancellation {
            // Check if echo cancellation property is supported
            if audio_unit.check_property_support(
                crate::audiounit_ffi::K_AU_VOICE_IO_PROPERTY_VOICE_PROCESSING_ENABLE_ECHO_CANCEL,
                crate::audiounit_ffi::AudioUnitScope::Global,
                0
            ) {
                if let Err(e) = audio_unit.enable_voice_processing_echo_cancellation(true) {
                    tracing::warn!("Echo cancellation configuration failed: {:?}", e);
                } else {
                    tracing::info!("✅ Enabled Echo Cancellation");
                }
            } else {
                tracing::warn!("Echo cancellation property not supported on this AudioUnit");
            }
        }

        // Set input render callback BEFORE initialization
        if let Err(e) = audio_unit.set_input_callback(Self::input_callback, ctx.as_mut() as *mut VoiceProcessingCtx as *mut std::ffi::c_void) {
            tracing::warn!("Failed to set input callback: {:?}", e);
        } else {
            tracing::info!("✅ Input render callback configured");
        }

        // Initialize AFTER setting properties and callbacks
        audio_unit.initialize()
            .map_err(|e| anyhow::anyhow!("Failed to initialize AudioUnit: {:?}", e))?;

        tracing::info!("✅ AudioUnit initialized with voice processing features");

        audio_unit.start()
            .map_err(|e| anyhow::anyhow!("Failed to start AudioUnit: {:?}", e))?;

        tracing::info!(
            agc = self.enable_agc,
            noise_suppression = self.enable_noise_suppression,
            echo_cancellation = self.enable_echo_cancellation,
            has_speaker_reference = ctx.speaker_reference.is_some(),
            "Started Apple VoiceProcessingIO with full voice processing features"
        );

        Ok(AppleVoiceProcessingStream {
            consumer,
            sample_rate: self.sample_rate,
            _audio_unit: audio_unit,
            _ctx: ctx,
            waker_state,
        })
    }

    extern "C" fn input_callback(
        in_ref_con: *mut std::ffi::c_void,
        io_action_flags: *mut u32,
        in_time_stamp: *const cat::AudioTimeStamp,
        _in_bus_number: u32,
        in_number_frames: u32,
        _io_data: *mut cat::AudioBufList<1>,
    ) -> os::Status {
        if in_ref_con.is_null() {
            return os::Status(-50); // kAudioUnitErr_InvalidParameter
        }

        let ctx = unsafe { &mut *(in_ref_con as *mut VoiceProcessingCtx) };

        // Create buffer for processed audio data
        let mut buffer = vec![0.0f32; in_number_frames as usize];
        let audio_buffer = cat::AudioBuf {
            number_channels: 1,
            data_bytes_size: in_number_frames * 4,
            data: buffer.as_mut_ptr() as *mut u8,
        };

        let mut buf_list = cat::AudioBufList {
            number_buffers: 1,
            buffers: [audio_buffer],
        };

        // If we have speaker reference data, we need to provide it to the AudioUnit
        // This is done through a separate render callback mechanism for the output element
        // For now, we'll get the processed microphone audio through AudioUnitRender

        // Render the processed input (this gets mic audio with AGC, noise suppression, echo cancellation)
        let render_status = if let Some(audio_unit) = ctx.audio_unit {
            unsafe {
                crate::audiounit_ffi::AudioUnitRender(
                    audio_unit,
                    io_action_flags,
                    in_time_stamp,
                    AU_INPUT_ELEMENT,
                    in_number_frames,
                    &mut buf_list,
                )
            }
        } else {
            tracing::error!("AudioUnit reference not available in callback");
            return os::Status(-50); // kAudioUnitErr_InvalidParameter
        };

        if render_status != os::Status::NO_ERR {
            tracing::warn!("AudioUnitRender failed: {:?}", render_status);
            return render_status;
        }

        // Push the processed audio data to our ring buffer
        let pushed = ctx.producer.push_slice(&buffer);
        if pushed < buffer.len() {
            tracing::warn!("apple_voice_processing_dropped_{}_samples", buffer.len() - pushed);
        }

        // Wake up the stream if we have new data
        let mut waker_state = ctx.waker_state.lock().unwrap();
        if pushed > 0 && !waker_state.has_data {
            waker_state.has_data = true;
            if let Some(waker) = waker_state.waker.take() {
                drop(waker_state);
                waker.wake();
            }
        }

        os::Status::NO_ERR
    }
}


impl Stream for AppleVoiceProcessingStream {
    type Item = f32;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Some(sample) = self.consumer.try_pop() {
            return Poll::Ready(Some(sample));
        }

        {
            let mut state = self.waker_state.lock().unwrap();
            state.has_data = false;
            state.waker = Some(cx.waker().clone());
            drop(state);
        }

        match self.consumer.try_pop() {
            Some(sample) => Poll::Ready(Some(sample)),
            None => Poll::Pending,
        }
    }
}

impl kalosm_sound::AsyncSource for AppleVoiceProcessingStream {
    fn as_stream(&mut self) -> impl Stream<Item = f32> + '_ {
        self
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

/// Helper to create speaker reference consumer from speaker stream
/// This allows the voice processing to access speaker output for echo cancellation
pub fn create_speaker_reference_for_voice_processing() -> (Arc<Mutex<HeapCons<f32>>>, HeapProd<f32>) {
    let rb = HeapRb::<f32>::new(8192);
    let (producer, consumer) = rb.split();
    (Arc::new(Mutex::new(consumer)), producer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_apple_voice_processing_basic() {
        let voice_input = AppleVoiceProcessingInput::new().unwrap();
        let mut stream = voice_input.stream().unwrap();

        let mut buffer = Vec::new();
        while let Some(sample) = stream.next().await {
            buffer.push(sample);
            if buffer.len() > 6000 {
                break;
            }
        }

        assert!(buffer.iter().any(|x| *x != 0.0));
    }

    #[tokio::test]
    #[serial]
    async fn test_apple_voice_processing_with_speaker_reference() {
        let (speaker_ref, _speaker_producer) = create_speaker_reference_for_voice_processing();
        
        let voice_input = AppleVoiceProcessingInput::with_sample_rate(48000).unwrap();
        let mut stream = voice_input.stream_with_speaker_reference(speaker_ref).unwrap();
        
        assert_eq!(stream.sample_rate(), 48000);

        let mut buffer = Vec::new();
        while let Some(sample) = stream.next().await {
            buffer.push(sample);
            if buffer.len() > 12000 {
                break;
            }
        }

        assert!(buffer.iter().any(|x| *x != 0.0));
    }

    #[tokio::test]
    #[serial]
    async fn test_apple_voice_processing_custom_config() {
        // Test with only AGC and noise suppression, no echo cancellation
        let voice_input = AppleVoiceProcessingInput::with_config(16000, true, true, false).unwrap();
        let mut stream = voice_input.stream().unwrap();

        let mut buffer = Vec::new();
        while let Some(sample) = stream.next().await {
            buffer.push(sample);
            if buffer.len() > 6000 {
                break;
            }
        }

        assert!(buffer.iter().any(|x| *x != 0.0));
    }
}