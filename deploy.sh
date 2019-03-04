#!/bin/sh

cargo web deploy --package tetris_client --release
cp target/deploy/tetris_client.* static/