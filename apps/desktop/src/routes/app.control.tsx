import { createFileRoute } from "@tanstack/react-router";
import { Camera, Circle, Grip, Settings, Square, Mic, MicOff, Volume2, VolumeX } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { emit } from "@tauri-apps/api/event";

import { commands as windowsCommands } from "@hypr/plugin-windows";
import { commands as listenerCommands, events as listenerEvents } from "@hypr/plugin-listener";

export const Route = createFileRoute("/app/control")({
  component: Component,
});

function Component() {
  emit("debug", "Control component mounted");

  const [position, setPosition] = useState(() => {
    const windowWidth = window.innerWidth;
    const initialX = (windowWidth - 200) / 2;
    return { x: initialX, y: window.innerHeight - 80 };
  });

  const [isDragging, setIsDragging] = useState(false);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });
  
  // Recording state from listener plugin
  const [recordingStatus, setRecordingStatus] = useState<"inactive" | "running_active" | "running_paused">("inactive");
  const [recordingLoading, setRecordingLoading] = useState(false);
  
  // Audio controls state
  const [micMuted, setMicMuted] = useState(false);
  const [speakerMuted, setSpeakerMuted] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  
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
    const unsubscribe = listenerEvents.sessionEvent.listen(({ payload }) => {
      console.log(`[Control Bar] Session event:`, payload);
      
      if (payload.type === "inactive" || payload.type === "running_active" || payload.type === "running_paused") {
        setRecordingStatus(payload.type);
        setRecordingLoading(false);
      }
    });
    
    return () => {
      unsubscribe.then(unlisten => unlisten());
    };
  }, []);
  
  // Debug logging
  useEffect(() => {
    console.log(`[Control Bar Debug] Recording status: ${recordingStatus}, isRecording: ${isRecording}, isRecordingActive: ${isRecordingActive}`);
  }, [recordingStatus, isRecording, isRecordingActive]);
  
  const controlRef = useRef<HTMLDivElement>(null);
  const toolbarRef = useRef<HTMLDivElement>(null);

  const updateOverlayBounds = async () => {
    emit("debug", "updateOverlayBounds called");
    emit("debug", `toolbarRef.current: ${toolbarRef.current ? 'exists' : 'null'}`);
    
    if (toolbarRef.current) {
      const rect = toolbarRef.current.getBoundingClientRect();
      const bounds = {
        x: position.x,
        y: position.y, 
        width: rect.width,
        height: rect.height,
      };
      emit("debug", `Toolbar position: ${JSON.stringify(position)}`);
      emit("debug", `Toolbar rect: ${JSON.stringify({x: rect.x, y: rect.y, width: rect.width, height: rect.height})}`);
      emit("debug", `Setting overlay bounds: ${JSON.stringify(bounds)}`);
      emit("debug", `Window dimensions: ${JSON.stringify({ width: window.innerWidth, height: window.innerHeight })}`);
      
      try {
        await windowsCommands.setFakeWindowBounds("control", bounds);
        emit("debug", "setFakeWindowBounds completed successfully");
      } catch (error) {
        emit("debug", `setFakeWindowBounds failed: ${error}`);
      }
    } else {
      emit("debug", "toolbarRef.current is null, skipping bounds update");
    }
  };

  // Add click handler to test if the fake window bounds are working
  const handleToolbarClick = (e: React.MouseEvent) => {
    emit("debug", `Toolbar clicked at: ${JSON.stringify({ x: e.clientX, y: e.clientY })}`);
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
      if (isDragging) {
        const newPosition = {
          x: e.clientX - dragOffset.x,
          y: e.clientY - dragOffset.y,
        };
        setPosition(newPosition);
        // Update bounds immediately during drag for smooth interaction
        setTimeout(updateOverlayBounds, 0);
      }
    };

    const handleMouseUp = () => {
      setIsDragging(false);
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);

    updateOverlayBounds();

    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
      windowsCommands.removeFakeWindow("control");
    };
  }, [isDragging, dragOffset]);

  useEffect(() => {
    // Update bounds whenever position changes
    updateOverlayBounds();
  }, [position]);

  // Also update bounds after initial render
  useEffect(() => {
    emit("debug", "Initial useEffect running");
    emit("debug", `windowsCommands available: ${!!windowsCommands}`);
    emit("debug", `windowsCommands.setFakeWindowBounds available: ${!!windowsCommands.setFakeWindowBounds}`);
    
    const timer = setTimeout(() => {
      emit("debug", "Timer fired, calling updateOverlayBounds");
      updateOverlayBounds();
    }, 100);
    return () => clearTimeout(timer);
  }, []);

  const handleMouseDown = (e: React.MouseEvent) => {
    setIsDragging(true);
    setDragOffset({
      x: e.clientX - position.x,
      y: e.clientY - position.y,
    });
  };

  const captureScreenshot = () => {
    emit("debug", "Capture screenshot");
    setTimeout(updateOverlayBounds, 0);
  };

  const toggleRecording = async () => {
    try {
      setRecordingLoading(true);
      
      if (isRecording) {
        if (isRecordingActive) {
          await listenerCommands.stopSession();
          console.log("[Control Bar] Stopped recording");
        } else if (isRecordingPaused) {
          await listenerCommands.resumeSession();
          console.log("[Control Bar] Resumed recording");
        }
      } else {
        // Create a new session and start recording
        const newSessionId = `control-session-${Date.now()}`;
        await listenerCommands.startSession(newSessionId);
        console.log(`[Control Bar] Started recording with session: ${newSessionId}`);
      }
    } catch (error) {
      console.error("[Control Bar] Recording error:", error);
    } finally {
      setRecordingLoading(false);
    }
    setTimeout(updateOverlayBounds, 0);
  };
  
  const pauseRecording = async () => {
    try {
      setRecordingLoading(true);
      if (isRecordingActive) {
        await listenerCommands.pauseSession();
        console.log("[Control Bar] Paused recording");
      }
    } catch (error) {
      console.error("[Control Bar] Pause error:", error);
    } finally {
      setRecordingLoading(false);
    }
    setTimeout(updateOverlayBounds, 0);
  };
  
  const toggleMic = async () => {
    try {
      const newMuted = !micMuted;
      await listenerCommands.setMicMuted(newMuted);
      setMicMuted(newMuted);
      console.log(`[Control Bar] ${newMuted ? "Muted" : "Unmuted"} microphone`);
    } catch (error) {
      console.error("[Control Bar] Mic toggle error:", error);
    }
  };
  
  const toggleSpeaker = async () => {
    try {
      const newMuted = !speakerMuted;
      await listenerCommands.setSpeakerMuted(newMuted);
      setSpeakerMuted(newMuted);
      console.log(`[Control Bar] ${newMuted ? "Muted" : "Unmuted"} speaker`);
    } catch (error) {
      console.error("[Control Bar] Speaker toggle error:", error);
    }
  };

  const openSettings = () => {
    setShowSettings(!showSettings);
    console.log(`[Control Bar] ${showSettings ? "Closed" : "Opened"} settings`);
    setTimeout(updateOverlayBounds, 0);
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
            <IconButton onClick={captureScreenshot} tooltip="Take Screenshot">
              <Camera size={16} />
            </IconButton>
            
            <IconButton
              onClick={toggleRecording}
              tooltip={isRecording ? (isRecordingActive ? "Stop Recording" : "Resume Recording") : "Start Recording"}
              className={`transition-all duration-200 ${
                isRecordingActive 
                  ? "bg-red-500/70 hover:bg-red-500/90 shadow-lg shadow-red-500/30" 
                  : isRecordingPaused
                  ? "bg-yellow-500/70 hover:bg-yellow-500/90 shadow-lg shadow-yellow-500/30"
                  : "bg-white/20 hover:bg-white/30"
              }`}
              disabled={recordingLoading}
            >
              {recordingLoading ? (
                <div className="animate-spin w-4 h-4 border-2 border-white/30 border-t-white rounded-full" />
              ) : isRecordingActive ? (
                <Square size={16} />
              ) : (
                <Circle size={16} />
              )}
            </IconButton>
            
            {/* Pause Button - only show when actively recording */}
            {isRecordingActive && (
              <IconButton
                onClick={pauseRecording}
                tooltip="Pause Recording"
                className="bg-yellow-500/60 hover:bg-yellow-500/80 shadow-lg shadow-yellow-500/30"
                disabled={recordingLoading}
              >
                {recordingLoading ? (
                  <div className="animate-spin w-4 h-4 border-2 border-white/30 border-t-white rounded-full" />
                ) : (
                  <div className="flex gap-0.5">
                    <div className="w-1 h-3 bg-white rounded-sm" />
                    <div className="w-1 h-3 bg-white rounded-sm" />
                  </div>
                )}
              </IconButton>
            )}

            {/* Audio Controls - show when recording */}
            {isRecording && (
              <>
                <IconButton
                  onClick={toggleMic}
                  tooltip={micMuted ? "Unmute Microphone" : "Mute Microphone"}
                  className={micMuted ? "bg-red-500/50 hover:bg-red-500/70" : ""}
                >
                  {micMuted ? <MicOff size={16} /> : <Mic size={16} />}
                </IconButton>
                
                <IconButton
                  onClick={toggleSpeaker}
                  tooltip={speakerMuted ? "Unmute Speaker" : "Mute Speaker"}
                  className={speakerMuted ? "bg-red-500/50 hover:bg-red-500/70" : ""}
                >
                  {speakerMuted ? <VolumeX size={16} /> : <Volume2 size={16} />}
                </IconButton>
              </>
            )}
            
            <div className="w-px h-6 bg-white/20 mx-1" />
            
            <IconButton onClick={openSettings} tooltip="Settings">
              <Settings size={16} />
            </IconButton>
            
            <div
              className="ml-1 p-1.5 text-white/60 cursor-move hover:text-white/90 hover:bg-white/10 rounded-lg transition-all duration-200"
              onMouseDown={handleMouseDown}
              title="Drag to move"
              style={{ userSelect: 'none' }}
            >
              <Grip size={16} />
            </div>
          </div>
        </div>
      </div>
      
      {/* Settings Popup */}
      <SettingsPopup 
        isOpen={showSettings} 
        onClose={() => setShowSettings(false)}
        position={position}
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
      className={`p-2 bg-white/15 backdrop-blur-sm rounded-xl text-white shadow-lg hover:bg-white/25 active:bg-white/35 transition-all duration-200 flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed border border-white/10 hover:border-white/20 ${className}`}
      title={tooltip}
    >
      {children}
    </button>
  );
}

// Settings popup component
function SettingsPopup({ isOpen, onClose, position }: {
  isOpen: boolean;
  onClose: () => void;
  position: { x: number; y: number };
}) {
  if (!isOpen) return null;

  // Smart positioning - open downwards if close to top, upwards otherwise
  const isNearTop = position.y < 250; // Within 250px of top
  const popupTop = isNearTop ? position.y + 60 : position.y - 200; // 60px below control bar or 200px above

  return (
    <div
      className="absolute z-50"
      style={{
        left: position.x,
        top: popupTop,
      }}
    >
      <div className="rounded-2xl shadow-2xl p-4 w-64"
        style={{
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
            <div className="w-10 h-6 bg-white/20 rounded-full relative transition-colors">
              <div className="w-4 h-4 bg-white rounded-full absolute top-1 left-1 transition-transform" />
            </div>
          </div>
          
          <div className="flex items-center justify-between">
            <span className="text-white/80 text-sm">Show audio levels</span>
            <div className="w-10 h-6 bg-blue-500/60 rounded-full relative">
              <div className="w-4 h-4 bg-white rounded-full absolute top-1 right-1 transition-transform" />
            </div>
          </div>
          
          <div className="flex items-center justify-between">
            <span className="text-white/80 text-sm">Always on top</span>
            <div className="w-10 h-6 bg-blue-500/60 rounded-full relative">
              <div className="w-4 h-4 bg-white rounded-full absolute top-1 right-1 transition-transform" />
            </div>
          </div>
          
          <div className="pt-2 border-t border-white/10">
            <button className="w-full bg-white/15 hover:bg-white/25 text-white text-sm py-2 px-3 rounded-lg transition-colors">
              Open Main Settings
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
