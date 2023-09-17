run:
  RUST_LOG=info,html5ever=error cargo watch -q -c -w src/ -x run | bunyan

test:
  cargo watch -x test | bunyan

build:
  cargo build --release
