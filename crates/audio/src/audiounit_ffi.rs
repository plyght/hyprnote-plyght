use std::ffi::c_void;

use cidre::{cat, os};

// AudioUnit types and constants
pub type AudioUnit = *mut c_void;
pub type AudioComponentInstance = AudioUnit;
pub type AudioComponent = *mut c_void;

// AudioUnit scopes
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum AudioUnitScope {
    Global = 0,
    Input = 1,
    Output = 2,
}

// AudioUnit elements
pub const AU_INPUT_ELEMENT: u32 = 1;
pub const AU_OUTPUT_ELEMENT: u32 = 0;

// AudioUnit properties for VoiceProcessingIO
pub const K_AU_VOICE_IO_PROPERTY_VOICE_PROCESSING_ENABLE_AGC: u32 = 2010;
pub const K_AU_VOICE_IO_PROPERTY_VOICE_PROCESSING_ENABLE_NOISE_SUPPRESSION: u32 = 2011;
pub const K_AU_VOICE_IO_PROPERTY_VOICE_PROCESSING_ENABLE_ECHO_CANCEL: u32 = 2009;
pub const K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT: u32 = 8;
pub const K_AUDIO_OUTPUT_UNIT_PROPERTY_ENABLE_IO: u32 = 2003;

// AudioComponent description for VoiceProcessingIO
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AudioComponentDescription {
    pub component_type: u32,
    pub component_sub_type: u32,
    pub component_manufacturer: u32,
    pub component_flags: u32,
    pub component_flags_mask: u32,
}

// Component types and subtypes
pub const K_AUDIO_UNIT_TYPE_OUTPUT: u32 = 0x61756F75; // 'auou'
pub const K_AUDIO_UNIT_SUB_TYPE_VOICE_PROCESSING_IO: u32 = 0x7670696F; // 'vpio'
pub const K_AUDIO_UNIT_MANUFACTURER_APPLE: u32 = 0x6170706C; // 'appl'

// AudioUnit callback types
pub type AudioUnitRenderCallback = extern "C" fn(
    in_ref_con: *mut c_void,
    io_action_flags: *mut u32,
    in_time_stamp: *const cat::AudioTimeStamp,
    in_bus_number: u32,
    in_number_frames: u32,
    io_data: *mut cat::AudioBufList<1>,
) -> os::Status;

#[repr(C)]
pub struct AURenderCallbackStruct {
    pub input_proc: AudioUnitRenderCallback,
    pub input_proc_ref_con: *mut c_void,
}

// AudioUnit property IDs
pub const K_AUDIO_UNIT_PROPERTY_SET_RENDER_CALLBACK: u32 = 23;
pub const K_AUDIO_UNIT_PROPERTY_SET_INPUT_CALLBACK: u32 = 7;

// Error codes
pub const K_AUDIO_UNIT_ERR_INVALID_PARAMETER: i32 = -50;
pub const NO_ERR: i32 = 0;

// External AudioUnit functions
extern "C" {
    pub fn AudioComponentFindNext(
        in_component: AudioComponent,
        in_desc: *const AudioComponentDescription,
    ) -> AudioComponent;

    pub fn AudioComponentInstanceNew(
        in_component: AudioComponent,
        out_instance: *mut AudioComponentInstance,
    ) -> os::Status;

    pub fn AudioUnitInitialize(in_unit: AudioUnit) -> os::Status;

    pub fn AudioUnitUninitialize(in_unit: AudioUnit) -> os::Status;

    pub fn AudioOutputUnitStart(in_unit: AudioUnit) -> os::Status;

    pub fn AudioOutputUnitStop(in_unit: AudioUnit) -> os::Status;

    pub fn AudioUnitSetProperty(
        in_unit: AudioUnit,
        in_id: u32,
        in_scope: u32,
        in_element: u32,
        in_data: *const c_void,
        in_data_size: u32,
    ) -> os::Status;

    pub fn AudioUnitGetProperty(
        in_unit: AudioUnit,
        in_id: u32,
        in_scope: u32,
        in_element: u32,
        out_data: *mut c_void,
        io_data_size: *mut u32,
    ) -> os::Status;

    pub fn AudioUnitRender(
        in_unit: AudioUnit,
        io_action_flags: *mut u32,
        in_time_stamp: *const cat::AudioTimeStamp,
        in_output_bus_number: u32,
        in_number_frames: u32,
        io_data: *mut cat::AudioBufList<1>,
    ) -> os::Status;

    pub fn AudioComponentInstanceDispose(in_instance: AudioComponentInstance) -> os::Status;
}

// Helper functions
impl AudioComponentDescription {
    pub fn voice_processing_io() -> Self {
        Self {
            component_type: K_AUDIO_UNIT_TYPE_OUTPUT,
            component_sub_type: K_AUDIO_UNIT_SUB_TYPE_VOICE_PROCESSING_IO,
            component_manufacturer: K_AUDIO_UNIT_MANUFACTURER_APPLE,
            component_flags: 0,
            component_flags_mask: 0,
        }
    }
}

// Safe wrapper for AudioUnit
pub struct VoiceProcessingAudioUnit {
    unit: AudioUnit,
}

impl VoiceProcessingAudioUnit {
    pub fn new() -> Result<Self, os::Status> {
        tracing::info!("🔧 Creating VoiceProcessingIO AudioUnit...");
        
        let desc = AudioComponentDescription::voice_processing_io();
        tracing::info!(
            component_type = format!("{:#x}", desc.component_type),
            component_sub_type = format!("{:#x}", desc.component_sub_type), 
            component_manufacturer = format!("{:#x}", desc.component_manufacturer),
            "🔍 Searching for AudioUnit component"
        );
        
        let component = unsafe { AudioComponentFindNext(std::ptr::null_mut(), &desc) };
        if component.is_null() {
            tracing::error!("❌ VoiceProcessingIO AudioUnit component not found!");
            tracing::error!("This usually means:");
            tracing::error!("  1. Running on non-macOS system");
            tracing::error!("  2. VoiceProcessingIO not available on this macOS version");
            tracing::error!("  3. Audio permissions not granted");
            return Err(os::Status(-50)); // kAudioUnitErr_InvalidParameter
        }
        
        tracing::info!("✅ Found VoiceProcessingIO AudioUnit component");

        let mut unit: AudioUnit = std::ptr::null_mut();
        let status = unsafe { AudioComponentInstanceNew(component, &mut unit) };
        
        if status != os::Status::NO_ERR {
            tracing::error!(
                status = ?status,
                "❌ Failed to create AudioUnit instance"
            );
            return Err(status);
        }
        
        tracing::info!("✅ Successfully created VoiceProcessingIO AudioUnit instance");
        Ok(Self { unit })
    }

    pub fn enable_io(&self, scope: AudioUnitScope, element: u32, enable: bool) -> Result<(), os::Status> {
        let enable_val: u32 = if enable { 1 } else { 0 };
        
        tracing::info!(
            scope = ?scope,
            element = element,
            enable = enable,
            "🔧 Configuring AudioUnit I/O"
        );
        
        let status = unsafe {
            AudioUnitSetProperty(
                self.unit,
                K_AUDIO_OUTPUT_UNIT_PROPERTY_ENABLE_IO,
                scope as u32,
                element,
                &enable_val as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            )
        };

        if status == os::Status::NO_ERR {
            tracing::info!(
                scope = ?scope,
                element = element,
                enable = enable,
                "✅ AudioUnit I/O configured successfully"
            );
            Ok(())
        } else {
            tracing::error!(
                scope = ?scope,
                element = element,
                enable = enable,
                status = ?status,
                "❌ Failed to configure AudioUnit I/O"
            );
            Err(status)
        }
    }

    pub fn set_stream_format(&self, format: &cat::AudioBasicStreamDesc, scope: AudioUnitScope, element: u32) -> Result<(), os::Status> {
        tracing::info!(
            scope = ?scope,
            element = element,
            sample_rate = format.sample_rate,
            format = ?format.format,
            format_flags = ?format.format_flags,
            channels = format.channels_per_frame,
            bits_per_channel = format.bits_per_channel,
            bytes_per_frame = format.bytes_per_frame,
            "🔧 Setting AudioUnit stream format"
        );
        
        let status = unsafe {
            AudioUnitSetProperty(
                self.unit,
                K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT,
                scope as u32,
                element,
                format as *const cat::AudioBasicStreamDesc as *const c_void,
                std::mem::size_of::<cat::AudioBasicStreamDesc>() as u32,
            )
        };

        if status == os::Status::NO_ERR {
            tracing::info!(
                scope = ?scope,
                element = element,
                "✅ Stream format set successfully"
            );
            Ok(())
        } else {
            tracing::error!(
                scope = ?scope,
                element = element,
                status = ?status,
                status_code = status.0,
                "❌ Failed to set stream format"
            );
            
            // Try to provide helpful error information
            match status.0 {
                -10865 => tracing::error!("kAudioUnitErr_FormatNotSupported - The audio format is not supported by this AudioUnit"),
                -10866 => tracing::error!("kAudioUnitErr_InvalidProperty - Invalid property for this AudioUnit"),
                -50 => tracing::error!("kAudioUnitErr_InvalidParameter - Invalid parameter"),
                _ => tracing::error!("Unknown AudioUnit error: {}", status.0),
            }
            
            Err(status)
        }
    }

    pub fn enable_voice_processing_agc(&self, enable: bool) -> Result<(), os::Status> {
        let enable_val: u32 = if enable { 1 } else { 0 };
        
        tracing::info!(enable = enable, "🔧 Configuring Automatic Gain Control (AGC)");
        
        let status = unsafe {
            AudioUnitSetProperty(
                self.unit,
                K_AU_VOICE_IO_PROPERTY_VOICE_PROCESSING_ENABLE_AGC,
                AudioUnitScope::Global as u32,
                0,
                &enable_val as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            )
        };

        if status == os::Status::NO_ERR {
            if enable {
                tracing::info!("✅ AGC ENABLED - Apple hardware-accelerated automatic gain control active");
            } else {
                tracing::info!("ℹ️  AGC disabled");
            }
            Ok(())
        } else {
            tracing::error!(
                enable = enable,
                status = ?status,
                "❌ Failed to configure AGC - this feature may not be available"
            );
            Err(status)
        }
    }

    pub fn enable_voice_processing_noise_suppression(&self, enable: bool) -> Result<(), os::Status> {
        let enable_val: u32 = if enable { 1 } else { 0 };
        
        tracing::info!(enable = enable, "🔧 Configuring Noise Suppression");
        
        let status = unsafe {
            AudioUnitSetProperty(
                self.unit,
                K_AU_VOICE_IO_PROPERTY_VOICE_PROCESSING_ENABLE_NOISE_SUPPRESSION,
                AudioUnitScope::Global as u32,
                0,
                &enable_val as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            )
        };

        if status == os::Status::NO_ERR {
            if enable {
                tracing::info!("✅ NOISE SUPPRESSION ENABLED - Apple advanced noise suppression active");
            } else {
                tracing::info!("ℹ️  Noise suppression disabled");
            }
            Ok(())
        } else {
            tracing::error!(
                enable = enable,
                status = ?status,
                "❌ Failed to configure noise suppression - this feature may not be available"
            );
            Err(status)
        }
    }

    pub fn enable_voice_processing_echo_cancellation(&self, enable: bool) -> Result<(), os::Status> {
        let enable_val: u32 = if enable { 1 } else { 0 };
        
        tracing::info!(enable = enable, "🔧 Configuring Echo Cancellation");
        
        let status = unsafe {
            AudioUnitSetProperty(
                self.unit,
                K_AU_VOICE_IO_PROPERTY_VOICE_PROCESSING_ENABLE_ECHO_CANCEL,
                AudioUnitScope::Global as u32,
                0,
                &enable_val as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            )
        };

        if status == os::Status::NO_ERR {
            if enable {
                tracing::info!("✅ ECHO CANCELLATION ENABLED - Apple echo cancellation active (requires speaker reference)");
            } else {
                tracing::info!("ℹ️  Echo cancellation disabled");
            }
            Ok(())
        } else {
            tracing::error!(
                enable = enable,
                status = ?status,
                "❌ Failed to configure echo cancellation - this feature may not be available"
            );
            Err(status)
        }
    }

    pub fn set_input_callback(&self, callback: AudioUnitRenderCallback, user_data: *mut c_void) -> Result<(), os::Status> {
        let callback_struct = AURenderCallbackStruct {
            input_proc: callback,
            input_proc_ref_con: user_data,
        };

        tracing::info!("🔧 Setting input render callback for VoiceProcessingIO");

        let status = unsafe {
            AudioUnitSetProperty(
                self.unit,
                K_AUDIO_UNIT_PROPERTY_SET_RENDER_CALLBACK,
                AudioUnitScope::Input as u32,
                AU_INPUT_ELEMENT,
                &callback_struct as *const AURenderCallbackStruct as *const c_void,
                std::mem::size_of::<AURenderCallbackStruct>() as u32,
            )
        };

        if status == os::Status::NO_ERR {
            tracing::info!("✅ Input render callback set successfully");
            Ok(())
        } else {
            tracing::error!(
                status = ?status,
                status_code = status.0,
                "❌ Failed to set input render callback"
            );
            Err(status)
        }
    }

    pub fn check_property_support(&self, property_id: u32, scope: AudioUnitScope, element: u32) -> bool {
        let mut size: u32 = 0;
        let status = unsafe {
            AudioUnitGetProperty(
                self.unit,
                property_id,
                scope as u32,
                element,
                std::ptr::null_mut(),
                &mut size,
            )
        };
        status == os::Status::NO_ERR
    }

    pub fn initialize(&self) -> Result<(), os::Status> {
        let status = unsafe { AudioUnitInitialize(self.unit) };
        
        if status == os::Status::NO_ERR {
            Ok(())
        } else {
            Err(status)
        }
    }

    pub fn start(&self) -> Result<(), os::Status> {
        let status = unsafe { AudioOutputUnitStart(self.unit) };
        
        if status == os::Status::NO_ERR {
            Ok(())
        } else {
            Err(status)
        }
    }

    pub fn stop(&self) -> Result<(), os::Status> {
        let status = unsafe { AudioOutputUnitStop(self.unit) };
        
        if status == os::Status::NO_ERR {
            Ok(())
        } else {
            Err(status)
        }
    }

    pub fn render(&self, 
        action_flags: &mut u32,
        timestamp: &cat::AudioTimeStamp,
        bus_number: u32,
        number_frames: u32,
        data: &mut cat::AudioBufList<1>
    ) -> Result<(), os::Status> {
        let status = unsafe {
            AudioUnitRender(
                self.unit,
                action_flags,
                timestamp,
                bus_number,
                number_frames,
                data,
            )
        };

        if status == os::Status::NO_ERR {
            Ok(())
        } else {
            Err(status)
        }
    }

    pub fn raw_unit(&self) -> AudioUnit {
        self.unit
    }
}

impl Drop for VoiceProcessingAudioUnit {
    fn drop(&mut self) {
        if !self.unit.is_null() {
            unsafe {
                AudioOutputUnitStop(self.unit);
                AudioUnitUninitialize(self.unit);
                AudioComponentInstanceDispose(self.unit);
            }
        }
    }
}

// SAFETY: VoiceProcessingAudioUnit wraps an AudioUnit pointer that is safe to send between threads.
// The AudioUnit API is thread-safe for the operations we perform.
unsafe impl Send for VoiceProcessingAudioUnit {}
// SAFETY: VoiceProcessingAudioUnit can be safely shared between threads as all AudioUnit operations
// are protected by the framework's internal synchronization.
unsafe impl Sync for VoiceProcessingAudioUnit {}