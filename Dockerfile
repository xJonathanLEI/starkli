FROM rust:alpine AS build

RUN apk add --update alpine-sdk

COPY . /src
WORKDIR /src

RUN cargo build --release

FROM alpine

LABEL org.opencontainers.image.source=https://github.com/xJonathanLEI/starkli

COPY --from=build /src/target/release/starkli /usr/bin/

ENTRYPOINT [ "starkli" ]
