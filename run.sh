#!/bin/bash

if [ -z "$1" ]; then
    echo "Error: Program name argument is missing."
    exit 1
fi

inputs() {
    case "$1" in
        address-lookup-table)
            PROGRAM_ID="AddressLookupTab1e1111111111111111111111111"
            FEATURE_ID="C97eKZygrkU4JxJsZdjgbUY7iQR7rKTr4NyDWo2E5pRm"
            BUFFER_ADDRESS="AhXWrD9BBUYcKjtpA3zuiiZG4ysbo6C6wjHo1QhERk6A"
            ;;
        config)
            PROGRAM_ID="Config1111111111111111111111111111111111111"
            FEATURE_ID="2Fr57nzzkLYXW695UdDxDeR5fhnZWSttZeZYemrnpGFV"
            BUFFER_ADDRESS="BuafH9fBv62u6XjzrzS4ZjAE8963ejqF5rt1f8Uga4Q3"
            ;;
        feature-gate)
            PROGRAM_ID="Feature111111111111111111111111111111111111"
            FEATURE_ID="4eohviozzEeivk1y9UbrnekbAFMDQyJz5JjA9Y6gyvky"
            BUFFER_ADDRESS="3D3ydPWvmEszrSjrickCtnyRSJm1rzbbSsZog8Ub6vLh"
            ;;
        *)
            echo "Invalid argument. Use 'address-lookup-table', 'config', or 'feature-gate'."
            exit 1
            ;;
    esac
}

inputs "$1"

cargo run --release --bin cbm -- stub-test $PROGRAM_ID $FEATURE_ID $BUFFER_ADDRESS
