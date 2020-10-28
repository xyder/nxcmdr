#!/usr/bin/env bash

cd ./nx_core
poetry install
poetry run maturin build --manylinux 2014
