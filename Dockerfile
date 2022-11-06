FROM rust:1.65

RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive \
    apt-get install --no-install-recommends --assume-yes \
      protobuf-compiler

WORKDIR /usr/src/push-server

COPY . .

RUN cargo install --path .

CMD ["push-server"]