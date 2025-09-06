#!/usr/bin/env bash

echo "Running graz-opnv-bot at $(date)"

# if the messae is empty nothing will be sent
matrix-commander-rs -m "$(target/release/graz-opnv-bot)"
