#!/bin/bash

# Define your custom subnet
BASE_IP="10.10.0"
START=1
END=50  # Number of IPs you want to assign (10.10.0.1 to 10.10.0.50)
NETMASK="255.255.255.0"

echo "Setting up virtual LAN on loopback (lo0) using $BASE_IP.0/24..."
sudo route -n add -host 10.10.0.255 -interface lo0 2>/dev/null || true

# Loop to add IP aliases to loopback
for i in $(seq $START $END); do
    IP="$BASE_IP.$i"
    echo "Adding IP alias: $IP"
    sudo ifconfig lo0 alias $IP $NETMASK
done

echo "Virtual LAN setup complete."