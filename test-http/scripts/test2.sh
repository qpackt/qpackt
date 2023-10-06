#!/usr/bin/env bash
function compare_sha() {
    sha256_file1=$(sha256sum "$1" | awk '{print $1}')
    sha256_file2=$(sha256sum "$2" | awk '{print $1}')

    if [ "$sha256_file1" != "$sha256_file2" ]; then
        echo "SHA-256 sums do not match: $1 and $2"
        exit 1
    fi
}

set -e

(cd .. && cargo build -r)
../../target/release/test-http &

curl -o received_data1.bin localhost:8080/download
killall -9 test-http > /dev/null
compare_sha data1.bin received_data1.bin
rm received_data1.bin
