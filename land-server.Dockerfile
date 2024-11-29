FROM rust:1.82 AS build

WORKDIR /usr/src/land-server
COPY . .
RUN rustup component add rustfmt
RUN bash /usr/src/land-server/scripts/land-server-deps.sh
RUN cargo version
RUN cargo build -p land-server --release

FROM debian:stable-slim
WORKDIR /opt/bin/
RUN \
  apt-get update && \
  apt-get install -y ca-certificates && \
  apt-get clean
COPY --from=build /usr/src/land-server/target/release/land-server /opt/bin/land-server
COPY --from=build /usr/src/land-server/wizer-v7.0.5-x86_64-linux /opt/bin/wizer
EXPOSE 9840
CMD ["/opt/bin/land-server","--verbose"]