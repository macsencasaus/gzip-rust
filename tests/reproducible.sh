#!/bin/bash

. ./tests/init.sh

TEMP_FILES="t1 t2"

echo a | $GZIP_RUST -n > t1 || fail "gzip-rust command failed"
sleep 1
echo a | $GZIP_RUST -n > t2 || fail "gzip-rust command failed"

if ! cmp t1 t2 >/dev/null 2>&1; then
    fail "Output does not match expected"
fi

exit 0
