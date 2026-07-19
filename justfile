build-parser:
    #!/usr/bin/env sh
    set -xe

    cd crates/kon-parser
    cargo build --release || exit 1
    cp target/release/libkon_parser.dylib out/
    cargo run --bin bindgen --features bindgen generate --language python --out-dir out --library target/release/libkon_parser.dylib

    # testing
    python3 test.py || exit 1

    mkdir -p ../../konc/include
    cp -r out/{*.py,*.dylib} ../../konc/include

venv:
    source .venv/bin/activate
