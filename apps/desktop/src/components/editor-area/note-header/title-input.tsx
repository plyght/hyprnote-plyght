import { useLingui } from "@lingui/react/macro";
import { SparklesIcon } from "lucide-react";
import { type ChangeEvent, type KeyboardEvent } from "react";

interface TitleInputProps {
  value: string;
  onChange: (e: ChangeEvent<HTMLInputElement>) => void;
  onNavigateToEditor?: () => void;
  editable?: boolean;
  onGenerateTitle?: () => void;
  isGeneratingTitle?: boolean;
}

export default function TitleInput({
  value,
  onChange,
  onNavigateToEditor,
  editable,
  onGenerateTitle,
  isGeneratingTitle,
}: TitleInputProps) {
  const { t } = useLingui();

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" || e.key === "Tab") {
      e.preventDefault();
      onNavigateToEditor?.();
    }
  };

  return (
    <div className="flex items-center gap-2 w-full">
      <input
        disabled={!editable}
        id="note-title-input"
        type="text"
        onChange={onChange}
        value={value}
        placeholder={t`Untitled`}
        className="flex-1 border-none bg-transparent text-2xl font-bold focus:outline-none placeholder:text-neutral-400"
        onKeyDown={handleKeyDown}
      />
      {editable && onGenerateTitle && (
        <button
          onClick={onGenerateTitle}
          disabled={isGeneratingTitle}
          className="p-1.5 rounded-md hover:bg-neutral-100 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          title="Generate title with AI"
          aria-label="Generate title with AI"
        >
          <SparklesIcon
            size={18}
            className={`text-neutral-600 ${isGeneratingTitle ? "animate-pulse" : ""}`}
          />
        </button>
      )}
    </div>
  );
}
