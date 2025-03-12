FROM ubuntu:24.04 AS build

# Install dependencies.
RUN apt-get update && apt-get install -y \
    build-essential \
    gdb \
    qemu-system-x86 \
    rustup \
    && rm -rf /var/lib/apt/lists/*
