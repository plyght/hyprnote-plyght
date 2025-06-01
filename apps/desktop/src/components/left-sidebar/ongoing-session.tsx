import { useNavigate } from "@tanstack/react-router";
import { motion } from "motion/react";

import { useHyprSearch } from "@/contexts";
import { useSession } from "@hypr/utils/contexts";

export default function OngoingSession({
  sessionId,
}: {
  sessionId: string;
}) {
  const navigate = useNavigate();
  const session = useSession(sessionId, (s) => s.session);

  const { setQuery } = useHyprSearch((s) => ({
    setQuery: s.setQuery,
  }));

  const handleClick = () => {
    setQuery("");

    navigate({
      to: "/app/note/$id",
      params: { id: sessionId },
    });
  };

  return (
    <motion.div
      key={sessionId}
      layout
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
      className="p-2 mb-4"
    >
      <button
        onClick={handleClick}
        className="w-full flex items-center justify-between transition-all bg-secondary hover:bg-secondary/80 px-3 py-2.5 rounded-lg hover:scale-95 duration-300"
      >
        <div className="font-medium text-sm text-secondary-foreground max-w-[180px] truncate">
          {session.title || "Untitled"}
        </div>

        <div className="relative h-2 w-2">
          <div className="absolute inset-0 rounded-full bg-secondary-foreground/30"></div>
          <div className="absolute inset-0 rounded-full bg-secondary-foreground animate-ping"></div>
        </div>
      </button>
    </motion.div>
  );
}
