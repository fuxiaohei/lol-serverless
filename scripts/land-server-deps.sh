#!/usr/bin/env bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Print in colors - 1=green, 2=red, other=neutral
# e.g. log_print 0 "All is great"
log_print() {
    if [[ $1 == 1 ]]; then
        echo -e "${GREEN}${2}${NC}"
        elif [[ $1 == 2 ]]; then
        echo -e "${RED}${2}${NC}"
    else
        echo -e "${2}"
    fi
}

set -e
set -o pipefail

# Function used to check if utilities are available
require() {
    if ! hash "$1" &>/dev/null; then
        log_print 2 "'$1' not found in PATH. This is required for this script to work."
        exit 1
    fi
}

require curl
require tar

download_wizer_binary() {
    log_print 0 "Downloading wizer"
    local archive_url="https://github.com/bytecodealliance/wizer/releases/download/v7.0.5/wizer-v7.0.5-x86_64-linux.tar.xz"
    log_print 1 "Downloading wizer: $archive_url"
    curl --progress-bar --show-error --location --fail $archive_url --output "wizer-v7.0.5.tar.xz"
    tar -xvf "wizer-v7.0.5.tar.xz"
}

download_wizer_binary