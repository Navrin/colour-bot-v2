FROM rustlang/rust:nightly

WORKDIR /app

RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add - &&\
    echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list

RUN apt-get update &&\
    apt-get install -y\
    libcairo2-dev\
    libgtk-3-dev\
    libpango-1.0-0\
    libpangocairo-1.0-0\
    postgresql\
    yarn

# installing in debug mode because it cuts down on build times
RUN cargo install diesel_cli --debug --no-default-features --features postgres

COPY . /app 
ONBUILD COPY . /app
EXPOSE 80 443

CMD ["/app/docker-start.sh"]