# Tubes2_RustDz

## Docker Quick Start

Jalankan dari root project:

```bash
docker compose up --build
```

Lalu buka:

```bash
frontend: http://localhost:8080
backend: http://localhost:3000
```

Kalau hanya mau backend:

```bash
cd backend
docker build -t tubes2-backend .
docker run --rm -p 3000:3000 -e PORT=3000 tubes2-backend
```

Backend akan listen di `0.0.0.0:3000` di dalam container, jadi bisa diakses dari host lewat `http://localhost:3000`.