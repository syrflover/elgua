FROM ubuntu:jammy

RUN sed -i.bak -re \
    "s/([a-z]{2}.)?archive.ubuntu.com|security.ubuntu.com/mirror.kakao.com/g" \
    /etc/apt/sources.list

RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y ca-certificates ffmpeg python3

ARG BINARY_FILE
ARG CFG_FILE

COPY $BINARY_FILE /elgua
COPY $CFG_FILE /cfg.json

ENTRYPOINT [ "/elgua" ]
