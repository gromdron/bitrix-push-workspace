FROM rust:1.87-bullseye AS builder

WORKDIR /usr/src/push-server

RUN echo "deb http://deb.debian.org/debian bullseye main" > /etc/apt/sources.list && \
    echo "deb http://deb.debian.org/debian bullseye-updates main" >> /etc/apt/sources.list && \
    echo "deb http://security.debian.org/debian-security bullseye-security main" >> /etc/apt/sources.list

RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive \
    apt-get install --no-install-recommends --assume-yes \
      protobuf-compiler

COPY . .

RUN cargo install --path .

FROM debian:bullseye-slim

WORKDIR /usr/src/push-server

LABEL version="2.0" maintainer="Andrei Nikolaev <gromdron@yandex.ru>"

COPY --from=builder /usr/local/cargo/bin/bitrix-push-workspace /usr/local/bin/push-server

COPY --from=builder /usr/src/push-server/push_config.toml /usr/src/push-server/push_config.toml

CMD ["push-server"]