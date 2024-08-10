#!/usr/bin/env bash

CLI=./target/release/land-cli

# iterate tests/js-files/*.js
for file in tests/js-files/*.js; do
    echo "Building $file"
    $CLI build $file
done