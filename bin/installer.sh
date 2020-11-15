#!/usr/bin/env bash


# get package id for v0.2.3
if [[$1 == 'apple' ]]; then
    # apple package id
    pkg_id=4638919
else
    # linux package id
    pkg_id=4638918
fi

curl https://gitlab.com/xyder/nxcmdr/-/package_files/$pkg_id/download --output /tmp/nxc.tar.gz


dest_dir=$HOME/.local/bin

mkdir -p $dest_dir

tar -xf /tmp/nxc.tar.gz -C $dest_dir ./nxc

ls $dest_dir

rm /tmp/nxc.tar.gz
