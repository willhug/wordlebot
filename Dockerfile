FROM rust:1.56
WORKDIR /usr/src/wordlebot
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=0 /usr/local/cargo/bin/wordlebot /usr/local/bin/wordlebot
CMD ["wordlebot"]