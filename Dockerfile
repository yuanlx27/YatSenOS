FROM ubuntu:24.04 AS build

# Install dependencies.
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    iputils-ping \
    gdb \
    qemu-system-x86 \
    rustup \
    tmux \
    && rm -rf /var/lib/apt/lists/*

# vim:et:ft=dockerfile:sw=4:ts=4
