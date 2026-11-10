# syntax=docker/dockerfile:1

FROM rust:1.99-slim-trixie AS build

WORKDIR /usr/local/src/app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/src/app/target \
    cargo build --locked --release \
 && cp target/release/mazzacunijo /usr/local/bin/

FROM debian:trixie-slim

COPY --from=build /usr/local/bin/mazzacunijo /usr/local/bin/

USER nobody
STOPSIGNAL SIGINT
ENTRYPOINT ["/usr/local/bin/mazzacunijo"]
