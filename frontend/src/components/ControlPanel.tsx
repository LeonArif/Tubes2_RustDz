import { useState } from "react";

// Mendefinisikan props dari App.tsx
interface ControlPanelProps {
  onTraverse: (
    sourceUrl: string,
    selector: string,
    method: "BFS" | "DFS",
  ) => void;
  isLoading: boolean;
}

export default function ControlPanel({
  onTraverse,
  isLoading,
}: ControlPanelProps) {
  // state lokal untuk input URL, selector, dan metode pencarian
  const [sourceUrl, setSourceUrl] = useState("");
  const [cssSelector, setCssSelector] = useState("");
  const [method, setMethod] = useState<"BFS" | "DFS">("BFS");

  const quickSamples = [
    { label: "Example", url: "https://example.com", selector: "h1" },
    { label: "MDN", url: "https://developer.mozilla.org", selector: "a" },
    { label: "Vite", url: "https://vite.dev", selector: "nav a" },
  ] as const;

  const isFormIncomplete = !sourceUrl.trim() || !cssSelector.trim();

  const applySample = (sample: (typeof quickSamples)[number]) => {
    setSourceUrl(sample.url);
    if (!cssSelector.trim()) {
      setCssSelector(sample.selector);
    }
  };

  return (
    <div className="control-panel">
      {/* 1. input URL */}
      <div className="field">
        <label className="field-label">URL Halaman HTML</label>
        <input
          type="url"
          placeholder="https://example.com"
          value={sourceUrl}
          onChange={(e) => setSourceUrl(e.target.value)}
          className="field-input"
        />
        <p className="field-help">
          URL akan diparsing menjadi DOM tree
        </p>
        <div className="chip-list">
          {quickSamples.map((sample) => (
            <button
              key={sample.label}
              type="button"
              onClick={() => applySample(sample)}
              className="chip-button"
            >
              {sample.label}
            </button>
          ))}
        </div>
      </div>

      {/* 2. input CSS Selector */}
      <div className="field">
        <label className="field-label">CSS Selector</label>
        <input
          type="text"
          placeholder="contoh: #header atau .btn"
          value={cssSelector}
          onChange={(e) => setCssSelector(e.target.value)}
          className="field-input"
        />
      </div>

      {/* 3. pilihan BFS / DFS */}
      <div className="field">
        <label className="field-label">Metode Pencarian</label>
        <select
          value={method}
          onChange={(e) => setMethod(e.target.value as "BFS" | "DFS")}
          className="field-input"
        >
          <option value="BFS">Breadth-First Search (BFS)</option>
          <option value="DFS">Depth-First Search (DFS)</option>
        </select>
      </div>

      {/* 4. tombol Eksekusi */}
      <button
        onClick={() => onTraverse(sourceUrl, cssSelector, method)}
        disabled={isLoading || isFormIncomplete}
        className="execute-button"
      >
        {isLoading ? "Memproses..." : "Cari Elemen"}
      </button>
    </div>
  );
}
