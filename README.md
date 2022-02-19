# Introduction

Friendle is a Discord bot that allows you to play wordle with friends 
by encoding words and sharing the code with friends so they can try to solve for the encoded word.

# Cross compilation and deployment for Raspberry Pi 

These instructions are for building Friendle on a somewhat modern Raspberry Pi (2/3/4).
Note that these instructions will not work for other devices, like a Raspberry Pi Zero.

## Rust target

You need to install the rustup target (armv7-unknown-linux-gnueabihf in our case):  
 `rustup target add armv7-unknown-linux-gnueabihf`

## C depdendencies

There are C dependencies involved in the build process, so you need to have a suitable compiler and linker installed.

On Ubuntu, this can be done e.g. via `sudo apt install gcc-arm-linux-gnueabihf`

## Target env variable
We need to point the linker to the right place. If you followed the previous instructions, this means  
`export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=/usr/bin/arm-linux-gnueabihf-gcc`

## build command
We need to build for the correct target (the same target that we added earlier):  
`cargo build --target armv7-unknown-linux-gnueabihf --release`.

## Deploying
After building, you only need to copy the resulting executable (and the word list) to the Raspberry Pi
and export the necessary environment variables in order to run Friendle on the Raspberry Pi sucessfully.

For a more robust deployment, you can add a systemd service. A sample skeleton is provided in systemd/system/friendle@.service. Don't forget to replace the placeholders with the information for your bot.


### systemd setup
- Move the adjusted service file to `/etc/systemd/system/friendle@.service`.
- Enable the service to start automatically upon startup (the sample systemd file also configures an
automatic restart in case the bot crashes): 
`sudo systemctl enable friendle@one`
(friendle@one is the name of the service instance here; if you pick a different name, replace the name in the following instructions accordingly).
- The previous step will not automatically start the service. In order to start it, reboot or 
start it manually via  
`sudo systemctl start friendle@one` 

### Operations
- You can check on the status of the service `sudo systemctl status friendle@one`
- You can check the service logs via `journalctl -u friendle@one`
- In order to upgrade the service to a newer version, build the new version, move it to the Raspberry Pi,
then stop the service `sudo systemctl stop friendle@one`, overwrite the old binary with the new one and restart the service.
- If you manually changed the systemd file, you will need to reload the systemd configuration:
`sudo systemctl daemon-reload`
