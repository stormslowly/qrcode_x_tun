#!/usr/bin/env bash

cargo build

sudo ./target/debug/qtrust&
pid=$!


sleep 3s

sudo ifconfig utun2 inet 10.0.0.1 10.0.0.2 up

echo "tun is up"

trap "sudo kill $pid" INT TERM
wait $pid
