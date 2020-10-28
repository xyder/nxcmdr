#!/usr/bin/env bash

# for this to work, make sure you have this in your ~/.pypirc file
#
# [distutils]
# index-servers =
#     gitlab_nxcmdr

# [gitlab_nxcmdr]
# repository = https://gitlab.com/api/v4/projects/22063064/packages/pypi
# username = YOUR_USER
# password = YOUR_TOKEN

poetry run python -m twine upload --repository gitlab_nxcmdr ./nx_core/target/wheels/*
poetry run python -m twine upload --repository gitlab_nxcmdr ./dist/*
