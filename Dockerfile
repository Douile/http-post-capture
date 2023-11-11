FROM rust:alpine AS build

RUN apk add --no-cache musl-dev

RUN mkdir -p /src /out
WORKDIR /src

COPY Cargo.toml Cargo.lock /src
COPY src/main.rs /src/src/main.rs

ENV CARGO_TARGET_DIR="/out"
ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN cargo build --release

FROM scratch

COPY --from=build /lib/ld-musl-x86_64.so.1 /lib/ld-musl-x86_64.so.1
COPY --from=build /usr/lib/libgcc_s.so.1 /usr/lib/libgcc_s.so.1

COPY --from=build /out/release/http-post-capture /usr/bin/http-post-capture

EXPOSE 8000/tcp

VOLUME /output

CMD [ "-l", "0.0.0.0:8000", "-o", "/output" ]
ENTRYPOINT [ "/usr/bin/http-post-capture" ]
