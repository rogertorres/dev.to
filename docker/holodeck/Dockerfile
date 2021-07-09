FROM rust:1.49 as builder

# create a new empty shell project
RUN USER=root cargo new --bin holodeck
WORKDIR /holodeck
RUN USER=root touch "src/lib.rs"

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/holodeck*
RUN cargo build --release
RUN cargo install --path .

CMD ["holodeck"]
# CMD ["./target/release/holodeck"]