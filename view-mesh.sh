#!/bin/bash
if [ -z "$1" ]; then
  echo "Usage: $0 <argument>, arguments is male/canon/suitcase/car."
  exit 1
fi

./target/release/examples/show_mesh $1
