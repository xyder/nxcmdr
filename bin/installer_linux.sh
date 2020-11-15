#!/usr/bin/env bash


# package id for v0.2.3
pkg_id=4638919

curl https://gitlab.com/xyder/nxcmdr/-/package_files/$pkg_id/download --output /tmp/nxc.tar.gz

dest_dir=$HOME/.local/bin

mkdir -p $dest_dir

tar -xf /tmp/nxc.tar.gz -C $dest_dir ./nxc

rm /tmp/nxc.tar.gz
