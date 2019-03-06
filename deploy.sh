#!/bin/sh

rm -rf deploy
cargo web deploy --package tetris_client --release
cargo build --package tetris_server --release
mkdir deploy
cp -r static deploy/
cp target/release/tetris_server deploy/server
cp target/release/tetris_server.exe deploy/server.exe
cp target/deploy/tetris_client.* deploy/static/