ARG RUST_VERSION=1.90.0
FROM rust:${RUST_VERSION}

RUN rustup self update
RUN rustup install stable
RUN rustup override set stable
RUN rustup update nightly
RUN rustup toolchain add nightly
RUN rustup component add rust-src rustfmt rust-docs clippy rust-analyzer rust-analysis
