import { useState, useEffect, useRef, useCallback } from "react";
import { graphGetTimeline } from "../../hooks/useTauri";
import type { TimelineEntry } from "../../types/chronos";

const ITEM_HEIGHT = 72;
const VISIBLE_BUFFER = 5;

export function TimelineView() {
  const [entries, setEntries] = useState<TimelineEntry[]>([]);
  const [scrollTop, setScrollTop] = useState(0);
  const [containerHeight, setContainerHeight] = useState(600);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    graphGetTimeline(200)
      .then(setEntries)
      .catch(() => {
        setEntries(getDemoTimeline());
      });
  }, []);

  const containerRef = useCallback((node: HTMLDivElement | null) => {
    if (!node) return;
    setContainerHeight(node.clientHeight);
    const observer = new ResizeObserver(() => {
      setContainerHeight(node.clientHeight);
    });
    observer.observe(node);
  }, []);

  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  };

  const visibleCount = Math.ceil(containerHeight / ITEM_HEIGHT) + VISIBLE_BUFFER * 2;
  const startIndex = Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - VISIBLE_BUFFER);
  const endIndex = Math.min(entries.length, startIndex + visibleCount);

  const formatTime = (ts: number) =>
    new Date(ts).toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit" });

  const getEventIcon = (type: string) => {
    switch (type) {
      case "thought_added": return "💭";
      case "claim_added": return "📝";
      case "wiki_updated": return "📚";
      case "connection_found": return "🔗";
      default: return "•";
    }
  };

  return (
    <div ref={containerRef} className="h-full overflow-y-auto p-4" onScroll={handleScroll}>
      <div style={{ height: entries.length * ITEM_HEIGHT, position: "relative" }}>
        {entries.slice(startIndex, endIndex).map((entry, i) => {
          const actualIndex = startIndex + i;
          const top = actualIndex * ITEM_HEIGHT;

          return (
            <div
              key={entry.id}
              className="absolute left-0 right-0 px-4 py-3 flex items-start gap-4 hover:bg-card-hover rounded-lg transition-colors"
              style={{ top, height: ITEM_HEIGHT }}
            >
              <span className="text-xs text-muted-foreground w-16 shrink-0 pt-1">
                {formatTime(entry.timestamp)}
              </span>
              <span className="text-lg shrink-0 mt-0.5">
                {getEventIcon(entry.event_type)}
              </span>
              <div className="flex-1 min-w-0">
                <p className="text-sm text-foreground truncate">{entry.description}</p>
                <div className="flex gap-2 mt-1">
                  {entry.tags.slice(0, 3).map((tag) => (
                    <span key={tag} className="text-xs text-muted-foreground">
                      #{tag}
                    </span>
                  ))}
                </div>
              </div>
              <div className="w-1 h-full rounded-full bg-primary shrink-0 opacity-50" />
            </div>
          );
        })}
      </div>

      {entries.length === 0 && (
        <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
          <span className="text-4xl mb-4">📅</span>
          <p className="text-sm">Your timeline is empty.</p>
          <p className="text-xs mt-1">Brain dump some thoughts to get started.</p>
        </div>
      )}
    </div>
  );
}

function getDemoTimeline(): TimelineEntry[] {
  const now = Date.now();
  return [
    { id: "1", event_type: "thought_added", description: "The synthesis engine should use bitemporal versioning", timestamp: now - 300000, entity_id: "1", tags: ["synthesis", "design"] },
    { id: "2", event_type: "wiki_updated", description: "Updated KuzuDB schema documentation", timestamp: now - 600000, entity_id: "2", tags: ["docs", "kuzu"] },
    { id: "3", event_type: "claim_added", description: "Chronos will achieve 60fps graph rendering", timestamp: now - 900000, entity_id: "3", tags: ["performance", "graph"] },
    { id: "4", event_type: "connection_found", description: "MiniMax and Gemini providers share the same budget pool", timestamp: now - 1200000, entity_id: "4", tags: ["ai", "providers"] },
    { id: "5", event_type: "thought_added", description: "2PC commit must prevent orphaned records in Qdrant", timestamp: now - 1800000, entity_id: "5", tags: ["sync", "2pc"] },
  ];
}
