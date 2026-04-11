import { useState, useEffect } from "react";
import { insightsGetPending } from "../../hooks/useTauri";
import type { InsightCard } from "../../types/chronos";

export function SynergyStream() {
  const [insights, setInsights] = useState<InsightCard[]>([]);
  const [filter, setFilter] = useState<string>("all");

  useEffect(() => {
    insightsGetPending()
      .then(setInsights)
      .catch(() => {
        setInsights(getDemoInsights());
      });

    const interval = setInterval(() => {
      setInsights((prev) => {
        if (prev.length > 20) return prev;
        return [createRandomInsight(), ...prev];
      });
    }, 8000);

    return () => clearInterval(interval);
  }, []);

  const filterTypes = ["all", "synthesis", "prediction", "contradiction", "connection"];

  const filtered =
    filter === "all" ? insights : insights.filter((i) => i.insight_type === filter);

  return (
    <div className="h-full flex flex-col p-4 gap-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold flex items-center gap-2">
          <span className="animate-pulse-glow inline-block w-2 h-2 rounded-full bg-secondary" />
          Synergy Stream
        </h2>
        <div className="flex gap-1">
          {filterTypes.map((type) => (
            <button
              key={type}
              onClick={() => setFilter(type)}
              className={`px-3 py-1 rounded-full text-xs transition-colors ${
                filter === type
                  ? "bg-secondary text-secondary-foreground"
                  : "bg-card text-muted-foreground hover:bg-card-hover"
              }`}
            >
              {type}
            </button>
          ))}
        </div>
      </div>

      <div className="flex-1 overflow-y-auto space-y-3">
        {filtered.map((insight) => (
          <InsightCardComponent key={insight.id} insight={insight} />
        ))}

        {filtered.length === 0 && (
          <div className="flex flex-col items-center justify-center h-48 text-muted-foreground">
            <span className="text-3xl mb-3">💡</span>
            <p className="text-sm">No insights yet.</p>
            <p className="text-xs mt-1">Insights appear as your knowledge graph grows.</p>
          </div>
        )}
      </div>
    </div>
  );
}

function InsightCardComponent({ insight }: { insight: InsightCard }) {
  const [expanded, setExpanded] = useState(false);

  const typeColors: Record<string, string> = {
    synthesis: "border-l-primary bg-primary/5",
    prediction: "border-l-secondary bg-secondary/5",
    contradiction: "border-l-accent bg-accent/5",
    connection: "border-l-warning bg-warning/5",
  };

  const typeIcons: Record<string, string> = {
    synthesis: "🧠",
    prediction: "🔮",
    contradiction: "⚡",
    connection: "🔗",
  };

  return (
    <div
      className={`rounded-xl p-4 border border-border border-l-4 ${typeColors[insight.insight_type] || ""} animate-slide-in`}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-sm">{typeIcons[insight.insight_type]}</span>
            <h3 className="text-sm font-medium">{insight.title}</h3>
          </div>
          <p className={`text-xs text-muted-foreground leading-relaxed ${!expanded && "line-clamp-2"}`}>
            {insight.content}
          </p>
          {insight.content.length > 120 && (
            <button
              onClick={() => setExpanded(!expanded)}
              className="text-xs text-primary mt-1 hover:underline"
            >
              {expanded ? "Show less" : "Read more"}
            </button>
          )}
          <div className="flex gap-2 mt-2">
            {insight.tags.slice(0, 4).map((tag) => (
              <span key={tag} className="px-2 py-0.5 rounded-full bg-border/50 text-xs text-muted-foreground">
                #{tag}
              </span>
            ))}
          </div>
        </div>
        <div className="text-right shrink-0">
          <div
            className="text-xs font-mono"
            style={{ color: confidenceColor(insight.confidence) }}
          >
            {Math.round(insight.confidence * 100)}%
          </div>
          <div className="text-xs text-muted-foreground mt-1">
            {new Date(insight.created_at).toLocaleTimeString()}
          </div>
        </div>
      </div>
    </div>
  );
}

function confidenceColor(conf: number): string {
  if (conf >= 0.8) return "#00d4aa";
  if (conf >= 0.6) return "#ffaa00";
  return "#ff6b6b";
}

function createRandomInsight(): InsightCard {
  const types = ["synthesis", "prediction", "contradiction", "connection"];
  const type = types[Math.floor(Math.random() * types.length)];
  return {
    id: `live-${Date.now()}`,
    title: "New insight detected",
    content: "The knowledge graph found a new connection between recent entries.",
    insight_type: type as InsightCard["insight_type"],
    confidence: 0.6 + Math.random() * 0.4,
    created_at: Date.now(),
    tags: ["live", "detected"],
  };
}

function getDemoInsights(): InsightCard[] {
  return [
    {
      id: "1",
      title: "Synthesis: Dual-provider routing",
      content: "The MiniMax + Gemini fallback strategy provides resilience. When T3 MiniMax fails, the system automatically retries with Gemini without user intervention.",
      insight_type: "synthesis",
      confidence: 0.92,
      created_at: Date.now() - 300000,
      tags: ["ai", "routing", "synthesis"],
    },
    {
      id: "2",
      title: "Connection: KuzuDB ↔ Qdrant sync",
      content: "The 2PC commit ensures KuzuDB and Qdrant never drift. If Qdrant upsert fails, KuzuDB is rolled back to the pre-write state.",
      insight_type: "connection",
      confidence: 0.88,
      created_at: Date.now() - 600000,
      tags: ["sync", "2pc", "database"],
    },
    {
      id: "3",
      title: "Prediction: Graph density spike",
      content: "Based on current brain dump frequency, the knowledge graph will reach 1000 nodes within 7 days.",
      insight_type: "prediction",
      confidence: 0.71,
      created_at: Date.now() - 900000,
      tags: ["prediction", "graph", "growth"],
    },
  ];
}
