# Build and Test

## for evi Rust codes

    cargo build --verbose
    cargo test --verbose

## for e2e tests

Install e2e test requirements.

    pip install -r e2e/requirements.txt

Run e2e tests.

    pytest e2e --verbose
