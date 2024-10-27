#!/bin/bash

SRC_DIR="/root/yggdrasil"
TARGET_DIR="/var/www/killarchive.fun/yggdrasil/target/"

cd $SRC_DIR

echo Building for linux ...
cargo build --release

echo \nDone ... Moving to target dir ...
cp target/release/yggdrasil $TARGET_DIR

echo Done ... Building for Windows ...
cargo build --release --target x86_64-pc-windows-gnu

echo \nDone ... Moving to target dir ...
cp target/x86_64-pc-windows-gnu/release/yggdrasil.exe $TARGET_DIR

echo \nAll done
