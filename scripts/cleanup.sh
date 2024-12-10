#!/bin/bash

# Check for unused dependencies
unused_deps=$(cargo machete | grep -oP '(?<=-- ./Cargo.toml:)[^\n]*' | tr '\n' ' ')

# If there are unused dependencies, remove them
if [ -n "$unused_deps" ]; then
    echo "Removing unused dependencies: $unused_deps"
    for dep in $unused_deps; do
        # Remove the dependency from Cargo.toml
        sed -i '' "/$dep/d" Cargo.toml
    done
    echo "Unused dependencies removed. Run 'cargo build' again."
else
    echo "No unused dependencies found."
fi
