#!/bin/bash
cargo build --release
cp target/release/jdisc ~/.local/bin/
chmod +x ~/.local/bin/jdisc
