FROM rust:1.86-slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --bin backend

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 appuser

WORKDIR /app

COPY --from=builder /app/target/release/backend /usr/local/bin/backend

ENV HOST=0.0.0.0
ENV PORT=3000

EXPOSE 3000

USER appuser

CMD ["/usr/local/bin/backend"]