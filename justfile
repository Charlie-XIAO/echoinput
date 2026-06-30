set windows-shell := ["powershell"]
set shell := ["bash", "-cu"]

alias f := fmt
alias l := lint

_default:
    just --list -u

fmt:
    cargo +nightly fmt
    taplo fmt

lint:
    cargo clippy --fix --allow-dirty --allow-staged -- -D warnings

ci: fmt lint
