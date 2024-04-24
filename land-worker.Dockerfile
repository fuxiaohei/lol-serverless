FROM rust:1.77 as build

WORKDIR /usr/src/land-src
COPY . .
RUN rustup component add rustfmt
RUN cargo version
RUN cargo build -p land-worker --release

FROM debian:stable-slim
WORKDIR /opt/bin/
COPY --from=build /usr/src/land-src/target/release/land-worker /opt/bin/land-worker
EXPOSE 9844
CMD ["/opt/bin/land-worker","--verbose"]

