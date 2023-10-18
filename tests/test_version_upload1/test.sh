#!/usr/bin/env bash
set -e
curl -v -H 'Content-type: multipart/form-data' -F "data=@htmls.zip" localhost:8444/upload-version
