FROM debian:bookworm AS base
MAINTAINER abj.dev@gmx.com

# Avoid interactive cli blockers
ENV DEBIAN_FRONTEND noninteractive

# Install basic stuff
RUN apt-get update -yq \
    && apt-get install -yq --no-install-recommends apt-utils

RUN apt-get install -yq --no-install-recommends \
    apt-transport-https lsb-release ca-certificates gnupg pkg-config \
    vim curl unzip build-essential \
    gcc make clang libssl-dev

# Clean up apt caches
RUN apt-get clean \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /var/cache/apt/*



FROM base AS demo
RUN mkdir -p /service/service_demo

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add Rust to path
ENV PATH="/root/.cargo/bin:$PATH"

# Build a release that can be started automatically
COPY service_demo /service/service_demo_rel
RUN cd /service/service_demo_rel && cargo build --release
COPY service_demo/.env.example /service/service_demo_rel/.env

WORKDIR /service/service_demo
CMD ["/service/service_demo/bin/startup.sh"]


