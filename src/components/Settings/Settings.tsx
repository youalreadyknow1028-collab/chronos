import { useState, useEffect } from "react";
import {
  saveApiKey,
  listSettings,
  triggerVaultSync,
} from "../../hooks/useTauri";

interface SettingsProps {
  isOpen: boolean;
  onClose: () => void;
}

export function Settings({ isOpen, onClose }: SettingsProps) {
  const [minimaxKey, setMinimaxKey] = useState("");
  const [geminiKey, setGeminiKey] = useState("");
  const [vaultPath, setVaultPath] = useState("");
  const [isSyncing, setIsSyncing] = useState(false);
  const [syncResult, setSyncResult] = useState<{ processed: number; errors: string[] } | null>(
    null
  );
  const [status, setStatus] = useState("");

  useEffect(() => {
    if (isOpen) loadSettings();
  }, [isOpen]);

  async function loadSettings() {
    try {
      const s = await listSettings();
      for (const [k, v] of s) {
        if (k === "MINIMAX_API_KEY") setMinimaxKey(v === "****" ? "" : v);
        if (k === "GEMINI_API_KEY") setGeminiKey(v === "****" ? "" : v);
      }
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  }

  async function saveKey(name: string, value: string) {
    if (!value.trim()) return;
    setStatus(`Saving ${name}...`);
    try {
      await saveApiKey(name, value);
      setStatus(`${name} saved`);
      setTimeout(() => setStatus(""), 3000);
    } catch (e) {
      setStatus(`Error: ${e}`);
    }
  }

  async function selectVaultDirectory() {
    const path = window.prompt(
      "Enter path to your PARA/Vault root folder:",
      vaultPath
    );
    if (path) setVaultPath(path);
  }

  async function triggerSync() {
    if (!vaultPath) {
      setStatus("Please enter a vault directory path first");
      return;
    }
    setIsSyncing(true);
    setSyncResult(null);
    setStatus("Syncing vault...");
    try {
      const result = await triggerVaultSync(vaultPath);
      setSyncResult(result);
      setStatus(
        result.processed > 0
          ? `Synced ${result.processed} files`
          : "No files processed"
      );
    } catch (e) {
      setStatus(`Sync failed: ${e}`);
    } finally {
      setIsSyncing(false);
    }
  }

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm">
      <div className="bg-[#1a1a2e] border border-[#16213e] rounded-2xl w-full max-w-md mx-4 shadow-2xl">
        {/* Header */}
        <div className="flex justify-between items-center px-6 py-4 border-b border-[#16213e]">
          <h2 className="text-lg font-bold text-white">⚙️ Settings</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors text-2xl leading-none"
          >
            ×
          </button>
        </div>

        <div className="p-6 space-y-6">
          {/* API Keys */}
          <section>
            <h3 className="text-xs font-bold text-gray-500 uppercase tracking-wider mb-3">
              🔑 API Keys
            </h3>
            <div className="space-y-3">
              <div>
                <label className="block text-sm text-gray-400 mb-1.5">
                  MINIMAX_API_KEY
                </label>
                <div className="flex gap-2">
                  <input
                    type="password"
                    value={minimaxKey}
                    onChange={(e) => setMinimaxKey(e.target.value)}
                    placeholder="sk-..."
                    className="flex-1 bg-[#0f0f23] border border-[#16213e] rounded-lg px-3 py-2.5 text-white text-sm placeholder-gray-600 focus:border-[#e94560] focus:outline-none transition-all"
                  />
                  <button
                    onClick={() => saveKey("MINIMAX_API_KEY", minimaxKey)}
                    className="bg-[#e94560] hover:bg-[#d63d56] text-white px-4 py-2.5 rounded-lg text-sm font-semibold transition-colors whitespace-nowrap"
                  >
                    Save
                  </button>
                </div>
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1.5">
                  GEMINI_API_KEY
                </label>
                <div className="flex gap-2">
                  <input
                    type="password"
                    value={geminiKey}
                    onChange={(e) => setGeminiKey(e.target.value)}
                    placeholder="AIza..."
                    className="flex-1 bg-[#0f0f23] border border-[#16213e] rounded-lg px-3 py-2.5 text-white text-sm placeholder-gray-600 focus:border-[#e94560] focus:outline-none transition-all"
                  />
                  <button
                    onClick={() => saveKey("GEMINI_API_KEY", geminiKey)}
                    className="bg-[#e94560] hover:bg-[#d63d56] text-white px-4 py-2.5 rounded-lg text-sm font-semibold transition-colors whitespace-nowrap"
                  >
                    Save
                  </button>
                </div>
              </div>
            </div>
          </section>

          {/* Vault Config */}
          <section>
            <h3 className="text-xs font-bold text-gray-500 uppercase tracking-wider mb-3">
              📁 Vault Configuration
            </h3>
            <div>
              <label className="block text-sm text-gray-400 mb-1.5">
                PARA Root Folder
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={vaultPath}
                  onChange={(e) => setVaultPath(e.target.value)}
                  placeholder="/path/to/PARA..."
                  className="flex-1 bg-[#0f0f23] border border-[#16213e] rounded-lg px-3 py-2.5 text-white text-sm placeholder-gray-600 focus:border-[#e94560] focus:outline-none transition-all"
                />
                <button
                  onClick={selectVaultDirectory}
                  className="bg-[#16213e] hover:bg-[#1a2a4e] text-white px-4 py-2.5 rounded-lg text-sm font-semibold transition-colors border border-[#0f3460] whitespace-nowrap"
                >
                  Browse
                </button>
              </div>
            </div>
          </section>

          {/* Sync Controls */}
          <section>
            <h3 className="text-xs font-bold text-gray-500 uppercase tracking-wider mb-3">
              🔄 Sync Controls
            </h3>
            <button
              onClick={triggerSync}
              disabled={isSyncing || !vaultPath}
              className="w-full bg-gradient-to-r from-[#0f3460] to-[#16213e] hover:from-[#16213e] hover:to-[#1a2a4e] disabled:opacity-40 disabled:cursor-not-allowed text-white py-3 rounded-xl font-semibold transition-all border border-[#0f3460] shadow-lg"
            >
              {isSyncing ? "⟳ Syncing..." : "🚀 Trigger Vault Sync"}
            </button>

            {syncResult && (
              <div className="mt-3 p-3 bg-[#0f0f23] rounded-xl border border-[#16213e]">
                <p className="text-sm text-emerald-400 font-medium">
                  ✅ Processed {syncResult.processed} files
                </p>
                {syncResult.errors.length > 0 && (
                  <div className="mt-2">
                    <p className="text-xs text-amber-400 mb-1">
                      ⚠️ {syncResult.errors.length} warnings:
                    </p>
                    <div className="max-h-24 overflow-y-auto space-y-0.5">
                      {syncResult.errors.slice(0, 5).map((e, i) => (
                        <p
                          key={i}
                          className="text-xs text-gray-600 font-mono truncate"
                          title={e}
                        >
                          {e.length > 60 ? e.slice(0, 60) + "..." : e}
                        </p>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}
          </section>

          {status && (
            <p className="text-center text-sm text-gray-400 animate-pulse">
              {status}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
