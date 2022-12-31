#add the following function to /home/user/.bashrc

lsi() {
  local output
  if output=$(ls-interactive "$@") && [[ $output ]] ; then
    cd "$output"
  fi
}
