FROM rustlang/rust:nightly

WORKDIR /app

ADD . /app 

RUN apt-get update &&\
    apt-get install -y\
    libcairo2-dev\
    libgtk-3-dev\
    libpango-1.0-0\
    libpangocairo-1.0-0\
    postgresql

RUN cargo install diesel_cli --no-default-features --features postgres

EXPOSE 80 443

RUN cargo build --release 

CMD ["/app/docker-start.sh"]