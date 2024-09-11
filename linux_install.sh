#!/bin/bash

bold="\033[1m"
normal="\033[0m"

echo "${bold}Installing dependencies:${normal}"

# Install clippers [https://docs.rs/clippers/latest/clippers/] linux dependencies:
# - libx11-dev/libX11-devel
# - libpng-dev/libpng-devel
# - cmake
sudo apt install xorg-dev cmake

# Install rustup
if ! which cargo > /dev/null; then
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
fi

# Build pw
echo "\n${bold}Building:${normal}"
cargo build --release

# Make it usable
echo "\n${bold}Installing:${normal}"
sudo cp ./target/release/pw /usr/bin/pw
echo "pw copied to /usr/bin/pw"

echo "\n${bold}Success!${normal}\nTry it out by running command 'pw'"
