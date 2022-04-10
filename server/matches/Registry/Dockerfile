# образ для запуска
FROM alpine:3.13.5
EXPOSE 5000/tcp
EXPOSE 3000/tcp
COPY grpc_health_probe /bin/grpc_health_probe
RUN chmod +x /bin/grpc_health_probe
COPY cheetah-matches-registry /cheetah-matches-registry
RUN adduser -D service
USER service
CMD ["/cheetah-matches-registry"]
