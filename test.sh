#!/usr/bin/env bash

cargo build

sudo ./target/debug/qtrust&
pid=$!


sleep 3s

sudo ifconfig utun4 inet 10.0.1.1 10.0.1.2 up netmask 255.255.255.0
#sudo ifconfig utun3 inet 10.0.4.1 10.0.4.1 up netmask 255.255.255.0
#sudo route add -net 10.0.4.0 10.0.4.1
sudo sysctl net.inet.ip.forwarding=0

echo "tun is up"

trap "sudo kill $pid" INT TERM
wait $pid
