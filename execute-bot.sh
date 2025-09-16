#!/usr/bin/env bash

echo "Running graz-opnv-bot at $(date)"

# if the messae is empty nothing will be sent
/home/thomas/.cargo/bin/matrix-commander-rs -m "$(target/release/graz-opnv-bot)" 2>&1

