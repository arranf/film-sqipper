FROM rust:1.54.0 AS builder
WORKDIR /usr/src/

RUN USER=root cargo new film-sqipper
WORKDIR /usr/src/film-sqipper
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY build ./build
COPY build.rs ./build.rs
RUN cargo build --release

# Bundle Stage
FROM gcr.io/distroless/cc-debian10
COPY --from=builder /usr/src/film-sqipper/target/release/film-sqipper /usr/local/bin/film-sqipper
CMD ["film-sqipper"]