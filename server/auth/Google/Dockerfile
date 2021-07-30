# определяем зависимости для кеширования
FROM akviring/rust-builder:1.54.0-01 as planner
WORKDIR planner
COPY server/auth/Cerberus/ server/auth/Cerberus/
COPY server/Microservice server/Microservice
COPY server/auth/User/ server/auth/User/
COPY server/auth/Google/ server/auth/Google/
RUN cd server/auth/Google/ && cargo chef prepare --recipe-path recipe.json

# собираем зависимости
FROM akviring/rust-builder:1.54.0-01 as cacher
WORKDIR cache
COPY proto/auth/ proto/auth/
COPY server/Microservice server/Microservice
COPY server/auth/User/ server/auth/User/
COPY server/auth/Cerberus/ server/auth/Cerberus/
COPY --from=planner /planner/server/auth/Google/recipe.json server/auth/Google/recipe.json
RUN cd server/auth/Google/ && cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.54.0-01 as builder
WORKDIR build
COPY proto/auth/ proto/auth/
COPY server/Microservice server/Microservice
COPY server/auth/User/ server/auth/User/
COPY server/auth/Cerberus server/auth/Cerberus
COPY server/auth/Google/ server/auth/Google/
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/auth/Google/target/ server/auth/Google/target/
RUN cd server/auth/Google/ && cargo build --offline --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/auth/Google/target/x86_64-unknown-linux-musl/release/cheetah-auth-google /service
RUN adduser -D service
USER service
CMD ["/service"]