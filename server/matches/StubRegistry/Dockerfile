FROM registry.dev.cheetah.games/cheetah/rust-builder-image:1.0.6 as builder
ARG RUSTFLAGS_ARG
COPY . .
RUN cd server/ && cargo fetcher --url file:///`pwd`/../.cache/crates/ --include-index sync
RUN cd server/matches/StubRegistry && RUSTFLAGS="$RUSTFLAGS_ARG" cargo build --offline --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5000/tcp
COPY server/grpc_health_probe /bin/grpc_health_probe
RUN chmod +x /bin/grpc_health_probe
COPY --from=builder /server/target/x86_64-unknown-linux-musl/release/cheetah-matches-stub-registry /service
RUN adduser -D service
USER service
CMD ["/service"]