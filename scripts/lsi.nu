# add the following function to env.nu
# you can open it via the following command in nushell:
# notepad $nu.env-path

def-env lsi [...path: string] {
    let output = (ls-interactive ($path | str collect ' '))
    cd (
        if ($output | empty?) { $env.PWD } 
        else { $output }
    )
}
