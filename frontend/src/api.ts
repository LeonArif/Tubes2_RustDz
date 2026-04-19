import axios from 'axios';
import type { TraversalResponse } from "./types";

// port server Axum
const BASE_URL = 'http://localhost:3000'; 
// endpoint traversal di backend
const TRAVERSAL_ENDPOINT = '/api/traverse';

const apiClient = axios.create({
  baseURL: BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
  timeout: 10000, // Timeout 10 detik agar tidak loading jika server mati
});

// tipe untuk menangani error response dari backend
type BackendErrorPayload = {
  message?: string;
  error?: string;
};

// fungsi untuk mengirim data traversal
export const fetchTraversalData = async (
  sourceUrl: string,
  selector: string, 
  method: 'BFS' | 'DFS'
): Promise<TraversalResponse> => {
  try {
    // request endpoint POST /api/traverse
    const response = await apiClient.post<TraversalResponse>(TRAVERSAL_ENDPOINT, {
      source_url: sourceUrl,
      css_selector: selector,
      method: method,
    });
    return response.data;
  } catch (error) {
    // error handling untuk response error dari backend
    if (axios.isAxiosError(error) && error.response) {
      const data = error.response.data as BackendErrorPayload;
      const backendMessage = data?.message || data?.error;

      throw new Error(backendMessage || 'Terjadi kesalahan pada server Rust');
    }
    throw new Error('Gagal terhubung ke server. Pastikan backend Axum sudah menyala.');
  }
};