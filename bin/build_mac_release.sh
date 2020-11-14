#!/usr/bin/env bash


docker build -f Dockerfile.macosx -t nxcmdr-macosx .

target_volume=nxcmdr-target

echo "Delete volume if exists .."
docker volume rm $target_volume || true

docker volume create $target_volume
docker run --rm -v $target_volume:/build/target -it nxcmdr-macosx
