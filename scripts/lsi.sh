# Add the following function to /home/user/.bashrc or /home/user/.zshrc

lsi() {
  local output
  if output=$(ls-interactive "$@") && [[ $output ]] ; then
    cd "$output"
  fi
}
