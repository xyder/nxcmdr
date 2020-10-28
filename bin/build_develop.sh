#!/usr/bin/env bash

cd ./nx_core
poetry install
poetry run maturin develop
