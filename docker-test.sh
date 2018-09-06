#!/bin/bash

./wait-for-it.sh testdb:5432

if ! [ -x "$(command -v diesel)" ]; then
    cargo install diesel_cli --debug --no-default-features --features postgres
fi

diesel setup
diesel migration run

cargo test