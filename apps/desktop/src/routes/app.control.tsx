import { createFileRoute } from "@tanstack/react-router";
import { Camera, Circle, Grip, Settings, Square } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { emit } from "@tauri-apps/api/event";

import { commands as windowsCommands } from "@hypr/plugin-windows";

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
  const [isRecording, setIsRecording] = useState(false);
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

  const toggleRecording = () => {
    setIsRecording(!isRecording);
    emit("debug", isRecording ? "Stop recording" : "Start recording");
    setTimeout(updateOverlayBounds, 0);
  };

  const openSettings = () => {
    emit("debug", "Open settings");
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
          className="bg-black/10 backdrop-blur-sm rounded-xl border border-white/30 shadow-lg flex items-center justify-center transition-all duration-200 p-2"
          ref={toolbarRef}
          onClick={handleToolbarClick}
          style={{ pointerEvents: 'auto' }}
        >
          <div className="flex gap-2 items-center">
            <IconButton onClick={captureScreenshot} tooltip="Take Screenshot">
              <Camera size={16} />
            </IconButton>
            <IconButton
              onClick={toggleRecording}
              tooltip={isRecording ? "Stop Recording" : "Start Recording"}
              className={isRecording ? "bg-red-500/50 hover:bg-red-500/70" : ""}
            >
              {isRecording ? <Square size={16} /> : <Circle size={16} />}
            </IconButton>
            <IconButton onClick={openSettings} tooltip="Settings">
              <Settings size={16} />
            </IconButton>
            <div
              className="ml-1 p-1 text-white/50 cursor-move hover:text-white/70 hover:bg-white/10 rounded transition-colors duration-200"
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
  );
}

function IconButton({ onClick, children, className = "", tooltip = "" }: {
  onClick?: ((e: React.MouseEvent<HTMLButtonElement>) => void) | (() => void);
  children: React.ReactNode;
  className?: string;
  tooltip?: string;
}) {
  const handleClick = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation(); // Prevent button clicks from triggering drag
    onClick?.(e);
  };

  return (
    <button
      onClick={handleClick}
      className={`p-1.5 bg-white/20 backdrop-blur-sm rounded-full text-xs shadow-md hover:bg-white/30 transition-colors duration-200 flex items-center justify-center ${className}`}
      title={tooltip}
    >
      {children}
    </button>
  );
}
