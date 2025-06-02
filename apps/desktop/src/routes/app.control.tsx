import { createFileRoute } from "@tanstack/react-router";
import { Circle, Grip, Settings, Square, Mic, MicOff, Volume2, VolumeX } from "lucide-react";
import React, { useEffect, useRef, useState } from "react";
import { emit, listen } from "@tauri-apps/api/event";

import { commands as windowsCommands } from "@hypr/plugin-windows";
import { commands as listenerCommands, events as listenerEvents } from "@hypr/plugin-listener";

export const Route = createFileRoute("/app/control")({
  component: Component,
});

function Component() {
  const [position, setPosition] = useState(() => {
    const windowWidth = window.innerWidth;
    const windowHeight = window.innerHeight;
    const initialX = (windowWidth - 200) / 2;
    const initialY = (windowHeight - 200) / 2; // Center vertically for debugging
    
    return { x: initialX, y: initialY };
  });

  const [isDragging, setIsDragging] = useState(false);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });
  
  // Use refs to store current values for event handlers
  const isDraggingRef = useRef(false);
  const dragOffsetRef = useRef({ x: 0, y: 0 });
  
  // Update refs whenever state changes
  useEffect(() => {
    isDraggingRef.current = isDragging;
  }, [isDragging]);
  
  useEffect(() => {
    dragOffsetRef.current = dragOffset;
  }, [dragOffset]);
  
  // Recording state from listener plugin
  const [recordingStatus, setRecordingStatus] = useState<"inactive" | "running_active" | "running_paused">("inactive");
  const [recordingLoading, setRecordingLoading] = useState(false);
  
  // Audio controls state
  const [micMuted, setMicMuted] = useState(false);
  const [speakerMuted, setSpeakerMuted] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  
  // Settings toggles state
  const [autoStartRecording, setAutoStartRecording] = useState(() => {
    return localStorage.getItem('autoStartRecording') === 'true';
  });
  const [showAudioLevels, setShowAudioLevels] = useState(() => {
    return localStorage.getItem('showAudioLevels') !== 'false'; // default true
  });
  const [alwaysOnTop, setAlwaysOnTop] = useState(() => {
    return localStorage.getItem('alwaysOnTop') !== 'false'; // default true
  });
  
  const isRecording = recordingStatus !== "inactive";
  const isRecordingActive = recordingStatus === "running_active";
  const isRecordingPaused = recordingStatus === "running_paused";
  
  // Load initial recording state and listen for changes
  useEffect(() => {
    const initializeState = async () => {
      try {
        // Get initial state from listener plugin
        const currentState = await listenerCommands.getState();
        console.log(`[Control Bar] Initial state: ${currentState}`);
        
        if (currentState === "running_active" || currentState === "running_paused" || currentState === "inactive") {
          setRecordingStatus(currentState as any);
        }
        
        // Get initial audio state
        const [initialMicMuted, initialSpeakerMuted] = await Promise.all([
          listenerCommands.getMicMuted(),
          listenerCommands.getSpeakerMuted()
        ]);
        setMicMuted(initialMicMuted);
        setSpeakerMuted(initialSpeakerMuted);
      } catch (error) {
        console.error("[Control Bar] Failed to load initial state:", error);
      }
    };
    
    initializeState();
    
    // Listen for session events
    const unsubscribeSession = listenerEvents.sessionEvent.listen(({ payload }) => {
      console.log(`[Control Bar] Session event:`, payload);
      
      if (payload.type === "inactive" || payload.type === "running_active" || payload.type === "running_paused") {
        setRecordingStatus(payload.type);
        setRecordingLoading(false);
      }
    });
    
    // Listen for audio state changes from other windows
    const unsubscribeMicState = listen<{ muted: boolean }>("audio-mic-state-changed", ({ payload }) => {
      console.log(`[Control Bar] Received mic state change:`, payload);
      setMicMuted(payload.muted);
    });
    
    const unsubscribeSpeakerState = listen<{ muted: boolean }>("audio-speaker-state-changed", ({ payload }) => {
      console.log(`[Control Bar] Received speaker state change:`, payload);
      setSpeakerMuted(payload.muted);
    });
    
    return () => {
      Promise.all([
        unsubscribeSession.then(unlisten => unlisten()),
        unsubscribeMicState.then(unlisten => unlisten()),
        unsubscribeSpeakerState.then(unlisten => unlisten())
      ]).catch(error => {
        console.error("Error during cleanup:", error);
      });
    };
  }, []);
  
  // Debug logging
  useEffect(() => {
    console.log(`[Control Bar Debug] Recording status: ${recordingStatus}, isRecording: ${isRecording}, isRecordingActive: ${isRecordingActive}`);
  }, [recordingStatus, isRecording, isRecordingActive]);
  
  const controlRef = useRef<HTMLDivElement>(null);
  const toolbarRef = useRef<HTMLDivElement>(null);
  const settingsPopupRef = useRef<HTMLDivElement>(null);
  const updateScheduledRef = useRef(false);

  const updateOverlayBounds = async () => {
    if (toolbarRef.current) {
      const toolbarRect = toolbarRef.current.getBoundingClientRect();
      
      let bounds = {
        x: position.x,
        y: position.y, 
        width: toolbarRect.width,
        height: toolbarRect.height,
      };

      // If settings popup is open, calculate combined bounds
      if (showSettings) {
        // Calculate popup position manually based on how it's positioned in the component
        const isNearTop = position.y < 250;
        const popupTop = isNearTop ? position.y + 60 : position.y - 200;
        const popupLeft = position.x;
        const popupWidth = 256; // w-64 in Tailwind = 256px
        const popupHeight = 200; // Approximate height of the settings popup
        
        // Calculate the combined bounding box
        const minX = Math.min(position.x, popupLeft);
        const minY = Math.min(position.y, popupTop);
        const maxX = Math.max(position.x + toolbarRect.width, popupLeft + popupWidth);
        const maxY = Math.max(position.y + toolbarRect.height, popupTop + popupHeight);
        
        bounds = {
          x: minX,
          y: minY,
          width: maxX - minX,
          height: maxY - minY,
        };
        
      }
      
      try {
        await windowsCommands.setFakeWindowBounds("control", bounds);
      } catch (error) {
        console.error("Failed to set fake window bounds:", error);
      }
    }
  };

  const handleToolbarClick = (e: React.MouseEvent) => {
    // Don't stop propagation to allow drag events to work properly
  };

  useEffect(() => {
    // Immediately set transparent background to prevent white flash
    document.body.style.background = "transparent";
    document.body.style.backgroundColor = "transparent";
    document.documentElement.style.background = "transparent";
    document.documentElement.style.backgroundColor = "transparent";
    document.documentElement.setAttribute("data-transparent-window", "true");

    const handleMouseMove = (e: MouseEvent) => {
      if (!isDraggingRef.current) return;
      
      // Get toolbar dimensions for clamping
      const toolbarWidth = toolbarRef.current?.getBoundingClientRect().width || 200;
      const toolbarHeight = toolbarRef.current?.getBoundingClientRect().height || 60;
      
      // Clamp position to keep toolbar on screen
      const clampedX = Math.max(0, Math.min(window.innerWidth - toolbarWidth, e.clientX - dragOffsetRef.current.x));
      const clampedY = Math.max(0, Math.min(window.innerHeight - toolbarHeight, e.clientY - dragOffsetRef.current.y));
      
      const newPosition = {
        x: clampedX,
        y: clampedY,
      };
      
      setPosition(newPosition);
      
      // Simple throttling - update bounds every few frames during drag
      if (!updateScheduledRef.current) {
        updateScheduledRef.current = true;
        setTimeout(() => {
          updateOverlayBounds();
          updateScheduledRef.current = false;
        }, 16); // ~60fps
      }
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      // Immediately update bounds when drag ends to ensure toolbar stays responsive
      setTimeout(() => {
        updateOverlayBounds();
      }, 10);
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    
    updateOverlayBounds();

    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
      windowsCommands.removeFakeWindow("control");
    };
  }, []); // Remove dependencies to prevent re-creating event listeners

  useEffect(() => {
    // Update bounds whenever position changes (safety mechanism)
    const timer = setTimeout(() => {
      updateOverlayBounds();
    }, 50);
    return () => clearTimeout(timer);
  }, [position]);

  // Separate effect for settings popup to ensure it's rendered
  useEffect(() => {
    if (showSettings) {
      // Wait for popup to be rendered and ref to be available
      const timer = setTimeout(() => {
        updateOverlayBounds();
      }, 50);
      return () => clearTimeout(timer);
    } else {
      // Add delay when popup closes to ensure DOM has updated
      const timer = setTimeout(() => {
        updateOverlayBounds();
      }, 50);
      return () => clearTimeout(timer);
    }
  }, [showSettings]);

  // Also update bounds after initial render
  useEffect(() => {
    const timer = setTimeout(() => {
      updateOverlayBounds();
    }, 100);
    return () => clearTimeout(timer);
  }, []);

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    
    setIsDragging(true);
    setDragOffset({
      x: e.clientX - position.x,
      y: e.clientY - position.y,
    });
  };


  const toggleRecording = async () => {
    try {
      setRecordingLoading(true);
      
      if (isRecording) {
        if (isRecordingActive) {
          await listenerCommands.stopSession();
        } else if (isRecordingPaused) {
          await listenerCommands.resumeSession();
        }
      } else {
        // Create a new session and start recording
        const newSessionId = `control-session-${Date.now()}`;
        await listenerCommands.startSession(newSessionId);
      }
    } catch (error) {
      console.error("[Control Bar] Recording error:", error);
    } finally {
      setRecordingLoading(false);
    }
    if (!updateScheduledRef.current) {
      updateScheduledRef.current = true;
      requestAnimationFrame(() => {
        updateOverlayBounds();
        updateScheduledRef.current = false;
      });
    }
  };
  
  const pauseRecording = async () => {
    try {
      setRecordingLoading(true);
      if (isRecordingActive) {
        await listenerCommands.pauseSession();
      }
    } catch (error) {
      console.error("[Control Bar] Pause error:", error);
    } finally {
      setRecordingLoading(false);
    }
    if (!updateScheduledRef.current) {
      updateScheduledRef.current = true;
      requestAnimationFrame(() => {
        updateOverlayBounds();
        updateScheduledRef.current = false;
      });
    }
  };
  
  const toggleMic = async () => {
    try {
      const newMuted = !micMuted;
      await listenerCommands.setMicMuted(newMuted);
      setMicMuted(newMuted);
      // Emit event to synchronize with other windows
      await emit("audio-mic-state-changed", { muted: newMuted });
    } catch (error) {
      console.error("[Control Bar] Mic toggle error:", error);
    }
  };
  
  const toggleSpeaker = async () => {
    try {
      const newMuted = !speakerMuted;
      await listenerCommands.setSpeakerMuted(newMuted);
      setSpeakerMuted(newMuted);
      // Emit event to synchronize with other windows
      await emit("audio-speaker-state-changed", { muted: newMuted });
    } catch (error) {
      console.error("[Control Bar] Speaker toggle error:", error);
    }
  };

  const openSettings = () => {
    setShowSettings(!showSettings);
  };

  const toggleAutoStart = () => {
    const newValue = !autoStartRecording;
    setAutoStartRecording(newValue);
    localStorage.setItem('autoStartRecording', newValue.toString());
  };

  const toggleAudioLevels = () => {
    const newValue = !showAudioLevels;
    setShowAudioLevels(newValue);
    localStorage.setItem('showAudioLevels', newValue.toString());
  };

  const toggleAlwaysOnTop = () => {
    const newValue = !alwaysOnTop;
    setAlwaysOnTop(newValue);
    localStorage.setItem('alwaysOnTop', newValue.toString());
  };
  

  return (
    <div
      className="w-screen h-[100vh] relative overflow-y-hidden"
      style={{ 
        scrollbarColor: "auto transparent",
        background: "transparent",
        backgroundColor: "transparent"
      }}
    >
      <div
        className="absolute"
        style={{
          left: position.x,
          top: position.y,
          transition: isDragging ? "none" : "all 0.1s ease",
        }}
        ref={controlRef}
      >
        <div
          className="rounded-2xl shadow-2xl flex items-center justify-center transition-all duration-200 p-3"
          ref={toolbarRef}
          onClick={handleToolbarClick}
          style={{ 
            pointerEvents: 'auto',
            background: 'rgba(0, 0, 0, 0.85)',
            boxShadow: '0 8px 32px 0 rgba(0, 0, 0, 0.6)'
          }}
        >
          <div className="flex gap-2 items-center">
            {/* Section 1: Mic + Speaker */}
            <div className="flex gap-1 items-center">
              <IconButton
                onClick={toggleMic}
                tooltip={micMuted ? "Unmute Microphone" : "Mute Microphone"}
                className={micMuted ? "bg-red-500/60 hover:bg-red-500/80" : "bg-gray-700/60 hover:bg-gray-600/80"}
              >
                {micMuted ? <MicOff size={16} /> : <Mic size={16} />}
              </IconButton>
              
              <IconButton
                onClick={toggleSpeaker}
                tooltip={speakerMuted ? "Unmute Speaker" : "Mute Speaker"}
                className={speakerMuted ? "bg-red-500/60 hover:bg-red-500/80" : "bg-gray-700/60 hover:bg-gray-600/80"}
              >
                {speakerMuted ? <VolumeX size={16} /> : <Volume2 size={16} />}
              </IconButton>
            </div>
            
            <div className="w-px h-6 bg-white/20 mx-1" />
            
            {/* Section 2: Pause + Stop */}
            <div className="flex gap-1 items-center">
              {/* Pause/Resume Button */}
              {isRecording && (
                <IconButton
                  onClick={isRecordingActive ? pauseRecording : toggleRecording}
                  tooltip={isRecordingActive ? "Pause Recording" : "Resume Recording"}
                  className={isRecordingActive 
                    ? "bg-amber-600/60 hover:bg-amber-500/80" 
                    : "bg-green-600/60 hover:bg-green-500/80"
                  }
                  disabled={recordingLoading}
                >
                  {recordingLoading ? (
                    <div className="animate-spin w-4 h-4 border-2 border-white/30 border-t-white rounded-full" />
                  ) : isRecordingActive ? (
                    <div className="flex gap-0.5">
                      <div className="w-1 h-3 bg-white rounded-sm" />
                      <div className="w-1 h-3 bg-white rounded-sm" />
                    </div>
                  ) : (
                    <Circle size={16} />
                  )}
                </IconButton>
              )}
              
              {/* Stop/Start Button */}
              <IconButton
                onClick={toggleRecording}
                tooltip={isRecording ? "Stop Recording" : "Start Recording"}
                className={isRecording 
                  ? "bg-red-600/70 hover:bg-red-500/90 shadow-lg shadow-red-500/30" 
                  : "bg-gray-700/60 hover:bg-gray-600/80"
                }
                disabled={recordingLoading}
              >
                {recordingLoading ? (
                  <div className="animate-spin w-4 h-4 border-2 border-white/30 border-t-white rounded-full" />
                ) : isRecording ? (
                  <Square size={16} />
                ) : (
                  <Circle size={16} />
                )}
              </IconButton>
            </div>
            
            <div className="w-px h-6 bg-white/20 mx-1" />
            
            {/* Section 3: Settings + Drag Handle */}
            <div className="flex gap-1 items-center">
              <IconButton onClick={openSettings} tooltip="Settings" className="bg-gray-700/60 hover:bg-gray-600/80">
                <Settings size={16} />
              </IconButton>
              
              <div
                className="ml-1 p-1.5 text-white/60 cursor-move hover:text-white/90 hover:bg-gray-600/40 rounded-lg transition-all duration-200"
                onMouseDown={handleMouseDown}
                title="Drag to move"
                style={{ userSelect: 'none' }}
              >
                <Grip size={16} />
              </div>
            </div>
          </div>
        </div>
      </div>
      
      {/* Settings Popup */}
      <SettingsPopup 
        ref={settingsPopupRef}
        isOpen={showSettings} 
        onClose={() => setShowSettings(false)}
        position={position}
        autoStartRecording={autoStartRecording}
        showAudioLevels={showAudioLevels}
        alwaysOnTop={alwaysOnTop}
        toggleAutoStart={toggleAutoStart}
        toggleAudioLevels={toggleAudioLevels}
        toggleAlwaysOnTop={toggleAlwaysOnTop}
      />
    </div>
  );
}

function IconButton({ onClick, children, className = "", tooltip = "", disabled = false }: {
  onClick?: ((e: React.MouseEvent<HTMLButtonElement>) => void) | (() => void);
  children: React.ReactNode;
  className?: string;
  tooltip?: string;
  disabled?: boolean;
}) {
  const handleClick = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation(); // Prevent button clicks from triggering drag
    if (!disabled) {
      onClick?.(e);
    }
  };

  return (
    <button
      onClick={handleClick}
      disabled={disabled}
      className={`p-2 bg-gray-800/50 backdrop-blur-sm rounded-xl text-white shadow-lg hover:bg-gray-700/60 active:bg-gray-600/70 transition-all duration-200 flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed border border-gray-600/30 hover:border-gray-500/40 ${className}`}
      title={tooltip}
      aria-label={tooltip}
    >
      {children}
    </button>
  );
}

// Settings popup component
const SettingsPopup = React.forwardRef<HTMLDivElement, {
  isOpen: boolean;
  onClose: () => void;
  position: { x: number; y: number };
  autoStartRecording: boolean;
  showAudioLevels: boolean;
  alwaysOnTop: boolean;
  toggleAutoStart: () => void;
  toggleAudioLevels: () => void;
  toggleAlwaysOnTop: () => void;
}>(({ 
  isOpen, 
  onClose, 
  position, 
  autoStartRecording,
  showAudioLevels,
  alwaysOnTop,
  toggleAutoStart,
  toggleAudioLevels,
  toggleAlwaysOnTop
}, ref) => {
  if (!isOpen) return null;

  // Smart positioning - open downwards if close to top, upwards otherwise
  const isNearTop = position.y < 250; // Within 250px of top
  const popupTop = isNearTop ? position.y + 60 : position.y - 200; // 60px below control bar or 200px above

  return (
    <div
      ref={ref}
      className="absolute z-50"
      style={{
        left: position.x,
        top: popupTop,
      }}
    >
      <div className="rounded-2xl shadow-2xl p-4 w-64"
        style={{
          pointerEvents: 'auto',
          background: 'rgba(0, 0, 0, 0.85)',
          boxShadow: '0 8px 32px 0 rgba(0, 0, 0, 0.6)'
        }}
      >
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-white font-medium">Recording Settings</h3>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onClose();
            }}
            className="text-white/60 hover:text-white/90 transition-colors p-1 hover:bg-white/10 rounded"
          >
            ×
          </button>
        </div>
        
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-white/80 text-sm">Auto-start recording</span>
            <button
              onClick={toggleAutoStart}
              className={`w-10 h-6 rounded-full relative transition-colors ${
                autoStartRecording ? 'bg-blue-500/60' : 'bg-white/20'
              }`}
            >
              <div className={`w-4 h-4 bg-white rounded-full absolute top-1 transition-transform ${
                autoStartRecording ? 'translate-x-4' : 'translate-x-0'
              }`} />
            </button>
          </div>
          
          <div className="flex items-center justify-between">
            <span className="text-white/80 text-sm">Show audio levels</span>
            <button
              onClick={toggleAudioLevels}
              className={`w-10 h-6 rounded-full relative transition-colors ${
                showAudioLevels ? 'bg-blue-500/60' : 'bg-white/20'
              }`}
            >
              <div className={`w-4 h-4 bg-white rounded-full absolute top-1 transition-transform ${
                showAudioLevels ? 'translate-x-4' : 'translate-x-0'
              }`} />
            </button>
          </div>
          
          <div className="flex items-center justify-between">
            <span className="text-white/80 text-sm">Always on top</span>
            <button
              onClick={toggleAlwaysOnTop}
              className={`w-10 h-6 rounded-full relative transition-colors ${
                alwaysOnTop ? 'bg-blue-500/60' : 'bg-white/20'
              }`}
            >
              <div className={`w-4 h-4 bg-white rounded-full absolute top-1 transition-transform ${
                alwaysOnTop ? 'translate-x-4' : 'translate-x-0'
              }`} />
            </button>
          </div>
          
          <div className="pt-2 border-t border-white/10">
            <button 
              onClick={async () => {
                await windowsCommands.windowShow({type:"settings"});
                onClose();
              }}
              className="w-full bg-white/15 hover:bg-white/25 text-white text-sm py-2 px-3 rounded-lg transition-colors"
            >
              Open Main Settings
            </button>
          </div>
        </div>
      </div>
    </div>
  );
});

SettingsPopup.displayName = "SettingsPopup";
