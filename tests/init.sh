#!/bin/bash

# Set up the test environment
GZIP_RUST="./target/debug/gzip-rust"

# Cleanup function
cleanup() {
    rm -f $TEMP_FILES
}

trap cleanup EXIT

fail() {
    printf "Test failed: $1"
    exit 1
}
