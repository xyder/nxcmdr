#!/usr/bin/env bash

VERSION='0.2.5'
dest_dir=$HOME/.local/bin
tmp_archive=/tmp/nxc.tar.gz

curl -L "https://gitlab.com/xyder/nxcmdr/-/releases/$VERSION/downloads/bin/apple-macosx" --output $tmp_archive

mkdir -p $dest_dir

tar -xf $tmp_archive -C $dest_dir ./nxc

rm tmp_archive
