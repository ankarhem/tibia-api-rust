FROM rust:1.67

WORKDIR /usr/src/tibia-api
COPY . .

RUN cargo test --release
RUN rm -rf tests
RUN cargo install --path .

EXPOSE 7032

CMD ["tibia-api"]
