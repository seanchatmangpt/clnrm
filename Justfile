test:
    cargo make test

test-full:
    cargo make test-all

polish:
    cargo make fix

build:
    cargo make quick

clean:
    cargo clean

doc:
    cargo doc --workspace --no-deps

ci: polish test-full
