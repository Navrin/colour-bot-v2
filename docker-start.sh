#!/bin/bash

(
    cd ./colour-bot-site

    yarn build
)

cargo build --release

./wait-for-it.sh db:5432

diesel setup
diesel migration run

/app/target/release/colour-bot-v2