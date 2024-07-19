FROM rust:1.79 as base

WORKDIR /code

COPY . .

RUN cargo b -r

ENTRYPOINT ["sleep", "infinity"]