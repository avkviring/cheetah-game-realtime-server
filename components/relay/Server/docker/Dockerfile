FROM debian:stable-20200607-slim
COPY cheetah_relay /cheetah_relay
RUN chmod +x /cheetah_relay
EXPOSE 5050/tcp 5050/udp 8080/tcp
ENTRYPOINT /cheetah_relay