#!/usr/bin/env bash

set -xe

cross build +nightly --target=aarch64-unknown-linux-gnu --release
scp credentials.json timonv@192.168.1.200:/home/timonv/credentials.json
scp tokencache.json timonv@192.168.1.200:/home/timonv/tokencache.json
scp target/aarch64-unknown-linux-gnu/release/deskclock timonv@192.168.1.200:/home/timonv/deskclock
ssh timonv@192.168.1.200 'DISPLAY=:0 /home/timonv/deskclock'
