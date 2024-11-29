FROM ubuntu:24.04

RUN apt-get update

RUN apt-get install -y \
    build-essential \
    git curl wget npm \
    unzip valgrind

RUN apt-get upgrade -y

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

RUN wget https://github.com/neovim/neovim/releases/latest/download/nvim-linux64.tar.gz \
    && tar -xvf nvim-linux64.tar.gz

ENV PATH="/root/.cargo/bin:/nvim-linux64/bin:${PATH}"
