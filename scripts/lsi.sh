#!/bin/bash

output=$("$(dirname "$0")/ls-interactive")
[ -n "$output" ] && cd "$(output)"
