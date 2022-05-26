#!/usr/bin/env fish

# Command for ls-interactive use.
# Copy this file into ~/.config/fish/functions/
# to have it accessible at all times.

function lsi
    set -l output (ls-interactive $argv)
    if test $status -eq 0
        and test -n "$output"
        cd $output
    end
end

