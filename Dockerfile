FROM rust:1.67

WORKDIR /usr/src/tibia-api

# Copy Cargo files
COPY ./Cargo.toml .
COPY ./Cargo.lock .

# Create fake main.rs file in src and build
RUN mkdir ./src && echo 'fn main() { println!("Dummy!"); }' > ./src/main.rs
RUN cargo build --release

COPY . .

RUN cargo install --path .

FROM rust:1.67

WORKDIR /usr/src/tibia-api
EXPOSE 7032
COPY static ./static

COPY --from=0 /usr/local/cargo/bin/tibia-api /usr/local/bin/tibia-api

CMD ["tibia-api"]
