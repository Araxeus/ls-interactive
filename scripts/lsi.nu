# add the following function to env.nu
# you can open env.nu in nushell with `config env`

def --env lsi [...path: string] {
    let output = (ls-interactive ($path | str join ' '))
    if ($output | is-empty) { return }
    if (($output | path type) == 'file') { 
        start $output # replace with your preferred editor
    } else { cd $output }
}
