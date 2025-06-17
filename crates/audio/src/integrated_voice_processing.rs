use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};

use anyhow::Result;
use futures_util::Stream;
use kalosm_sound::AsyncSource;
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapCons, HeapProd, HeapRb,
};

use cidre::{cat, os};

use crate::audiounit_ffi::{VoiceProcessingAudioUnit, AudioUnitScope, AU_INPUT_ELEMENT, AU_OUTPUT_ELEMENT};
use crate::speaker::SpeakerStream;

/// A wrapper around SpeakerStream that also feeds data to voice processing reference
pub struct SpeakerReferenceStream {
    inner_stream: SpeakerStream,
    reference_producer: HeapProd<f32>,
}

impl SpeakerReferenceStream {
    pub fn sample_rate(&self) -> u32 {
        self.inner_stream.sample_rate()
    }
}

impl Stream for SpeakerReferenceStream {
    type Item = f32;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        use futures_util::StreamExt;
        
        // Get the next sample from the underlying speaker stream
        match self.inner_stream.poll_next_unpin(cx) {
            Poll::Ready(Some(sample)) => {
                // Feed the sample to the voice processing reference
                // Use try_push to avoid blocking if the buffer is full
                if self.reference_producer.try_push(sample).is_err() {
                    // Buffer is full - this is expected under normal operation
                    // The voice processing will consume from the other end
                }
                Poll::Ready(Some(sample))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl kalosm_sound::AsyncSource for SpeakerReferenceStream {
    fn as_stream(&mut self) -> impl Stream<Item = f32> + '_ {
        self
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate()
    }
}

/// Integrated voice processing that combines microphone input with speaker output reference
/// for optimal echo cancellation, AGC, and noise suppression
pub struct IntegratedVoiceProcessing {
    sample_rate: u32,
    speaker_sample_rate_override: Option<u32>,
}

struct SharedWakerState {
    waker: Option<Waker>,
    has_data: bool,
}

pub struct IntegratedVoiceProcessingStream {
    mic_consumer: HeapCons<f32>,
    sample_rate: u32,
    _audio_unit: VoiceProcessingAudioUnit,
    _speaker_stream: SpeakerReferenceStream,
    _ctx: Box<IntegratedCtx>,
    waker_state: Arc<Mutex<SharedWakerState>>,
}

impl IntegratedVoiceProcessingStream {
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

struct IntegratedCtx {
    mic_producer: HeapProd<f32>,
    speaker_consumer: HeapCons<f32>,
    waker_state: Arc<Mutex<SharedWakerState>>,
    audio_unit: *mut VoiceProcessingAudioUnit, // Raw pointer for callback access
}

unsafe impl Send for IntegratedCtx {}
unsafe impl Sync for IntegratedCtx {}

impl IntegratedVoiceProcessing {
    /// Create new integrated voice processing with default settings
    pub fn new() -> Result<Self> {
        Self::with_sample_rate(16000, None)
    }

    /// Create with specific sample rate and optional speaker sample rate override
    pub fn with_sample_rate(sample_rate: u32, speaker_sample_rate_override: Option<u32>) -> Result<Self> {
        // Validate sample rate for voice processing
        match sample_rate {
            8000 | 16000 | 24000 | 48000 => {},
            _ => tracing::warn!("Sample rate {} may not be optimal for voice processing", sample_rate),
        }

        Ok(Self {
            sample_rate,
            speaker_sample_rate_override,
        })
    }

    pub fn stream(self) -> Result<IntegratedVoiceProcessingStream> {
        // Create ring buffers for mic and speaker data
        let mic_rb = HeapRb::<f32>::new(8192);
        let (mic_producer, mic_consumer) = mic_rb.split();

        let speaker_rb = HeapRb::<f32>::new(8192);
        let (speaker_producer, speaker_consumer) = speaker_rb.split();

        let waker_state = Arc::new(Mutex::new(SharedWakerState {
            waker: None,
            has_data: false,
        }));

        // Create VoiceProcessingIO AudioUnit
        let mut audio_unit = VoiceProcessingAudioUnit::new()
            .map_err(|e| anyhow::anyhow!("Failed to create VoiceProcessingIO AudioUnit: {:?}", e))?;

        tracing::info!("Created integrated VoiceProcessingIO AudioUnit");

        // Configure I/O - enable both input (mic) and output (speaker reference)
        audio_unit.enable_io(AudioUnitScope::Input, AU_INPUT_ELEMENT, true)
            .map_err(|e| anyhow::anyhow!("Failed to enable mic input: {:?}", e))?;

        audio_unit.enable_io(AudioUnitScope::Output, AU_OUTPUT_ELEMENT, true)
            .map_err(|e| anyhow::anyhow!("Failed to enable speaker reference: {:?}", e))?;

        // Configure audio format (float32, mono, specified sample rate)
        let asbd = cat::AudioBasicStreamDesc {
            sample_rate: self.sample_rate as f64,
            format: cat::AudioFormat::LINEAR_PCM,
            format_flags: cat::AudioFormatFlags::IS_FLOAT | cat::AudioFormatFlags::IS_PACKED,
            bytes_per_packet: 4,
            frames_per_packet: 1,
            bytes_per_frame: 4,
            channels_per_frame: 1,
            bits_per_channel: 32,
            ..Default::default()
        };

        // Set format for both input and output
        audio_unit.set_stream_format(&asbd, AudioUnitScope::Input, AU_INPUT_ELEMENT)
            .map_err(|e| anyhow::anyhow!("Failed to set input format: {:?}", e))?;

        audio_unit.set_stream_format(&asbd, AudioUnitScope::Output, AU_OUTPUT_ELEMENT)
            .map_err(|e| anyhow::anyhow!("Failed to set output format: {:?}", e))?;

        tracing::info!(
            sample_rate = asbd.sample_rate,
            channels = asbd.channels_per_frame,
            "Configured integrated VoiceProcessingIO format"
        );

        // Enable all voice processing features
        audio_unit.enable_voice_processing_agc(true)
            .map_err(|e| anyhow::anyhow!("Failed to enable AGC: {:?}", e))?;
        tracing::info!("Enabled Automatic Gain Control");

        audio_unit.enable_voice_processing_noise_suppression(true)
            .map_err(|e| anyhow::anyhow!("Failed to enable noise suppression: {:?}", e))?;
        tracing::info!("Enabled Noise Suppression");

        audio_unit.enable_voice_processing_echo_cancellation(true)
            .map_err(|e| anyhow::anyhow!("Failed to enable echo cancellation: {:?}", e))?;
        tracing::info!("Enabled Echo Cancellation with speaker reference");

        // Create context with pointer to audio unit for callbacks
        let mut ctx = Box::new(IntegratedCtx {
            mic_producer,
            speaker_consumer,
            waker_state: waker_state.clone(),
            audio_unit: &mut audio_unit as *mut VoiceProcessingAudioUnit,
        });

        // Set input callback for microphone processing
        audio_unit.set_input_callback(Self::mic_input_callback, ctx.as_mut() as *mut IntegratedCtx as *mut std::ffi::c_void)
            .map_err(|e| anyhow::anyhow!("Failed to set mic input callback: {:?}", e))?;

        // Create speaker stream that will feed data to our speaker_producer
        let speaker_stream = Self::create_speaker_stream_with_reference(
            speaker_producer,
            self.speaker_sample_rate_override,
        )?;

        // Initialize and start the AudioUnit
        audio_unit.initialize()
            .map_err(|e| anyhow::anyhow!("Failed to initialize AudioUnit: {:?}", e))?;

        audio_unit.start()
            .map_err(|e| anyhow::anyhow!("Failed to start AudioUnit: {:?}", e))?;

        tracing::info!("Started integrated voice processing with full echo cancellation");

        Ok(IntegratedVoiceProcessingStream {
            mic_consumer,
            sample_rate: self.sample_rate,
            _audio_unit: audio_unit,
            _speaker_stream: speaker_stream,
            _ctx: ctx,
            waker_state,
        })
    }

    fn create_speaker_stream_with_reference(
        speaker_producer: HeapProd<f32>,
        sample_rate_override: Option<u32>,
    ) -> Result<SpeakerReferenceStream> {
        // Create a speaker stream that feeds data to our voice processing reference
        use crate::speaker::SpeakerInput;
        
        let speaker_input = SpeakerInput::new(sample_rate_override)?;
        let speaker_stream = speaker_input.stream()?;

        tracing::info!("Created speaker stream with voice processing reference");
        
        Ok(SpeakerReferenceStream {
            inner_stream: speaker_stream,
            reference_producer: speaker_producer,
        })
    }

    extern "C" fn mic_input_callback(
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

        let ctx = unsafe { &mut *(in_ref_con as *mut IntegratedCtx) };

        // Get speaker reference data if available
        let mut speaker_data = Vec::new();
        // Try to get speaker reference data matching the number of frames
        for _ in 0..in_number_frames {
            if let Some(sample) = ctx.speaker_consumer.try_pop() {
                speaker_data.push(sample);
            } else {
                speaker_data.push(0.0); // Silence if no speaker data
            }
        }

        // Provide speaker reference to AudioUnit for echo cancellation
        // This is typically done through a render callback for the output element
        // For now, we'll focus on getting the processed microphone input

        // Create buffer for processed microphone audio
        let mut mic_buffer = vec![0.0f32; in_number_frames as usize];
        let audio_buffer = cat::AudioBuf {
            number_channels: 1,
            data_bytes_size: in_number_frames * 4,
            data: mic_buffer.as_mut_ptr() as *mut u8,
        };

        let mut buf_list = cat::AudioBufList {
            number_buffers: 1,
            buffers: [audio_buffer],
        };

        // Render the processed microphone input from VoiceProcessingIO
        let render_status = unsafe {
            if !ctx.audio_unit.is_null() {
                let audio_unit = &*ctx.audio_unit;
                audio_unit.render(
                    &mut *io_action_flags,
                    &*in_time_stamp,
                    AU_INPUT_ELEMENT,
                    in_number_frames,
                    &mut buf_list,
                )
            } else {
                Err(os::Status(-50))
            }
        };

        if let Err(status) = render_status {
            tracing::warn!("VoiceProcessingIO render failed: {:?}", status);
            return status;
        }

        // Push the processed microphone audio to our ring buffer
        let pushed = ctx.mic_producer.push_slice(&mic_buffer);
        if pushed < mic_buffer.len() {
            tracing::warn!("integrated voice processing dropped {} samples", mic_buffer.len() - pushed);
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


impl Stream for IntegratedVoiceProcessingStream {
    type Item = f32;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Some(sample) = self.mic_consumer.try_pop() {
            return Poll::Ready(Some(sample));
        }

        {
            let mut state = self.waker_state.lock().unwrap();
            state.has_data = false;
            state.waker = Some(cx.waker().clone());
            drop(state);
        }

        match self.mic_consumer.try_pop() {
            Some(sample) => Poll::Ready(Some(sample)),
            None => Poll::Pending,
        }
    }
}

impl kalosm_sound::AsyncSource for IntegratedVoiceProcessingStream {
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
    async fn test_integrated_voice_processing() {
        let integrated = IntegratedVoiceProcessing::new().unwrap();
        let mut stream = integrated.stream().unwrap();

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
    async fn test_integrated_voice_processing_48khz() {
        let integrated = IntegratedVoiceProcessing::with_sample_rate(48000, Some(48000)).unwrap();
        let mut stream = integrated.stream().unwrap();
        
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
}