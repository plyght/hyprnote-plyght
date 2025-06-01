import { Trans } from "@lingui/react/macro";
import { useLingui } from "@lingui/react/macro";
import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type LinkProps, useMatch, useNavigate } from "@tanstack/react-router";
import { confirm } from "@tauri-apps/plugin-dialog";
import { endOfMonth, startOfMonth, subMonths } from "date-fns";
import { AppWindowMacIcon, ArrowUpRight, CalendarDaysIcon, TrashIcon } from "lucide-react";
import { motion } from "motion/react";
import { useCallback, useEffect, useRef } from "react";

import { useHypr } from "@/contexts";
import { useEnhancePendingState } from "@/hooks/enhance-pending";
import { commands as dbCommands, type Event, type Session } from "@hypr/plugin-db";
import { commands as miscCommands } from "@hypr/plugin-misc";
import { commands as windowsCommands } from "@hypr/plugin-windows";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@hypr/ui/components/ui/context-menu";
import { SplashLoader } from "@hypr/ui/components/ui/splash";
import { cn } from "@hypr/ui/lib/utils";
import { useSession, useSessions } from "@hypr/utils/contexts";
import { format, formatDate, formatRelative } from "@hypr/utils/datetime";
import { safeNavigate } from "@hypr/utils/navigation";

interface NotesListProps {
  filter: (session: Session) => boolean;
  ongoingSessionId?: string | null;
}

type SessionWithEvent = Session & {
  event: Event | null;
};

export default function NotesList({ ongoingSessionId, filter }: NotesListProps) {
  const observerRef = useRef<IntersectionObserver | null>(null);
  const lastItemRef = useRef<HTMLElement | null>(null);

  const { insertSession, sessionsStore } = useSessions((s) => ({
    insertSession: s.insert,
    sessionsStore: s.sessions,
  }));

  const { userId } = useHypr();
  const sessions = useInfiniteQuery({
    queryKey: ["sessions"],
    queryFn: async ({ pageParam: { monthOffset } }) => {
      const now = new Date();
      const [from, to] = [startOfMonth(now), endOfMonth(now)]
        .map((d) => subMonths(d, monthOffset))
        .map((d) => d.toISOString());

      const sessions = await dbCommands.listSessions({
        type: "dateRange",
        user_id: userId,
        start: from,
        end: to,
        limit: 100,
      });
      sessions.forEach(insertSession);

      const sessionWithEvents = await Promise.all(sessions.map(async (session) => {
        const event = await dbCommands.sessionGetEvent(session.id);
        return { ...session, event };
      }));

      return { sessions: groupSessions(sessionWithEvents) };
    },
    initialPageParam: { monthOffset: 0 },
    getNextPageParam: (_lastPage, _, { monthOffset }) => {
      return monthOffset > 12 * 10 ? undefined : { monthOffset: monthOffset + 1 };
    },
  });

  useEffect(() => {
    if (observerRef.current) {
      observerRef.current.disconnect();
    }

    const observer = new IntersectionObserver((entries) => {
      if (entries[0]?.isIntersecting && sessions.hasNextPage) {
        sessions.fetchNextPage();
      }
    }, {
      threshold: 0.1,
      rootMargin: "50px",
    });

    if (lastItemRef.current) {
      observer.observe(lastItemRef.current);
    }

    observerRef.current = observer;
    return () => observer.disconnect();
  }, [
    sessions.hasNextPage,
    sessions.fetchNextPage,
    sessions.data,
  ]);

  const setLastItemRef = useCallback((node: HTMLElement | null) => {
    lastItemRef.current = node;

    if (node && observerRef.current) {
      observerRef.current.observe(node);
    }
  }, []);

  const noteMatch = useMatch({ from: "/app/note/$id", shouldThrow: false });

  if (!noteMatch) {
    return null;
  }

  const { params: { id: activeSessionId } } = noteMatch;

  return (
    <>
      {sessions.data?.pages.map((page, pageIndex) =>
        page.sessions
          .map(([key, items]: [string, Session[]]) => {
            const filteredItems = items
              .filter((session) => sessionsStore[session.id])
              .filter((session) => !(session.id !== activeSessionId && session.id === ongoingSessionId))
              .filter(filter);

            if (filteredItems.length === 0) {
              return null;
            }

            return (
              <section key={`${key}-${pageIndex}`}>
                <h2 className="font-bold text-muted-foreground mb-1">
                  {key}
                </h2>

                <motion.div layout>
                  {filteredItems.map((session: Session) => (
                    <motion.div
                      key={session.id}
                      layout
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      exit={{ opacity: 0 }}
                      transition={{ duration: 0.2 }}
                    >
                      <NoteItem
                        activeSessionId={activeSessionId}
                        currentSessionId={session.id}
                      />
                    </motion.div>
                  ))}
                </motion.div>
              </section>
            );
          })
      )}

      <div
        ref={sessions.hasNextPage ? setLastItemRef : undefined}
        aria-hidden="true"
        className="h-2 w-full"
      />
    </>
  );
}

function NoteItem({
  activeSessionId,
  currentSessionId,
}: {
  activeSessionId: string;
  currentSessionId: string;
}) {
  const { t } = useLingui();
  const navigate = useNavigate();

  const currentSession = useSession(currentSessionId, (s) => ({
    title: s.session.title,
    created_at: s.session.created_at,
    record_start: s.session.record_start,
  }));

  const isActive = activeSessionId === currentSessionId;

  const isEnhancePending = useEnhancePendingState(currentSessionId);
  const shouldShowEnhancePending = !isActive && isEnhancePending;

  const currentSessionEvent = useQuery({
    queryKey: ["event", currentSessionId],
    queryFn: () => dbCommands.sessionGetEvent(currentSessionId),
  });

  const sessionDate = currentSessionEvent.data?.start_date ?? currentSession.record_start ?? currentSession.created_at;
  const formattedSessionDate = formatDate(sessionDate);

  const queryClient = useQueryClient();

  const deleteSession = useMutation({
    mutationFn: () => dbCommands.deleteSession(currentSessionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sessions"] });
      miscCommands.deleteSessionFolder(currentSessionId).then(() => {
        if (isActive) {
          navigate({ to: "/app/new" });
        }
      });
    },
  });

  const handleClick = () => {
    navigate({
      to: "/app/note/$id",
      params: { id: currentSessionId },
    });
  };

  const handleOpenWindow = () => {
    windowsCommands.windowShow({ type: "note", value: currentSessionId });
  };

  const handleOpenCalendar = () => {
    const params = {
      to: "/app/calendar",
      search: { date: format(currentSession.created_at, "yyyy-MM-dd") },
    } as const satisfies LinkProps;

    const url = `${params.to}?date=${params.search.date}`;

    safeNavigate({ type: "calendar" }, url);
  };

  const buttonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!isActive || !buttonRef.current) {
      return;
    }

    const height = window.innerHeight || document.documentElement.clientHeight;
    const width = window.innerWidth || document.documentElement.clientWidth;

    const { top, left, bottom, right } = buttonRef.current.getBoundingClientRect();
    const isInView = top >= 0 && left >= 0 && bottom <= height && right <= width;

    if (!isInView) {
      buttonRef.current.scrollIntoView({ behavior: "smooth", block: "center" });
    }
  }, [isActive]);

  const handleClickDelete = () => {
    confirm(t`Are you sure you want to delete this note?`).then((yes) => {
      if (yes) {
        deleteSession.mutate();
      }
    });
  };

  return (
    <ContextMenu>
      <ContextMenuTrigger>
        <button
          ref={buttonRef}
          onClick={handleClick}
          disabled={isActive}
          className={cn(
            "group flex items-start gap-3 py-2 w-full text-left transition-all rounded-lg px-2",
            isActive ? "bg-muted" : "hover:bg-muted/50",
          )}
        >
          <div className="flex items-center gap-1 w-full">
            <div className="flex-1 flex flex-col items-start gap-1 truncate">
              <div className="flex items-center justify-between gap-1">
                <div className="font-medium text-sm">
                  {currentSession.title || "Untitled"}
                </div>
              </div>

              <div className="flex items-center gap-3 text-xs text-muted-foreground">
                <span className="font-medium">{formattedSessionDate}</span>
              </div>
            </div>

            {shouldShowEnhancePending && <SplashLoader size={20} strokeWidth={2} />}
          </div>
        </button>
      </ContextMenuTrigger>

      <ContextMenuContent>
        <ContextMenuItem
          className="cursor-pointer flex items-center justify-between"
          onClick={handleOpenWindow}
        >
          <div className="flex items-center gap-2">
            <AppWindowMacIcon size={16} />
            <Trans>New window</Trans>
          </div>
          <ArrowUpRight size={16} className="ml-1 text-zinc-500" />
        </ContextMenuItem>

        <ContextMenuItem
          className="cursor-pointer flex items-center justify-between"
          onClick={handleOpenCalendar}
        >
          <div className="flex items-center gap-2">
            <CalendarDaysIcon size={16} />
            <Trans>View calendar</Trans>
          </div>
          <ArrowUpRight size={16} className="ml-1 text-zinc-500" />
        </ContextMenuItem>

        <ContextMenuSeparator />

        <ContextMenuItem
          className="text-red-500 hover:bg-red-100 hover:text-red-600 cursor-pointer"
          onClick={handleClickDelete}
        >
          <TrashIcon size={16} className="mr-2" />
          <Trans>Delete</Trans>
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}

const groupSessions = (sessions: SessionWithEvent[]): [string, SessionWithEvent[]][] => {
  const getSessionDate = (s: SessionWithEvent): string => {
    if (s.event?.start_date) {
      return s.event.start_date;
    }
    if (s.record_start) {
      return s.record_start;
    }
    return s.created_at;
  };

  const grouped = sessions.reduce<Record<string, SessionWithEvent[]>>((acc, session) => {
    const key = formatRelative(getSessionDate(session));

    if (!acc[key]) {
      acc[key] = [];
    }
    acc[key].push(session);
    return acc;
  }, {});

  const groupedAndSorted = Object.entries(grouped).map(([key, group]) => {
    const sorted = group.sort(
      (a, b) => new Date(getSessionDate(b)).getTime() - new Date(getSessionDate(a)).getTime(),
    );
    return [key, sorted] as [string, SessionWithEvent[]];
  });

  return groupedAndSorted.sort(([_, sessionsA], [__, sessionsB]) => {
    if (sessionsA.length === 0 || sessionsB.length === 0) {
      return 0;
    }

    const newestA = new Date(getSessionDate(sessionsA[0])).getTime();
    const newestB = new Date(getSessionDate(sessionsB[0])).getTime();
    return newestB - newestA;
  });
};
