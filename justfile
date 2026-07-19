build-parser:
    #!/usr/bin/env sh
    set -xe

    cd crates/kon-parser
    cargo build --release
    cp target/release/libkon_parser.dylib out/
    cargo run --bin bindgen --features bindgen generate --language python --out-dir out --library target/release/libkon_parser.dylib

    # testing
    python3 test.py


venv:
    source .venv/bin/activate
