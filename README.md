# Tubes2_RustDz

## Backend Docker Quick Start

Build image dari folder `backend`:

```bash
cd backend
docker build -t tubes2-backend .
```

Jalankan container:

```bash
docker run --rm -p 3000:3000 -e PORT=3000 tubes2-backend
```

Backend akan listen di `0.0.0.0:3000` di dalam container, jadi bisa diakses dari host lewat `http://localhost:3000`.