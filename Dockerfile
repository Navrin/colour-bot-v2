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

FROM rustlang/rust:nightly as compiling

RUN cargo install diesel_cli --debug --no-default-features --features postgres

RUN USER=root cargo new --bin app
WORKDIR /app

RUN apt-get update &&\
    apt-get install -y\
    libcairo2-dev\
    libgtk-3-dev\
    libpango-1.0-0\
    libpangocairo-1.0-0\
    postgresql

COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN cargo build --release
RUN rm -r ./src

# actual build
COPY . /app 
COPY ./src ./src
RUN rm ./target/release/deps/colour_bot_v2*
RUN cargo build --release


COPY --from=web /app/colour-bot-site/build /app/colour-bot-site/build

EXPOSE 80 443
WORKDIR /app

CMD ["/app/docker-start.sh"]