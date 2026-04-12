import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/index.css";

// ── Global error handlers — catch everything so the WebView doesn't silently die ──
window.addEventListener("error", (e) => {
  console.error("[Chronos Uncaught JS Error]", e.error);
  // Don't prevent default — let the ErrorBoundary handle it if one exists
});

window.addEventListener("unhandledrejection", (e) => {
  console.error("[Chronos Unhandled Rejection]", e.reason);
});

// ── Error Boundary — prevents a single component crash from killing the whole app ──
class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean; errorMessage: string }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false, errorMessage: "" };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, errorMessage: error.message };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error("[Chronos ErrorBoundary]", error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{
          padding: "2rem",
          fontFamily: "system-ui, sans-serif",
          color: "#ef4444",
          background: "#1a1a1a",
          minHeight: "100vh",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
        }}>
          <h1 style={{ fontSize: "1.5rem", marginBottom: "1rem" }}>Chronos crashed</h1>
          <p style={{ color: "#9ca3af", maxWidth: "500px", textAlign: "center" }}>
            {this.state.errorMessage}
          </p>
          <button
            onClick={() => window.location.reload()}
            style={{
              marginTop: "1.5rem",
              padding: "0.5rem 1.5rem",
              background: "#3b82f6",
              color: "white",
              border: "none",
              borderRadius: "6px",
              cursor: "pointer",
            }}
          >
            Reload
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

// ── Mount — wrapped in try/catch at the outermost level ──
const root = document.getElementById("root");
if (!root) {
  throw new Error("Root element #root not found in DOM");
}

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>
);
