import { useState, useEffect } from "react";
import { BrainPulseDashboard } from "../BrainPulseDashboard/BrainPulseDashboard";
import { GraphView } from "../GraphView/GraphView";
import { TimelineView } from "../TimelineView/TimelineView";
import { SynergyStream } from "../SynergyStream/SynergyStream";
import { getHealth } from "../../hooks/useTauri";
import type { HealthStatus } from "../../types/chronos";

type View = "graph" | "timeline" | "insights";

export function AppShell() {
  const [activeView, setActiveView] = useState<View>("graph");
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(true);

  useEffect(() => {
    getHealth().then(setHealth).catch(() => {});
    const interval = setInterval(() => {
      getHealth().then(setHealth).catch(() => {});
    }, 30000);
    return () => clearInterval(interval);
  }, []);

  const navItems: { id: View; label: string; icon: string }[] = [
    { id: "graph", label: "Knowledge Graph", icon: "🕸️" },
    { id: "timeline", label: "Timeline", icon: "📅" },
    { id: "insights", label: "Insights", icon: "💡" },
  ];

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
      {/* Sidebar */}
      <aside
        className={`${sidebarOpen ? "w-64" : "w-16"} transition-all duration-300 flex flex-col border-r border-border bg-card`}
      >
        {/* Logo */}
        <div className="flex items-center gap-3 p-4 border-b border-border">
          <button
            onClick={() => setSidebarOpen(!sidebarOpen)}
            className="text-lg hover:opacity-70 transition-opacity"
          >
            🕐
          </button>
          {sidebarOpen && (
            <div className="animate-fade-in">
              <h1 className="text-lg font-bold text-foreground">Chronos</h1>
              <p className="text-xs text-muted-foreground">Second Brain</p>
            </div>
          )}
        </div>

        {/* Navigation */}
        <nav className="flex-1 py-4">
          {navItems.map((item) => (
            <button
              key={item.id}
              onClick={() => setActiveView(item.id)}
              className={`w-full flex items-center gap-3 px-4 py-3 text-sm transition-colors ${
                activeView === item.id
                  ? "bg-primary/10 text-primary border-r-2 border-primary"
                  : "text-muted-foreground hover:bg-card-hover hover:text-foreground"
              }`}
            >
              <span className="text-lg">{item.icon}</span>
              {sidebarOpen && <span>{item.label}</span>}
            </button>
          ))}
        </nav>

        {/* Health indicators */}
        {sidebarOpen && (
          <div className="p-4 border-t border-border animate-fade-in">
            <div className="space-y-1 text-xs">
              <div className="flex justify-between">
                <span className="text-muted-foreground">KuzuDB</span>
                <span className={health?.kuzu_connected ? "text-success" : "text-error"}>
                  {health?.kuzu_connected ? "🟢" : "⚠️"}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Qdrant</span>
                <span className={health?.qdrant_connected ? "text-success" : "text-error"}>
                  {health?.qdrant_connected ? "🟢" : "⚠️"}
                </span>
              </div>
            </div>
          </div>
        )}
      </aside>

      {/* Main content */}
      <main className="flex-1 flex flex-col overflow-hidden">
        {/* Top bar */}
        <header className="flex items-center justify-between px-6 py-3 border-b border-border bg-card/50 backdrop-blur">
          <div>
            <h2 className="text-sm font-medium">
              {navItems.find((n) => n.id === activeView)?.label}
            </h2>
          </div>
          <BrainPulseDashboard compact />
        </header>

        {/* View area */}
        <div className="flex-1 overflow-hidden">
          {activeView === "graph" && <GraphView />}
          {activeView === "timeline" && <TimelineView />}
          {activeView === "insights" && <SynergyStream />}
        </div>
      </main>
    </div>
  );
}
