#!/usr/bin/env bash

cargo build

./target/debug/qtrust &
pid=$!
trap "kill $pid" INT TERM
wait $pid

ifconfig utun1 


