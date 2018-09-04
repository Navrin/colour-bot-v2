#!/bin/bash

./wait-for-it.sh testdb:5432

cargo install diesel_cli --debug --no-default-features --features postgres

diesel setup
diesel migration run

cargo test