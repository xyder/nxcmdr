#!/usr/bin/env bash

rm -fr releases && mkdir releases

./bin/create_linux_release_archive.sh
./bin/create_mac_release_archive.sh
