FROM rust:alpine AS builder

## Use Alpine mirrors for faster package downloads in China (uncomment if needed)
# RUN echo -e "https://mirrors.tuna.tsinghua.edu.cn/alpine/v$(cat /etc/alpine-release | cut -d'.' -f1-2)/main/\nhttps://mirrors.tuna.tsinghua.edu.cn/alpine/v$(cat /etc/alpine-release | cut -d'.' -f1-2)/community/" > /etc/apk/repositories

## Setting up Rust mirrors for faster builds in China (uncomment if needed)
# ENV RUSTUP_DIST_SERVER=https://mirrors.tuna.tsinghua.edu.cn/rustup
# ENV RUSTUP_UPDATE_ROOT=https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup
# 
# RUN mkdir -vp ${CARGO_HOME:-$HOME/.cargo} \
#     && cat << EOF | tee -a ${CARGO_HOME:-$HOME/.cargo}/config.toml
# [source.crates-io]
# replace-with = 'mirror'
# [source.mirror]
# registry = "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/"
# [registries.mirror]
# index = "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/"
# EOF

RUN apk add --no-cache musl-dev protoc protobuf-dev

WORKDIR /app
COPY . ./

RUN cargo build --release


FROM alpine:latest

WORKDIR /app
COPY --from=builder /app/target/release/ ./

EXPOSE 1234
EXPOSE 1235
EXPOSE 3000
ENV RUST_LOG=info
RUN mkdir ./output

CMD ["./gps_location_server"]