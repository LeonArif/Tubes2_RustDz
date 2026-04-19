import { useState } from 'react';

//mMendefinisikan tipe ke App.tsx
interface ControlPanelProps {
  onTraverse: (html: string, selector: string, method: 'BFS' | 'DFS') => void;
  isLoading: boolean;
}

export default function ControlPanel({ onTraverse, isLoading }: ControlPanelProps) {
  // state lokal khusus untuk menyimpan ketikan user
  const [htmlContent, setHtmlContent] = useState('');
  const [cssSelector, setCssSelector] = useState('');
  const [method, setMethod] = useState<'BFS' | 'DFS'>('BFS');

  // ;ogika membaca file
  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (event) => {
      setHtmlContent(event.target?.result as string);
    };
    reader.readAsText(file); //mengubah file HTML menjadi string panjang
  };

  return (
    <div className="flex flex-col gap-4">
      {/* 1. upload File */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">Upload File HTML</label>
        <input 
          type="file" 
          accept=".html" 
          onChange={handleFileUpload}
          className="w-full border border-gray-300 p-2 rounded cursor-pointer"
        />
        {htmlContent && <p className="text-xs text-green-600 mt-1">File berhasil dimuat!</p>}
      </div>

      {/* 2. input CSS Selector */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">CSS Selector</label>
        <input 
          type="text" 
          placeholder="contoh: #header atau .btn"
          value={cssSelector}
          onChange={(e) => setCssSelector(e.target.value)}
          className="w-full border border-gray-300 p-2 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>

      {/* 3. pilihan BFS / DFS */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">Metode Pencarian</label>
        <select 
          value={method} 
          onChange={(e) => setMethod(e.target.value as 'BFS' | 'DFS')}
          className="w-full border border-gray-300 p-2 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value="BFS">Breadth-First Search (BFS)</option>
          <option value="DFS">Depth-First Search (DFS)</option>
        </select>
      </div>

      {/* 4. tombol Eksekusi */}
      <button 
        onClick={() => onTraverse(htmlContent, cssSelector, method)}
        disabled={isLoading}
        className={`mt-4 py-2 px-4 rounded font-bold text-white transition-colors
          ${isLoading ? 'bg-gray-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700'}`}
      >
        {isLoading ? 'Memproses...' : 'Cari Elemen'}
      </button>
    </div>
  );
}