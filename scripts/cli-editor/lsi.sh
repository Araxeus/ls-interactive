# Add the following function to /home/user/.bashrc or /home/user/.zshrc

lsi() {
    local output
    if output=$(ls-interactive "$@" -x) && [[ $output ]]; then
        if [ -f "$output" ]; then
            micro "$output"
        else
            cd "$output"
        fi
    fi
}
