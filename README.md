# Deskclock

Personal project to display a clock and the next appointment on a Raspberry Pi with a 7" touch screen.

I tend to be late, this gives me the illusion that I tried my best by throwing engineering at the problem.

Feel free to fork and experiment.

![image](https://github.com/timonv/deskclock/assets/49373/3f26a1d7-ef5f-46e2-81d2-eaa55a026904)

## Features

- Display the current time
- Display the next appointment from a Google Calendar
- Display appointments for today and later
- Next appointment is green if started, red if happening soon

## Deployment

Example script to deploy the project on a Raspberry Pi with a 7" touch screen.

For authentication, create a service account and share your calendar with the service account. It needs to have google calendar read access.

Credentials are expected to be in `./credentials.json` and a `GOOGLE_CALENDAR_ACCOUNT` set in a local dot env file.

```bash

#!/usr/bin/env bash

set -xe

cross build +nightly --target=aarch64-unknown-linux-gnu --release
scp target/aarch64-unknown-linux-gnu/release/deskclock $PI_URL:/home/pi/deskclock
ssh $PI_URL 'DISPLAY=:0 /home/pi/deskclock'
```
