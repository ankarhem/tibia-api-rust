FROM rust:1.67

WORKDIR /usr/src/tibia-api
EXPOSE 7032

# Copy Cargo files
COPY ./Cargo.toml .
COPY ./Cargo.lock .

# Create fake main.rs file in src and build
RUN mkdir ./src && echo 'fn main() { println!("Dummy!"); }' > ./src/main.rs
RUN cargo build --release

COPY . .

RUN cargo install --path .
CMD ["tibia-api"]
