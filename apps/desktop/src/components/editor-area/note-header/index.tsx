import { useMatch } from "@tanstack/react-router";
import { CheckIcon, CopyIcon } from "lucide-react";
import { type ChangeEvent, useCallback, useState } from "react";

import { commands as tauriCommands } from "@/types/tauri.gen";
import { convertHtmlToMarkdown } from "@/utils/parse";
import { getCurrentWebviewWindowLabel } from "@hypr/plugin-windows";
import { cn } from "@hypr/ui/lib/utils";
import { useSession } from "@hypr/utils/contexts";
import Chips from "./chips";
import ListenButton from "./listen-button";
import TitleInput from "./title-input";

interface NoteHeaderProps {
  onNavigateToEditor?: () => void;
  editable?: boolean;
  sessionId: string;
  hashtags?: string[];
}

export function NoteHeader({ onNavigateToEditor, editable, sessionId, hashtags = [] }: NoteHeaderProps) {
  const updateTitle = useSession(sessionId, (s) => s.updateTitle);
  const sessionTitle = useSession(sessionId, (s) => s.session.title);
  const [showRaw, enhancedContent] = useSession(sessionId, (s) => [
    s.showRaw,
    s.session?.enhanced_memo_html ?? "",
  ]);

  const handleTitleChange = (e: ChangeEvent<HTMLInputElement>) => {
    updateTitle(e.target.value);
  };

  const handleCopyEnhanced = useCallback(async () => {
    if (enhancedContent) {
      try {
        const markdown = convertHtmlToMarkdown(enhancedContent);
        await tauriCommands.clipboardWriteText(markdown);
      } catch (error) {
        console.error("Failed to copy enhanced notes:", error);
      }
    }
  }, [enhancedContent]);

  const noteMatch = useMatch({ from: "/app/note/$id", shouldThrow: false });
  const windowLabel = getCurrentWebviewWindowLabel();
  const isInNoteMain = windowLabel === "main" && noteMatch;

  return (
    <div className="flex items-center w-full pl-8 pr-6 pb-4 gap-4">
      <div className="flex-1 space-y-1">
        <TitleInput
          editable={editable}
          value={sessionTitle}
          onChange={handleTitleChange}
          onNavigateToEditor={onNavigateToEditor}
        />
        <Chips sessionId={sessionId} hashtags={hashtags} />
      </div>

      <div className="flex items-center gap-2">
        {!showRaw && enhancedContent && <CopyButton onCopy={handleCopyEnhanced} />}
        {isInNoteMain && <ListenButton sessionId={sessionId} />}
      </div>
    </div>
  );
}

function CopyButton({ onCopy }: { onCopy: () => void }) {
  const [copied, setCopied] = useState(false);

  const handleClick = () => {
    onCopy();
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const buttonClasses = cn(
    "rounded-md border px-2.5 py-1.5 transition-all ease-in-out",
    "border-border bg-background text-neutral-600 hover:bg-neutral-100 hover:text-neutral-800",
    "hover:scale-105 transition-transform duration-200",
  );

  return (
    <button
      onClick={handleClick}
      className={buttonClasses}
      title="Copy notes"
    >
      {copied
        ? <CheckIcon size={14} className="text-neutral-800" />
        : <CopyIcon size={14} className="text-neutral-600" />}
    </button>
  );
}
