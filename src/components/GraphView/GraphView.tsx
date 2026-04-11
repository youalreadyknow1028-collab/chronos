import { useEffect, useRef, useState, useCallback } from "react";
import { graphGetNodes } from "../../hooks/useTauri";
import type { GraphData, GraphNode } from "../../types/chronos";

interface Node extends GraphNode {
  x: number;
  y: number;
  vx: number;
  vy: number;
  radius: number;
}

interface Edge {
  source: Node;
  target: Node;
  label?: string;
}

type Mode = "microscope" | "telescope";

export function GraphView() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animRef = useRef<number>(0);
  const [graphData, setGraphData] = useState<GraphData | null>(null);
  const [nodes, setNodes] = useState<Node[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  const [mode, setMode] = useState<Mode>("microscope");
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);
  const [hoveredNode, setHoveredNode] = useState<Node | null>(null);
  const nodesRef = useRef<Node[]>([]);
  const edgesRef = useRef<Edge[]>([]);

  const getDemoGraph = (): GraphData => ({
    nodes: [
      { id: "1", label: "Nexus Core", type: "Agent", confidence: 1.0 },
      { id: "2", label: "KuzuDB", type: "Concept", confidence: 0.9 },
      { id: "3", label: "Qdrant", type: "Concept", confidence: 0.9 },
      { id: "4", label: "MiniMax AI", type: "Concept", confidence: 0.85 },
      { id: "5", label: "Gemini", type: "Concept", confidence: 0.8 },
      { id: "6", label: "Thoughts", type: "WikiEntry", confidence: 0.75 },
      { id: "7", label: "Claims", type: "WikiEntry", confidence: 0.75 },
      { id: "8", label: "Timeline", type: "WikiEntry", confidence: 0.7 },
    ],
    edges: [
      { source: "1", target: "2", type: "USES" },
      { source: "1", target: "3", type: "USES" },
      { source: "1", target: "4", type: "USES" },
      { source: "4", target: "5", type: "FALLBACK" },
      { source: "6", target: "7", type: "LINKS_TO" },
      { source: "6", target: "8", type: "LINKS_TO" },
    ],
  });

  const initializeGraph = useCallback((data: GraphData) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const width = canvas.width;
    const height = canvas.height;

    const nodeMap = new Map<string, Node>();
    const initializedNodes: Node[] = data.nodes.map((n, i) => {
      const angle = (i / data.nodes.length) * Math.PI * 2;
      const radius = Math.min(width, height) * 0.25;
      const node: Node = {
        ...n,
        x: width / 2 + Math.cos(angle) * radius,
        y: height / 2 + Math.sin(angle) * radius,
        vx: 0,
        vy: 0,
        radius: n.type === "Agent" ? 12 : n.type === "Concept" ? 8 : 6,
      };
      nodeMap.set(n.id, node);
      return node;
    });

    const initializedEdges: Edge[] = data.edges
      .map((e) => {
        const src = nodeMap.get(e.source);
        const tgt = nodeMap.get(e.target);
        if (!src || !tgt) return null;
        return { source: src, target: tgt, label: e.label };
      })
      .filter(Boolean) as Edge[];

    nodesRef.current = initializedNodes;
    edgesRef.current = initializedEdges;
    setNodes(initializedNodes);
    setEdges(initializedEdges);
  }, []);

  // Load graph data
  useEffect(() => {
    graphGetNodes()
      .then((data) => {
        setGraphData(data);
        initializeGraph(data);
      })
      .catch(() => {
        const demo = getDemoGraph();
        setGraphData(demo);
        initializeGraph(demo);
      });
  }, [initializeGraph]);

  // Force-directed simulation
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || nodesRef.current.length === 0) return;

    const width = canvas.width;
    const height = canvas.height;

    const simulation = () => {
      const nodes = nodesRef.current;
      const edges = edgesRef.current;

      const repulsionStrength = mode === "microscope" ? 800 : 200;
      const attractionStrength = mode === "microscope" ? 0.01 : 0.03;
      const centerPull = 0.005;

      for (let i = 0; i < nodes.length; i++) {
        const node = nodes[i];

        for (let j = 0; j < nodes.length; j++) {
          if (i === j) continue;
          const other = nodes[j];
          const dx = node.x - other.x;
          const dy = node.y - other.y;
          const dist = Math.sqrt(dx * dx + dy * dy) || 1;
          const force = repulsionStrength / (dist * dist);
          node.vx += (dx / dist) * force;
          node.vy += (dy / dist) * force;
        }

        for (const edge of edges) {
          if (edge.source.id === node.id || edge.target.id === node.id) {
            const other = edge.source.id === node.id ? edge.target : edge.source;
            const dx = other.x - node.x;
            const dy = other.y - node.y;
            node.vx += dx * attractionStrength;
            node.vy += dy * attractionStrength;
          }
        }

        node.vx += (width / 2 - node.x) * centerPull;
        node.vy += (height / 2 - node.y) * centerPull;

        node.x += node.vx * 0.8;
        node.y += node.vy * 0.8;
        node.vx *= 0.5;
        node.vy *= 0.5;

        node.x = Math.max(20, Math.min(width - 20, node.x));
        node.y = Math.max(20, Math.min(height - 20, node.y));
      }

      nodesRef.current = [...nodes];
      setNodes([...nodes]);
      animRef.current = requestAnimationFrame(simulation);
    };

    animRef.current = requestAnimationFrame(simulation);
    return () => cancelAnimationFrame(animRef.current);
  }, [mode]);

  // Render
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const render = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      ctx.strokeStyle = "#2a2a3a";
      ctx.lineWidth = 1;
      for (const edge of edgesRef.current) {
        ctx.beginPath();
        ctx.moveTo(edge.source.x, edge.source.y);
        ctx.lineTo(edge.target.x, edge.target.y);
        ctx.stroke();

        if (edge.label && mode === "microscope") {
          const mx = (edge.source.x + edge.target.x) / 2;
          const my = (edge.source.y + edge.target.y) / 2;
          ctx.fillStyle = "#555566";
          ctx.font = "9px Inter, sans-serif";
          ctx.fillText(edge.label, mx - 16, my - 4);
        }
      }

      for (const node of nodesRef.current) {
        const isHovered = hoveredNode?.id === node.id;
        const isSelected = selectedNode?.id === node.id;
        const color = getNodeColor(node.type);

        ctx.shadowColor = color;
        ctx.shadowBlur = isHovered || isSelected ? 16 : 4;

        ctx.beginPath();
        ctx.arc(node.x, node.y, node.radius + (isHovered ? 2 : 0), 0, Math.PI * 2);
        ctx.fillStyle = color;
        ctx.fill();

        if (mode === "microscope" || isHovered) {
          ctx.shadowBlur = 0;
          ctx.fillStyle = "#e8e8f0";
          ctx.font = `${mode === "microscope" ? 10 : 11}px Inter, sans-serif`;
          ctx.fillText(node.label, node.x - node.label.length * 3, node.y + node.radius + 14);
        }
      }

      ctx.shadowBlur = 0;
    };

    const interval = setInterval(render, 1000 / 60);
    return () => clearInterval(interval);
  }, [nodes, hoveredNode, selectedNode, mode]);

  // Mouse interaction
  const handleMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    let found: Node | null = null;
    for (const node of nodesRef.current) {
      const dx = x - node.x;
      const dy = y - node.y;
      if (Math.sqrt(dx * dx + dy * dy) < node.radius + 4) {
        found = node;
        break;
      }
    }
    setHoveredNode(found);
    canvas.style.cursor = found ? "pointer" : "default";
  }, []);

  const handleClick = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    for (const node of nodesRef.current) {
      const dx = x - node.x;
      const dy = y - node.y;
      if (Math.sqrt(dx * dx + dy * dy) < node.radius + 4) {
        setSelectedNode(node === selectedNode ? null : node);
        break;
      }
    }
  }, [selectedNode]);

  // Resize
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const observer = new ResizeObserver(() => {
      if (canvas.parentElement) {
        canvas.width = canvas.parentElement.clientWidth;
        canvas.height = canvas.parentElement.clientHeight;
      }
    });
    observer.observe(canvas.parentElement!);
    return () => observer.disconnect();
  }, []);

  return (
    <div className="relative w-full h-full">
      <canvas
        ref={canvasRef}
        width={800}
        height={600}
        onMouseMove={handleMouseMove}
        onClick={handleClick}
        className="w-full h-full"
      />

      <div className="absolute top-4 right-4 flex gap-2">
        <button
          onClick={() => setMode("microscope")}
          className={`px-3 py-1 rounded text-xs font-medium transition-colors ${
            mode === "microscope" ? "bg-primary text-primary-foreground" : "bg-card text-muted-foreground"
          }`}
        >
          🔬 Microscope
        </button>
        <button
          onClick={() => setMode("telescope")}
          className={`px-3 py-1 rounded text-xs font-medium transition-colors ${
            mode === "telescope" ? "bg-secondary text-secondary-foreground" : "bg-card text-muted-foreground"
          }`}
        >
          🔭 Telescope
        </button>
      </div>

      {selectedNode && (
        <div className="absolute bottom-4 left-4 w-72 glass rounded-xl p-4 animate-slide-in">
          <div className="flex justify-between items-start">
            <div>
              <h3 className="font-semibold text-sm">{selectedNode.label}</h3>
              <p className="text-xs text-muted-foreground">{selectedNode.type}</p>
            </div>
            <button
              onClick={() => setSelectedNode(null)}
              className="text-muted-foreground hover:text-foreground text-lg"
            >
              ×
            </button>
          </div>
          {selectedNode.tags && (
            <div className="flex flex-wrap gap-1 mt-2">
              {selectedNode.tags.map((tag) => (
                <span key={tag} className="px-2 py-0.5 rounded-full bg-border text-xs text-muted-foreground">
                  {tag}
                </span>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function getNodeColor(type: GraphNode["type"]): string {
  switch (type) {
    case "Agent": return "#6c63ff";
    case "Concept": return "#00d4aa";
    case "Thought": return "#ff6b6b";
    case "Claim": return "#ffaa00";
    case "WikiEntry": return "#8888aa";
    default: return "#555566";
  }
}
