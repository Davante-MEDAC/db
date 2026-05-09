default:
    just --list

# Runs the Development-Kit Container
dkc:
    docker pull ghcr.io/leoborai/dkc:latest
    docker run -it --rm \
        -v $(pwd):/app \
        -w /app \
        ghcr.io/leoborai/dkc:latest

# Runs the Development Server with Trunk
run:
    trunk serve

# Builds the project for release
release:
    trunk build --release

# Type-check the web crate (native target, fast)
check:
    cargo check -p qa --target wasm32-unknown-unknown

# Type-check all workspace members
check-all:
    cargo check --workspace --target wasm32-unknown-unknown
