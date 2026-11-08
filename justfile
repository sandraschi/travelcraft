set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

# travelcraft automation (Pure-Rust ArtCraft pattern)

default:
    @just --list

# Strict lint gate across all workspace members
check:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo clippy --workspace --all-targets --all-features -- -D warnings

# Unit + integration tests
test:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo test --workspace

# Formatting gate
fmt-check:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo fmt --all --check

# All gates in one command
ci: fmt-check check test

# MCP stdio server (a client normally starts this; useful for a manual probe)
run-mcp:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo run -q -p travelcraft-mcp

# CLI smoke: editions, then a 3 day Austria itinerary
run-cli:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo run -q -p travelcraft-app --bin travelcraft-cli -- countries; cargo run -q -p travelcraft-app --bin travelcraft-cli -- itinerary AT 3

# Production build
build-release:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo build --release --workspace