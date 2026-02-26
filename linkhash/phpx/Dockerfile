FROM rust:1.85-bookworm AS builder
WORKDIR /workspace

# Build deka release binary from monorepo source.
COPY deka/deka /workspace/deka
RUN cd /workspace/deka && cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /workspace/deka/target/release/deka /usr/local/bin/deka
COPY --from=builder /workspace/deka/target/release/deka-runtime /usr/local/bin/deka-runtime
COPY linkhash /app

ENV PORT=8530
EXPOSE 8530

CMD ["deka", "serve", "main.phpx"]
