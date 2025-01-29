FROM rust:latest AS build
COPY . /home/build
WORKDIR /home/build

RUN apt-get install -y libssl-dev pkg-config

WORKDIR /home/build/
RUN  cargo build --release --all-features

LABEL org.label-schema.name="CS-25-303" \
    org.label-schema.vendor="wyatt-herkamp" \
    org.label-schema.schema-version="1.0" \
    org.label-schema.url="https://github.com/VCU-CS-Capstone/CS-25-303-SON-clinicians"

# The Final Image
FROM debian:bookworm-slim
RUN apt-get update -y && apt-get -y install libssl-dev openssl

RUN mkdir -p /opt/cs_25_303
RUN mkdir -p /app
COPY --from=build /home/build/target/release/cs25-303-backend /app/cs25-303-backend
COPY --from=build /home/build/entrypoint.sh /app/entrypoint.sh

WORKDIR /opt/cs_25_303
ENTRYPOINT ["/bin/sh", "/app/entrypoint.sh"]
CMD []