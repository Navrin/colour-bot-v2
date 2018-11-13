# web building step
FROM node as web

RUN npm install --global yarn

WORKDIR /app/colour-bot-site


RUN mkdir -p /app/colour-bot-site
COPY ./colour-bot-site/package.json .
COPY ./colour-bot-site/yarn.lock    .

RUN yarn

COPY ./colour-bot-site /app/colour-bot-site
RUN yarn build

# All cli tools the startup script needs.
FROM rustlang/rust:nightly as installing

RUN mkdir -p /cargo/bin
RUN cargo install --root /cargo/ diesel_cli --debug --no-default-features --features postgres
RUN ls /cargo/bin

# complining the apps dependencies first, then the src
FROM rustlang/rust:nightly as compiling

# RUN apt-get update &&
#     apt-get install -y\
#     libcairo2-dev\
#     libgtk-3-dev\
#     libpango-1.0-0\
#     libpangocairo-1.0-0\
#     postgresql

RUN USER=root cargo new --bin app

WORKDIR /app
COPY ./Makefile /app/Makefile

RUN make get_deps

COPY ./Cargo.lock .
COPY ./Cargo.toml .
RUN cargo build --release
RUN rm -r ./src

# actual build
COPY . /app 
COPY ./src ./src
RUN rm -f ./target/release/deps/colour_bot_v2*
RUN cargo build --release

FROM ubuntu AS final

RUN apt-get update && apt-get install make

WORKDIR /app

COPY --from=installing /cargo/bin/diesel /usr/local/bin/diesel
COPY --from=compiling /app/target/release/colour-bot-v2 .
COPY --from=web /app/colour-bot-site/build ./colour-bot-site/build
COPY . /app
COPY ./Makefile /app/Makefile

WORKDIR /app
RUN make get_deps

EXPOSE 80 443

CMD ["/app/docker-start.sh"]