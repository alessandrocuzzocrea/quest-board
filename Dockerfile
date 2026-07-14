FROM rust:slim-bookworm AS build
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && mkdir -p handlers repository models migrations
RUN cargo build --release 2>/dev/null || true
COPY backend/src src/
COPY backend/migrations migrations/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update -qq && apt-get install -y -qq ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/quest-board /usr/local/bin/
EXPOSE 3001
ENV DATABASE_URL=postgres://postgres:quest@localhost:5432/quest
WORKDIR /app
CMD ["quest-board"]
