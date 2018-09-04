FROM rustlang/rust:nightly

WORKDIR /app

RUN apt-get update &&\
    apt-get install -y\
    libcairo2-dev\
    libgtk-3-dev\
    libpango-1.0-0\
    libpangocairo-1.0-0\
    postgresql

# installing in debug mode because it cuts down on build times
RUN cargo install diesel_cli --debug --no-default-features --features postgres

COPY . /app 
ONBUILD COPY . /app
EXPOSE 80 443

CMD ["/app/docker-start.sh"]