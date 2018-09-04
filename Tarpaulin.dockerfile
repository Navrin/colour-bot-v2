FROM navrin/colour-bot-v2-base:latest

RUN apt-get install -y libssl-dev pkg-config cmake zlib1g-dev
RUN RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install cargo-tarpaulin --debug

CMD ["cargo", "tarpaulin"]