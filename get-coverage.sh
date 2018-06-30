# runs a coverage tool from docker over the project

linux="false"

if [[ $OSTYPE == linux* ]];
then
    $linux="true"
fi


if [ -x $(command -v docker) ] && [ $linux == "false" ];
then
    docker run \
    --security-opt seccomp=unconfined \
    -v "$PWD:/volume" navrin/colour-bot-tarpaulin:latest
elif [[ $linux == "true" ]];
then
    cargo tarpaulin
else
    echo "Docker is not installed! Go install it from a package manager or docker's offical site!"
fi