# Tubes2_RustDz

Visualisasi traversal Document Object Model (DOM) HTML menggunakan algoritma BFS dan DFS.

Live app: https://tubes2-rust-dz.vercel.app/

## Daftar Isi

- [Ringkasan Project](#ringkasan-project)
- [Fitur Utama](#fitur-utama)
- [Arsitektur](#arsitektur)
- [Tech Stack](#tech-stack)
- [Struktur Folder](#struktur-folder)
- [Menjalankan Secara Lokal](#menjalankan-secara-lokal)
- [Menjalankan Dengan Docker](#menjalankan-dengan-docker)
- [Dokumentasi API Backend](#dokumentasi-api-backend)
- [Deploy](#deploy)
- [Troubleshooting](#troubleshooting)

## Ringkasan Project

Project ini menerima input URL halaman HTML dan CSS selector, lalu:

1. Mengambil konten HTML dari URL.
2. Mem-parse HTML menjadi struktur tree DOM internal.
3. Melakukan traversal tree dengan BFS atau DFS.
4. Menemukan node yang cocok dengan CSS selector.
5. Menampilkan hasil traversal, performa eksekusi, dan visualisasi tree di frontend.

## Fitur Utama

- Traversal DOM dengan dua algoritma: BFS dan DFS.
- Pencocokan elemen dengan CSS selector.
- Visualisasi tree dan jalur traversal.
- Riwayat request terakhir di frontend.
- Endpoint health-check backend.
- CORS sudah diaktifkan untuk integrasi frontend-backend lintas origin.

## Arsitektur

Project menggunakan arsitektur terpisah frontend dan backend:

- Frontend (React + Vite + TypeScript)
	- UI input URL, selector, dan metode traversal.
	- Mengirim request ke backend endpoint POST /api/traverse.
	- Menampilkan hasil dan visualisasi tree.

- Backend (Rust + Axum)
	- Menyediakan REST API.
	- Parsing HTML ke tree internal.
	- Menjalankan BFS/DFS (termasuk mode concurrent pada kondisi tertentu).
	- Mengembalikan response JSON siap render untuk frontend.

## Tech Stack

- Backend
	- Rust 1.86
	- Axum
	- Tokio
	- Serde / Serde JSON
	- Reqwest
	- Tower HTTP (CORS)

- Frontend
	- React
	- TypeScript
	- Vite
	- Axios

- Container
	- Docker
	- Docker Compose

## Struktur Folder

```text
.
|-- backend/
|   |-- src/
|   |   |-- main.rs
|   |   `-- modules/
|   |-- Cargo.toml
|   `-- Dockerfile
|-- frontend/
|   |-- src/
|   |   |-- App.tsx
|   |   |-- api.ts
|   |   `-- components/
|   |-- package.json
|   `-- Dockerfile
|-- docker-compose.yml
`-- README.md
```

## Menjalankan Secara Lokal

### 1. Jalankan backend

Prerequisite:

- Rust toolchain (minimal sesuai Cargo.toml: Rust 1.86)

Command:

```bash
cd backend
cargo run
```

Backend default berjalan di:

- http://localhost:3000

Endpoint cepat cek:

- http://localhost:3000/health

### 2. Jalankan frontend

Prerequisite:

- Node.js 20+ (direkomendasikan)
- npm

Command:

```bash
cd frontend
npm install
npm run dev
```

Frontend default berjalan di:

- http://localhost:5173

Konfigurasi base URL API frontend:

- File frontend/src/api.ts menggunakan env VITE_API_BASE_URL.
- Default fallback adalah http://localhost:3000.

Contoh .env untuk frontend:

```env
VITE_API_BASE_URL=http://localhost:3000
```

## Menjalankan Dengan Docker

Jalankan dari root project:

```bash
docker compose up --build
```

Akses service:

- Frontend: http://localhost:8080
- Backend: http://localhost:3000

Hentikan container:

```bash
docker compose down
```

Jika hanya ingin backend:

```bash
cd backend
docker build -t tubes2-backend .
docker run --rm -p 3000:3000 -e PORT=3000 tubes2-backend
```

## Dokumentasi API Backend

Base URL lokal:

- http://localhost:3000

### GET /

Response:

```json
{
	"status": "ok",
	"message": "Server is running"
}
```

### GET /health

Response:

```json
{
	"status": "ok",
	"message": "Health check passed"
}
```

### POST /api/traverse

Request body:

```json
{
	"source_url": "https://example.com",
	"css_selector": "a",
	"method": "BFS"
}
```

Catatan:

- method hanya menerima BFS atau DFS.
- source_url atau html_content wajib diisi minimal salah satu.

Contoh success response:

```json
{
	"execution_time_us": 1234,
	"matched_nodes": [
		{
			"id": "7",
			"tag": "a",
			"class": "nav-link"
		}
	],
	"traversal_path": ["0", "1", "2", "7"],
	"tree_data": {
		"root_id": "0",
		"nodes": []
	}
}
```

Contoh error response:

```json
{
	"error": "method must be either BFS or DFS"
}
```

## Deploy

### Frontend

Frontend dapat di-deploy di Vercel:

- https://tubes2-rust-dz.vercel.app/

### Backend

Backend dapat di-deploy di Render:

- (https://tubes2-rustdz-backend.onrender.com)

## Troubleshooting

- 404 NOT_FOUND di Vercel:
	- Biasanya terjadi jika backend long-running dicoba langsung di Vercel tanpa adaptasi serverless.
	- Solusi: deploy backend di platform container, lalu arahkan VITE_API_BASE_URL ke backend tersebut.

- Gagal fetch URL sumber (403 atau timeout):
	- Beberapa situs memblokir request otomatis.
	- Coba URL lain atau gunakan html_content langsung melalui API.

- Frontend tidak bisa connect backend lokal:
	- Pastikan backend aktif di port 3000.
	- Pastikan VITE_API_BASE_URL sesuai environment yang digunakan.
