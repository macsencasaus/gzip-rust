#!/bin/bash

# Ensure that gzip-rust interprets "-" as stdin.

. ./tests/init.sh

# Temporary files
TEMP_FILES="in out err exp"


# Create test input
printf a | $GZIP_RUST > in || fail "Failed to create test input"
printf aaa > exp || fail "Failed to create expected output"

# Run the test
$GZIP_RUST -dc in - in < in > out 2>err
if [ $? -ne 0 ]; then
    fail "gzip-rust command failed"
fi

# Compare outputs
if ! cmp exp out >/dev/null 2>&1; then
    fail "Output does not match expected"
fi

if [ -s err ]; then
    fail "Unexpected error output"
fi

exit 0
