# образ для запуска
FROM alpine:3.13.5
EXPOSE 5000/tcp
EXPOSE 5555/udp
COPY grpc_health_probe /bin/grpc_health_probe
RUN chmod +x /bin/grpc_health_probe
COPY cheetah-matches-relay /cheetah-matches-relay
RUN adduser -D service
USER service
CMD ["/cheetah-matches-relay"]
