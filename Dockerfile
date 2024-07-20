FROM rust:1.79-slim as base

ENV SQLX_OFFLINE_DIR=.sqlx

WORKDIR /code

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN mkdir src && touch ./src/main.rs

RUN cargo fetch --locked

COPY . .

RUN cargo b -r

FROM debian:bookworm-slim as release

WORKDIR /app

COPY --from=base /code/target/release/zero2prod ./zero2prod
COPY --from=base /code/migrations ./migrations
COPY --from=base /code/configuration.toml ./configuration.toml

ENV APP_ENVIRONMENT=Production
#ENTRYPOINT ["sleep", "infinity"]
ENTRYPOINT ["./zero2prod"]