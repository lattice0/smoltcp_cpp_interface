FROM ubuntu:bionic

WORKDIR /home/project

RUN export DEBIAN_FRONTEND=noninteractive && apt-get update\
    && apt-get install -y curl build-essential cmake autoconf git m4 iproute2 clang

RUN curl --proto '=https' --tlsv1.2 -o rust.sh https://sh.rustup.rs\
    && /bin/bash rust.sh -y

ENV PATH="/root/.cargo/bin:${PATH}"
