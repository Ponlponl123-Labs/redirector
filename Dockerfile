FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN cargo build --release
RUN mv ./target/release/redirector ./app

FROM scratch AS runtime
WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/app"]
