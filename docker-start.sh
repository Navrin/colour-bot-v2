#!/bin/bash

./wait-for-it.sh db:5432

diesel setup
diesel migration run

./colour-bot-v2