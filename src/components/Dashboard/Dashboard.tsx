import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface HealthStatus {
  status: string;
  kuzu_connected: boolean;
  qdrant_connected: boolean;
  timestamp: number;
}

export function Dashboard() {
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [mindDump, setMindDump] = useState("");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    invoke<HealthStatus>("health_check")
      .then(setHealth)
      .catch(console.error);
  }, []);

  const handleSave = async () => {
    if (!mindDump.trim()) return;
    setSaving(true);
    try {
      const result = await invoke<{ note_id: string; status: string }>("vault_ingest_file", {
        path: "",
        title: new Date().toISOString().split("T")[0],
        tags: ["mind-dump"],
      });
      console.log("Saved:", result);
      setMindDump("");
    } catch (e) {
      console.error("Failed to save:", e);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h1 className="text-4xl font-bold mb-1">🕐 Chronos</h1>
      <p className="text-sm text-muted-foreground mb-8">Your Second Brain — Remembers everything.</p>

      <div className="grid grid-cols-2 gap-4 mb-6">
        <div className="p-5 rounded-xl border border-border bg-card">
          <h2 className="text-sm font-medium text-muted-foreground">Knowledge Graph</h2>
          <p className="text-3xl font-bold mt-1">—</p>
          <p className="text-xs text-muted-foreground mt-1">
            {health?.kuzu_connected ? "🟢 KuzuDB connected" : "⚠️ KuzuDB offline"}
          </p>
        </div>
        <div className="p-5 rounded-xl border border-border bg-card">
          <h2 className="text-sm font-medium text-muted-foreground">Vector Search</h2>
          <p className="text-3xl font-bold mt-1">—</p>
          <p className="text-xs text-muted-foreground mt-1">
            {health?.qdrant_connected ? "🟢 Qdrant connected" : "⚠️ Qdrant offline"}
          </p>
        </div>
      </div>

      <div className="p-6 rounded-xl border border-border bg-card">
        <h2 className="text-lg font-semibold mb-3">💭 Daily Mind Dump</h2>
        <p className="text-xs text-muted-foreground mb-4">
          Dump everything — thoughts, ideas, facts, feelings. Chronos turns it into a living knowledge graph.
        </p>
        <textarea
          className="w-full h-40 p-3 rounded-lg border border-input bg-background resize-none text-sm"
          placeholder="What's on your mind?..."
          value={mindDump}
          onChange={(e) => setMindDump(e.target.value)}
        />
        <button
          onClick={handleSave}
          disabled={saving || !mindDump.trim()}
          className="mt-3 px-6 py-2 rounded-lg bg-primary text-primary-foreground hover:opacity-90 disabled:opacity-50 text-sm font-medium"
        >
          {saving ? "Saving..." : "Save to Chronos"}
        </button>
      </div>
    </div>
  );
}
