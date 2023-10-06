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
curl -v -H 'Content-type: multipart/form-data' -F "data=@data1.bin" -F "data=@data2.bin" localhost:8080/upload
sleep 1
killall -9 test-http

compare_sha data1.bin received_data1.bin
compare_sha data2.bin received_data2.bin

rm -v received_*


