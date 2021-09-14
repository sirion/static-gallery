#!/bin/bash
RUST_BACKTRACE=1 cargo run -- --output data/test/output/ --template data/templates/hauer/ --collection "data/test/input01/;data/test/backgrounds01/;Collection 01" -vvv "$@"