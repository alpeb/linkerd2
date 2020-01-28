FROM alpine:3.11.3
RUN apt-get update && apt-get install -r --no-install-recommends curl
COPY entrypoint.sh /
ENTRYPOINT ["/entrypoint.sh"]