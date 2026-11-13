#!/bin/bash

server_bin="./target/release/server" # from cargo build --release --bin server

nohup $server_bin &
