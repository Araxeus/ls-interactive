# add the following function to env.nu
# you can edit your config using one of the following commands:

# gedit $nu.env-path
# notepad $nu.env-path

def-env lsi [...path: string] {
    let output = (ls-interactive ($path | str collect ' '))
    cd (
        if ($output | is-empty) { $env.PWD } 
        else { $output }
    )
}
