#!/bin/bash

# Check if exactly three arguments are given
if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <LIR file> <JSON file> <function name>"
    exit 1
fi

# Extract arguments
LIR_FILE="$1"
JSON_FILE="$2" # This script ignores this argument based on your requirements
FUNC_NAME="$3"

# Run the constants analysis
cargo run --bin intervals "$JSON_FILE" "$FUNC_NAME"
