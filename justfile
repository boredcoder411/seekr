debug:
  GTK_DEBUG=interactive cargo run --release

run:
  cargo run --release

build:
  cargo i18n && cargo build --release
