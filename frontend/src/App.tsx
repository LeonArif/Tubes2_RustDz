import { useState } from "react";
import { fetchTraversalData } from "./api";
import type { TraversalResponse } from "./types";
import ControlPanel from "./components/ControlPanel";
import "./App.css";

// tipe untuk menyimpan riwayat traversal yang berhasil
type TraversalHistoryItem = {
  id: string;
  sourceUrl: string;
  selector: string;
  method: "BFS" | "DFS";
  executionTimeUs: number;
  matchedCount: number;
  createdAt: string;
};

export default function App() {
  // state hasil traversal
  const [result, setResult] = useState<TraversalResponse | null>(null);
  const [history, setHistory] = useState<TraversalHistoryItem[]>([]);

  // state untuk status proses (loading & error)
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  // fungsi utama yang dipanggil saat tombol diklik
  const handleStartTraversal = async (
    sourceUrl: string,
    selector: string,
    method: "BFS" | "DFS",
  ) => {
    // validasi input sebelum memulai proses
    if (!sourceUrl.trim()) {
      setError("URL HTML tidak boleh kosong.");
      return;
    }

    try {
      new URL(sourceUrl);
    } catch {
      setError("Format URL tidak valid. Contoh: https://example.com");
      return;
    }

    if (!selector.trim()) {
      setError("CSS Selector tidak boleh kosong.");
      return;
    }

    setIsLoading(true);
    setError(null); // reset error setiap mulai pencarian baru
    setResult(null); // reset hasil lama

    try {
      // memanggil API
      const data = await fetchTraversalData(sourceUrl, selector, method);
      setResult(data); // simpan hasil sukses ke stat!
      setHistory((prev) => {
        const nextEntry: TraversalHistoryItem = {
          id: crypto.randomUUID(),
          sourceUrl,
          selector,
          method,
          executionTimeUs: data.execution_time_us,
          matchedCount: data.matched_nodes.length,
          createdAt: new Date().toLocaleTimeString("id-ID", {
            hour: "2-digit",
            minute: "2-digit",
          }),
        };

        return [nextEntry, ...prev].slice(0, 5);
      });
    } catch (err: unknown) {
      setError(
        err instanceof Error
          ? err.message
          : "Terjadi kesalahan tidak diketahui",
      ); // tampilkan pesan error
    } finally {
      setIsLoading(false); // matikan loading, apa pun hasilnya
    }
  };

  return (
    <div className="app-shell">
      <div className="app-grid">
        <aside className="panel panel-controls">
          <div className="panel-title-wrap">
            <h1 className="panel-title">
              HTML Document Object Model Traversal Explorer
            </h1>
            <p className="panel-subtitle">
              Masukkan URL halaman, selector CSS, lalu pilih algoritma BFS atau
              DFS
            </p>
          </div>

          <ControlPanel
            onTraverse={handleStartTraversal}
            isLoading={isLoading}
          />

          {error && <div className="alert alert-error">{error}</div>}

          {result && (
            <div className="alert alert-success push-bottom">
              <h3 className="summary-title">Hasil Traversal</h3>
              <p>
                Waktu Eksekusi: <b>{result.execution_time_us} µs</b>
              </p>
              <p>
                Node Ditemukan: <b>{result.matched_nodes.length}</b>
              </p>
            </div>
          )}
        </aside>

        <main className="panel panel-results">
          <div className="status-area">
            {isLoading ? (
              <div className="status status-loading">
                <div className="status-heading">
                  Menganalisis Document Object Model
                </div>
                <div className="status-subheading">Mohon ditunggu...</div>
              </div>
            ) : result ? (
              <div className="status status-success">
                <div className="status-heading">Data berhasil didapatkan</div>
                <div className="status-subheading">
                  (visualisasi tree)
                </div>
              </div>
            ) : (
              <div className="status status-idle">
                <div className="status-heading">Belum ada hasil</div>
                <div className="status-subheading">
                  Masukkan parameter lalu klik Cari Elemen
                </div>
              </div>
            )}
          </div>

          <section className="history-card">
            <div className="history-head">
              <h2 className="history-title">Riwayat Request</h2>
              <span className="history-note">Maks. 5 terakhir</span>
            </div>

            {history.length === 0 ? (
              <p className="history-empty">Belum ada request yang berhasil.</p>
            ) : (
              <ul className="history-list">
                {history.map((item) => (
                  <li key={item.id} className="history-item">
                    <div className="history-item-top">
                      <span className="history-url" title={item.sourceUrl}>
                        {item.sourceUrl}
                      </span>
                      <span className="history-time">{item.createdAt}</span>
                    </div>
                    <div className="history-meta">
                      Selector: {item.selector} | Metode: {item.method} | Match:{" "}
                      {item.matchedCount} | {item.executionTimeUs} µs
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </section>
        </main>
      </div>
    </div>
  );
}
