# образ для запуска
FROM alpine:3.13.5
LABEL org.opencontainers.image.source="https://github.com/cheetah-game-platform/platform"
EXPOSE 5000/tcp
COPY grpc_health_probe /bin/grpc_health_probe
RUN chmod +x /bin/grpc_health_probe
COPY cheetah-system-compatibility-server /cheetah-system-compatibility-server
RUN adduser -D service
USER service
CMD ["/cheetah-system-compatibility-server"]
