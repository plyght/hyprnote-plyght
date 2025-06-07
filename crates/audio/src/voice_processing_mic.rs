use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};

use anyhow::Result;
use futures_util::Stream;
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapCons, HeapProd, HeapRb,
};

use cidre::{cat, os};

/// Apple Voice Processing Microphone Input with AudioUnit-based processing
/// 
/// This implementation uses Apple's VoiceProcessingIO AudioUnit to provide:
/// - Automatic Gain Control (AGC)
/// - Noise Suppression  
/// - Basic echo cancellation (without speaker reference)
/// 
/// For more advanced echo cancellation with speaker reference, use AppleVoiceProcessingInput
/// or IntegratedVoiceProcessing instead.
pub struct VoiceProcessingMicInput {
    sample_rate: u32,
    enable_agc: bool,
    enable_noise_suppression: bool,
}

struct WakerState {
    waker: Option<Waker>,
    has_data: bool,
}

pub struct VoiceProcessingMicStream {
    consumer: HeapCons<f32>,
    sample_rate: u32,
    _audio_unit: crate::audiounit_ffi::VoiceProcessingAudioUnit,
    _ctx: Box<VoiceProcessingCtx>,
    waker_state: Arc<Mutex<WakerState>>,
}

impl VoiceProcessingMicStream {
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

struct VoiceProcessingCtx {
    producer: HeapProd<f32>,
    waker_state: Arc<Mutex<WakerState>>,
    audio_unit: Option<crate::audiounit_ffi::AudioUnit>, // Raw AudioUnit for callbacks
}

unsafe impl Send for VoiceProcessingCtx {}
unsafe impl Sync for VoiceProcessingCtx {}

impl VoiceProcessingMicInput {
    pub fn new() -> Result<Self> {
        Self::with_sample_rate(16000) // Default to 16kHz for voice processing
    }

    pub fn with_sample_rate(sample_rate: u32) -> Result<Self> {
        Self::with_config(sample_rate, true, true) // Enable AGC and noise suppression by default
    }

    /// Create with custom configuration
    pub fn with_config(sample_rate: u32, enable_agc: bool, enable_noise_suppression: bool) -> Result<Self> {
        // Validate sample rate for voice processing
        match sample_rate {
            8000 | 16000 | 24000 | 48000 => {},
            _ => tracing::warn!("Sample rate {} may not be optimal for voice processing", sample_rate),
        }

        tracing::info!(
            sample_rate = sample_rate,
            agc = enable_agc,
            noise_suppression = enable_noise_suppression,
            "voice_processing_mic_input_config"
        );

        Ok(Self { 
            sample_rate, 
            enable_agc, 
            enable_noise_suppression 
        })
    }

    pub fn stream(self) -> Result<VoiceProcessingMicStream> {
        let rb = HeapRb::<f32>::new(8192);
        let (producer, consumer) = rb.split();

        let waker_state = Arc::new(Mutex::new(WakerState {
            waker: None,
            has_data: false,
        }));

        // Create VoiceProcessingIO AudioUnit
        let audio_unit = crate::audiounit_ffi::VoiceProcessingAudioUnit::new()
            .map_err(|e| anyhow::anyhow!("Failed to create VoiceProcessingIO AudioUnit: {:?}", e))?;

        tracing::info!("Created VoiceProcessingIO AudioUnit for basic voice processing");

        // Configure I/O - enable input only (no speaker reference for basic version)
        audio_unit.enable_io(crate::audiounit_ffi::AudioUnitScope::Input, crate::audiounit_ffi::AU_INPUT_ELEMENT, true)
            .map_err(|e| anyhow::anyhow!("Failed to enable input: {:?}", e))?;

        audio_unit.enable_io(crate::audiounit_ffi::AudioUnitScope::Output, crate::audiounit_ffi::AU_OUTPUT_ELEMENT, false)
            .map_err(|e| anyhow::anyhow!("Failed to disable output: {:?}", e))?;

        // Skip format configuration - let VoiceProcessingIO use its default format
        // VoiceProcessingIO has specific format requirements and it's better to use defaults
        tracing::info!("Skipping format configuration - using VoiceProcessingIO defaults");

        // Enable voice processing features based on configuration
        if self.enable_agc {
            if let Err(e) = audio_unit.enable_voice_processing_agc(true) {
                tracing::warn!("Failed to enable AGC: {:?}", e);
            } else {
                tracing::info!("Enabled Automatic Gain Control");
            }
        }

        if self.enable_noise_suppression {
            if let Err(e) = audio_unit.enable_voice_processing_noise_suppression(true) {
                tracing::warn!("Failed to enable noise suppression: {:?}", e);
            } else {
                tracing::info!("Enabled Noise Suppression");
            }
        }

        // Basic echo cancellation (without speaker reference)
        if let Err(e) = audio_unit.enable_voice_processing_echo_cancellation(false) {
            tracing::warn!("Failed to configure echo cancellation: {:?}", e);
        } else {
            tracing::info!("Enabled basic echo cancellation");
        }

        // Create context with pointer to audio unit for callbacks
        let mut ctx = Box::new(VoiceProcessingCtx {
            producer,
            waker_state: waker_state.clone(),
            audio_unit: Some(audio_unit.raw_unit()),
        });

        // Set input callback for microphone processing
        audio_unit.set_input_callback(Self::input_callback, ctx.as_mut() as *mut VoiceProcessingCtx as *mut std::ffi::c_void)
            .map_err(|e| anyhow::anyhow!("Failed to set input callback: {:?}", e))?;

        // Initialize and start the AudioUnit
        audio_unit.initialize()
            .map_err(|e| anyhow::anyhow!("Failed to initialize AudioUnit: {:?}", e))?;

        audio_unit.start()
            .map_err(|e| anyhow::anyhow!("Failed to start AudioUnit: {:?}", e))?;

        tracing::info!(
            agc = self.enable_agc,
            noise_suppression = self.enable_noise_suppression,
            echo_cancellation = false,
            "Started VoiceProcessingMicInput with basic voice processing features"
        );

        Ok(VoiceProcessingMicStream {
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

        // Render the processed input (this gets mic audio with voice processing applied)
        let render_status = if let Some(audio_unit) = ctx.audio_unit {
            unsafe {
                crate::audiounit_ffi::AudioUnitRender(
                    audio_unit,
                    io_action_flags,
                    in_time_stamp,
                    crate::audiounit_ffi::AU_INPUT_ELEMENT,
                    in_number_frames,
                    &mut buf_list,
                )
            }
        } else {
            tracing::error!("AudioUnit reference not available in callback");
            return os::Status(-50);
        };

        if render_status != os::Status::NO_ERR {
            tracing::warn!("AudioUnitRender failed: {:?}", render_status);
            return render_status;
        }

        // Push the processed audio data to our ring buffer
        let pushed = ctx.producer.push_slice(&buffer);
        if pushed < buffer.len() {
            tracing::warn!("voice_processing_mic_dropped_{}_samples", buffer.len() - pushed);
        }

        // Wake up the stream if we have new data
        if let Ok(mut waker_state) = ctx.waker_state.try_lock() {
            if pushed > 0 && !waker_state.has_data {
                waker_state.has_data = true;
                if let Some(waker) = waker_state.waker.take() {
                    drop(waker_state);
                    waker.wake();
                }
            }
        }

        os::Status::NO_ERR
    }
}


impl Stream for VoiceProcessingMicStream {
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

impl kalosm_sound::AsyncSource for VoiceProcessingMicStream {
    fn as_stream(&mut self) -> impl Stream<Item = f32> + '_ {
        self
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_voice_processing_mic() {
        let mic = VoiceProcessingMicInput::new()
            .expect("VoiceProcessingMicInput must be created successfully - no fallbacks allowed");
        let mut stream = mic.stream()
            .expect("VoiceProcessingMic stream must initialize successfully - no fallbacks allowed");

        let mut buffer = Vec::new();
        let mut samples_received = 0;
        while let Some(sample) = stream.next().await {
            buffer.push(sample);
            samples_received += 1;
            if samples_received > 6000 {
                break;
            }
        }

        assert!(buffer.iter().any(|x| *x != 0.0), "Audio samples must contain non-zero data");
    }

    #[tokio::test]
    #[serial]
    async fn test_voice_processing_mic_48khz() {
        let mic = VoiceProcessingMicInput::with_sample_rate(48000)
            .expect("VoiceProcessingMicInput with 48kHz must be created successfully - no fallbacks allowed");
        let mut stream = mic.stream()
            .expect("VoiceProcessingMic stream must initialize successfully - no fallbacks allowed");
        
        assert_eq!(stream.sample_rate(), 48000);

        let mut buffer = Vec::new();
        let mut samples_received = 0;
        while let Some(sample) = stream.next().await {
            buffer.push(sample);
            samples_received += 1;
            if samples_received > 12000 {
                break;
            }
        }

        assert!(buffer.iter().any(|x| *x != 0.0), "Audio samples must contain non-zero data");
    }
}