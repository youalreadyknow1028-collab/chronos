import { useState, useEffect } from "react";
import { pipelineBudgetStatus, pipelineIngest } from "../../hooks/useTauri";
import type { BudgetStatus, IngestResponse } from "../../types/chronos";

interface Props {
  compact?: boolean;
}

export function BrainPulseDashboard({ compact = false }: Props) {
  const [budget, setBudget] = useState<BudgetStatus | null>(null);
  const [lastIngest, setLastIngest] = useState<IngestResponse | null>(null);
  const [saving, setSaving] = useState(false);
  const [mindDump, setMindDump] = useState("");

  useEffect(() => {
    pipelineBudgetStatus()
      .then(setBudget)
      .catch(() => setBudget(null));

    const interval = setInterval(() => {
      pipelineBudgetStatus()
        .then(setBudget)
        .catch(() => {});
    }, 15000);

    return () => clearInterval(interval);
  }, []);

  const handleSave = async () => {
    if (!mindDump.trim()) return;
    setSaving(true);
    try {
      const result = await pipelineIngest({
        content: mindDump,
        source: "mind_dump",
        tags: ["brain-pulse"],
      });
      setLastIngest(result);
      setMindDump("");
    } catch (e) {
      console.error("Ingest failed:", e);
    } finally {
      setSaving(false);
    }
  };

  if (compact) {
    return (
      <div className="flex items-center gap-4">
        {budget && (
          <>
            <div className="flex items-center gap-2 text-xs">
              <span className="text-muted-foreground">MiniMax</span>
              <div className="w-20 h-1.5 rounded-full bg-border overflow-hidden">
                <div
                  className="h-full rounded-full bg-primary transition-all"
                  style={{
                    width: `${(budget.minimax_tokens / budget.minimax_tokens_limit) * 100}%`,
                  }}
                />
              </div>
              <span className="text-muted-foreground">
                {budget.minimax_calls}/{budget.minimax_calls_limit}
              </span>
            </div>
            <div className="flex items-center gap-2 text-xs">
              <span className="text-muted-foreground">Gemini</span>
              <div className="w-12 h-1.5 rounded-full bg-border overflow-hidden">
                <div
                  className="h-full rounded-full bg-secondary transition-all"
                  style={{
                    width: `${(budget.gemini_tokens / budget.gemini_tokens_limit) * 100}%`,
                  }}
                />
              </div>
            </div>
          </>
        )}
      </div>
    );
  }

  return (
    <div className="p-4 space-y-4">
      <h3 className="text-sm font-semibold text-foreground flex items-center gap-2">
        <span className="animate-pulse-glow inline-block w-2 h-2 rounded-full bg-primary" />
        Brain Pulse
      </h3>

      {budget && (
        <div className="space-y-3">
          <div>
            <div className="flex justify-between text-xs mb-1">
              <span className="text-muted-foreground">MiniMax (T4)</span>
              <span className="text-muted-foreground">
                {budget.minimax_calls}/{budget.minimax_calls_limit}
              </span>
            </div>
            <div className="h-2 rounded-full bg-border overflow-hidden">
              <div
                className="h-full rounded-full bg-primary transition-all duration-500"
                style={{
                  width: `${(budget.minimax_tokens / budget.minimax_tokens_limit) * 100}%`,
                }}
              />
            </div>
          </div>
          <div>
            <div className="flex justify-between text-xs mb-1">
              <span className="text-muted-foreground">Gemini (T3)</span>
              <span className="text-muted-foreground">
                {budget.gemini_calls}/{budget.gemini_calls_limit ?? 10}
              </span>
            </div>
            <div className="h-2 rounded-full bg-border overflow-hidden">
              <div
                className="h-full rounded-full bg-secondary transition-all duration-500"
                style={{
                  width: `${(budget.gemini_tokens / (budget.gemini_tokens_limit || 10000)) * 100}%`,
                }}
              />
            </div>
          </div>
        </div>
      )}

      <div>
        <textarea
          className="w-full h-24 p-2 rounded-lg border border-border bg-background text-sm resize-none focus:outline-none focus:border-primary"
          placeholder="💭 Brain dump..."
          value={mindDump}
          onChange={(e) => setMindDump(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) handleSave();
          }}
        />
        <button
          onClick={handleSave}
          disabled={saving || !mindDump.trim()}
          className="mt-2 w-full py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 disabled:opacity-50 transition-opacity"
        >
          {saving ? "Processing..." : "→ Think"}
        </button>
        {lastIngest && (
          <p className="mt-2 text-xs text-success animate-fade-in">
            ✓ Saved via {lastIngest.provider_used}/{lastIngest.tier_used}
          </p>
        )}
      </div>
    </div>
  );
}
