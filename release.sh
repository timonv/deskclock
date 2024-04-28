#!/usr/bin/env bash

set -xe

cross build --target=aarch64-unknown-linux-gnu --release
scp target/aarch64-unknown-linux-gnu/release/deskclock timonv@192.168.1.200:/home/timonv/deskclock
