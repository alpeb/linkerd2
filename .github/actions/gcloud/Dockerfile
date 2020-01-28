FROM alpine:3.11.3
RUN apk add bash && apk add curl
COPY entrypoint.sh /
ENTRYPOINT ["/entrypoint.sh"]