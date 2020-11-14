#!/usr/bin/env bash


temp_release_dir=./releases/temp
target=x86_64-unknown-linux-gnu
target_archive=./releases/nxc-0.2.3-$target.tar.gz

# reset temp release directory
rm -fr $temp_release_dir && mkdir -p $temp_release_dir

# copy executable
cp ./target/release/nxc $temp_release_dir

# copy license and readme
cp README.md LICENSE $temp_release_dir

# create archive
tar -czvf $target_archive -C $temp_release_dir . > /dev/null

# cleanup
rm -fr $temp_release_dir

echo "Created release archive $target_archive"
