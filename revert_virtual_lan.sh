#!/bin/bash

BASE_IP="10.10.0"
START=1
END=50

echo "Removing virtual LAN IP aliases from loopback (lo0)..."
sudo route -n delete -host 10.10.0.255 -interface lo0 2>/dev/null || true

for i in $(seq $START $END); do
    IP="$BASE_IP.$i"
    echo "Removing IP alias: $IP"
    sudo ifconfig lo0 -alias $IP
done

echo "Virtual LAN cleanup complete."