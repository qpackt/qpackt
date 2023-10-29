#!/usr/bin/env bash

# Compiles backend server, waits for successful start.
function start_backend() {
  (
    cd vaden-backend || exit
    mkdir test_run_directory
    cargo r -r -- vaden.yaml &
    while :;
    do
      if curl localhost:8444/static/ > /dev/null 2>&1; then break ; fi
      sleep 1
    done
  )
}

function stop_backend() {
    (
      cd vaden-backend || exit
      killall vaden-backend
    )
}

################################
# MAIN
################################

start_backend
find tests -name "test_*" |
while read -r name;
do
  echo "Executing test in $name"
  (cd "$name" || exit; ./test.sh)
done
stop_backend
