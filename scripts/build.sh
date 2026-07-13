#!/usr/bin/env bash

cd ..
cargo build -p nt_client
mkdir -p addons/nodetunnel/bin
if [ -f target/debug/libnt_client.so ]; then
  cp -f target/debug/libnt_client.so addons/nodetunnel/bin/
fi
