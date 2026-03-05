FROM rust:1.88 AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY ecs ./ecs
COPY ecs_macros ./ecs_macros
COPY engine_sim ./engine_sim
COPY engine_replay ./engine_replay
COPY scheduler ./scheduler
COPY engine ./engine
COPY grid ./grid
COPY engine_net ./engine_net
COPY engine_net_quic ./engine_net_quic
COPY cavern_hunt ./cavern_hunt
COPY grotto_online ./grotto_online
COPY grotto_fleet_control ./grotto_fleet_control
COPY grotto_server ./grotto_server
COPY grotto_client ./grotto_client
COPY assets ./assets
COPY game ./game

RUN cargo build --release -p grotto_server

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/grotto_server /usr/local/bin/grotto_server
COPY --from=builder /app/assets ./assets
COPY --from=builder /app/game ./game

EXPOSE 7000/udp

CMD ["/usr/local/bin/grotto_server", "--config", "/app/config/server.ron"]
