#!/usr/bin/env fish

# Command for ls-interactive use.
# Copy this file into ~/.config/fish/functions/ OR copy this function into your config.fish

function lsi
    set -l output
    if output=(ls-interactive $argv -x); and [ -n "$output" ]
        if test -f "$output"
            micro "$output"
        else
            cd "$output"
        end
    end
end

