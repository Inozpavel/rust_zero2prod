FROM rust:1.79-slim as base

ENV SQLX_OFFLINE_DIR=.sqlx

WORKDIR /code

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