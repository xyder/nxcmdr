#!/usr/bin/env python

""" Builds the rust crate as a python package """

from plumbum import local, FG


def build_for_develop():
    maturin = local['maturin']
    with local.cwd(local.cwd / './nx_core/crates/nx_core_interface'):
        maturin['develop'] & FG


build_for_develop()
