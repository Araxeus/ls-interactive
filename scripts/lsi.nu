# add the following function to env.nu
# you can open env.nu in nushell with `config env`

def-env lsi [...path: string] {
    let output = (ls-interactive ($path | str collect ' '))
    cd (
        if ($output | is-empty) { $env.PWD } 
        else { $output }
    )
}
