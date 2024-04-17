#!/bin/bash

supported_programs=("address-lookup-table" "config" "feature-gate")

if [ -z "$1" ]; then
    echo "Error: Program name argument is missing."
    exit 1
fi

if [[ ! " ${supported_programs[*]} " =~ " $1 " ]]; then
    echo "Error: Invalid program name '$1'. Allowed values are: ${supported_programs[*]}."
    exit 2
fi

mkdir -p elfs
mkdir -p programs

if [ -d "programs/$1" ]; then
    echo "Updating program $1...";
    (cd programs/$1 && git fetch && git pull);
else
    echo "Cloning program $1...";
    git clone https://github.com/solana-program/$1 programs/$1;
fi

cargo build-sbf --manifest-path=programs/$1/program/Cargo.toml --sbf-out-dir elfs --features bpf-entrypoint