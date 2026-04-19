import { useState } from "react";
import type { TraversalResponse } from "./types";
import "./App.css";

function App() {
  // state input
  const [htmlContent, setHtmlContent] = useState("");
  const [cssSelector, setCssSelector] = useState("");
  const [method, setMethod] = useState<"BFS" | "DFS">("BFS");

  // state output server
  const [result, setResult] = useState<TraversalResponse | null>(null);

  // state UI
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleMockRun = () => {
    setIsLoading(true);
    setError(null);

    // Temporary local mock (no backend integration)
    setResult({
      execution_time_us: 0,
      matched_nodes: [],
      traversal_path: [],
      tree_data: {
        root: "html",
      },
    });

    setIsLoading(false);
  };

  return (
    <>
      <section id="center">
        <header style={{ marginBottom: "1rem", textAlign: "center" }}>
          <h1>Document Object Model Traversal Visualizer</h1>
          <p>Analyze HTML nodes with BFS or DFS using CSS selectors</p>
        </header>

        <div style={{ marginTop: "1rem", width: "100%", maxWidth: "520px" }}>
          <textarea
            placeholder="Paste HTML content"
            value={htmlContent}
            onChange={(e) => setHtmlContent(e.target.value)}
            rows={4}
            style={{ width: "100%", marginBottom: "0.5rem" }}
          />
          <input
            type="text"
            placeholder="CSS selector (e.g. .item > p)"
            value={cssSelector}
            onChange={(e) => setCssSelector(e.target.value)}
            style={{ width: "100%", marginBottom: "0.5rem" }}
          />
          <select
            value={method}
            onChange={(e) => setMethod(e.target.value as "BFS" | "DFS")}
            style={{ width: "100%", marginBottom: "0.5rem" }}
          >
            <option value="BFS">BFS</option>
            <option value="DFS">DFS</option>
          </select>

          <button
            className="counter"
            onClick={handleMockRun}
            disabled={isLoading || !htmlContent.trim() || !cssSelector.trim()}
          >
            {isLoading ? "Running..." : `Run ${method}`}
          </button>

          {error && (
            <p style={{ color: "#c00", marginTop: "0.5rem" }}>{error}</p>
          )}
          {result && (
            <p style={{ marginTop: "0.5rem" }}>
              Done in {result.execution_time_us} us, matched{" "}
              {result.matched_nodes.length} nodes.
            </p>
          )}
        </div>
      </section>

      <div className="ticks"></div>

      <div className="ticks"></div>
      <section id="spacer"></section>
    </>
  );
}

export default App;
