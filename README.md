# Deskclock

Personal project to display a clock and the next appointment on a Raspberry Pi with a 7" touch screen.

I tend to be late, this gives me the illusion that I tried my best by throwing engineering at the problem.

Feel free to fork and experiment.

## Features

- Display the current time
- Display the next appointment from a Google Calendar
- Display appointments for today and later
- Next appointment is green if started, red if happening soon

## Deployment

Example script to deploy the project on a Raspberry Pi with a 7" touch screen.

Make sure you have set up a Google Cloud project with the Calendar API enabled and downloaded the credentials. Personally I prefer to auth locally and just copy over the token cache.

```bash

#!/usr/bin/env bash

set -xe

cross build +nightly --target=aarch64-unknown-linux-gnu --release
scp credentials.json $PI_URL:/home/pi/credentials.json
scp tokencache.json $PI_URL:/home/pi/tokencache.json
scp target/aarch64-unknown-linux-gnu/release/deskclock $PI_URL:/home/pi/deskclock
ssh $PI_URL 'DISPLAY=:0 /home/pi/deskclock'
```
