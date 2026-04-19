import { useState } from "react";
import { fetchTraversalData } from "./api";
import type { TraversalResponse } from "./types";
import ControlPanel from "./components/ControlPanel";
// import GraphViewer from './components/GraphViewer'; // (Ini nanti diisi oleh Role B)

export default function App() {
  // state hasil traversal
  const [result, setResult] = useState<TraversalResponse | null>(null);

  // state untuk status proses (loading & error)
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  // fungsi utama yang dipanggil saat tombol diklik
  const handleStartTraversal = async (
    html: string,
    selector: string,
    method: "BFS" | "DFS",
  ) => {
    // validasi input sebelum memulai proses
    if (!html.trim()) {
      setError("Silakan upload file HTML terlebih dahulu.");
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
      const data = await fetchTraversalData(html, selector, method);
      setResult(data); // simpan hasil sukses ke stat!
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
    <div className="flex h-screen w-full bg-gray-50 font-sans">
      {/* sisi kiri: panel control */}
      <div className="w-1/3 p-6 bg-white border-r border-gray-200 flex flex-col gap-4 shadow-sm z-10">
        <h1 className="text-2xl font-bold text-gray-800">DOM Traversal TB2</h1>

        {/* fungsi eksekusi oleh komponen control panel */}
        <ControlPanel onTraverse={handleStartTraversal} isLoading={isLoading} />

        {/* indikator UI */}
        {error && (
          <div className="p-3 bg-red-100 text-red-700 rounded-md border border-red-200">
            {error}
          </div>
        )}

        {result && (
          <div className="p-4 bg-green-50 text-green-800 rounded-md border border-green-200 mt-auto">
            <h3 className="font-bold mb-2">Pencarian Berhasil!</h3>
            <p>
              Waktu Eksekusi: <b>{result.execution_time_us} µs</b>
            </p>
            <p>
              Node Ditemukan: <b>{result.matched_nodes.length}</b>
            </p>
          </div>
        )}
      </div>

      {/* sisi kanan: graph viewer */}
      <div className="w-2/3 bg-gray-100 relative flex items-center justify-center">
        {isLoading ? (
          <div className="text-xl animate-pulse text-gray-500">
            Menganalisis Document Object Model
          </div>
        ) : result ? (
          <div className="text-green-600 font-bold">
            Data berhasil didapat! Siap dioper ke komponen GraphViewer
          </div>
        ) : (
          <div className="text-gray-400">
            Silakan masukkan parameter dan mulai pencarian
          </div>
        )}
      </div>
    </div>
  );
}
