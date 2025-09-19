#!/bin/bash

set -euo pipefail

# Get the current version from Cargo.toml
get_version() {
    grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'
}

# Update README.md with the current version
update_readme() {
    local current_version="$1"
    local readme_file="README.md"
    
    if [[ ! -f "$readme_file" ]]; then
        echo "Error: README.md not found"
        exit 1
    fi
    
    # Create a backup
    cp "$readme_file" "${readme_file}.backup"
    
    # Update .deb package URL
    sed -i "s|clifx-v[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*[^/]*-amd64\.deb|clifx-v${current_version}-amd64.deb|g" "$readme_file"
    
    # Update tarball URL
    sed -i "s|clifx-v[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*[^/]*-x86_64-unknown-linux-gnu\.tar\.gz|clifx-v${current_version}-x86_64-unknown-linux-gnu.tar.gz|g" "$readme_file"
    
    # Update extraction command for tarball
    sed -i "s|tar -xzf clifx-v[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*[^/]*-x86_64-unknown-linux-gnu\.tar\.gz|tar -xzf clifx-v${current_version}-x86_64-unknown-linux-gnu.tar.gz|g" "$readme_file"
    
    echo "Updated README.md with version ${current_version}"
    
    # Show the changes
    if command -v diff >/dev/null 2>&1; then
        echo "Changes made:"
        diff "${readme_file}.backup" "$readme_file" || true
    fi
}

main() {
    # Check if we're in the project root
    if [[ ! -f "Cargo.toml" ]]; then
        echo "Error: This script must be run from the project root directory"
        exit 1
    fi
    
    local version
    version=$(get_version)
    
    if [[ -z "$version" ]]; then
        echo "Error: Could not determine version from Cargo.toml"
        exit 1
    fi
    
    echo "Current version: $version"
    update_readme "$version"
}

main "$@"