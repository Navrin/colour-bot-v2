#!/bin/bash
if [[ "$TRAVIS_BRANCH" == "master" ]];
then
    docker login -u $DOCKER_USERNAME -p $DOCKER_PASSWORD
    make build
    make upload
fi