FROM oven/bun:1 AS frontend
WORKDIR /app
COPY app/package.json app/bun.lock ./
RUN bun install --frozen-lockfile
COPY app/ ./
RUN bun run build

FROM rust:slim-bookworm AS backend
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && mkdir -p handlers repository models migrations
RUN cargo build --release 2>/dev/null || true
COPY src src/
COPY migrations migrations/
COPY --from=frontend /app/dist app/dist/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update -qq && apt-get install -y -qq ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=backend /app/target/release/quest-board /usr/local/bin/
EXPOSE 3001
ENV DATABASE_URL=postgres://postgres:quest@localhost:5432/quest
WORKDIR /app
CMD ["quest-board"]
