#!/bin/bash

cargo build --release

./wait-for-it.sh db:5432

diesel setup
diesel migration run

/app/target/release/colour-bot-v2