#!/usr/bin/env bash


temp_release_dir=./releases/temp
target=x86_64-apple-darwin
target_archive=./releases/nxc-0.2.4-$target.tar.gz

# reset temp release directory
rm -fr $temp_release_dir && mkdir -p $temp_release_dir

# copy executable from docker volume
docker container create --name interim -v nxcmdr-target:/root tianon/true > /dev/null
docker cp interim:/root/$target/release/nxc $temp_release_dir > /dev/null
docker rm interim > /dev/null

# copy license and readme
cp README.md LICENSE $temp_release_dir

# create archive
tar -czvf $target_archive -C $temp_release_dir . > /dev/null

# cleanup
rm -fr $temp_release_dir

echo "Created release archive $target_archive"
