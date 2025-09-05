#!/usr/bin/env bash

# Test all these features with "std" enabled.
#
# Ignore this if crate does not have "std" feature.
FEATURES_WITH_STD=""

# Test all these features without "std" enabled.
#
# Use this even if crate does not have "std" feature.
FEATURES_WITHOUT_STD=""

# Run these examples.
EXAMPLES="simple_sign_verify_encoded:default"
