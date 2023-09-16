run:
  RUST_LOG=info,html5ever=error cargo watch -q -c -w src/ -x run | bunyan

test:
  cargo watch -x test | bunyan

build:
  cargo build --release

mock:
  curl --compressed -o ./tests/mocks/towns-200.html \
  -H "User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0" \
  https://www.tibia.com/community/?subtopic=houses
