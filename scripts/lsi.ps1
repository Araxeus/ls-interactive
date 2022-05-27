# paste the following function inside your Microsoft.PowerShell_profile.ps1
# you can open it using the following command:

# notepad $profile

# which will open either:

# New Powershell:
# C:\Users\MyUser\Documents\PowerShell\Microsoft.PowerShell_profile.ps1

# Old Powershell
# C:\Users\MyUser\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1


function lsi {
    $output = (ls-interactive "$args")
    if ($output) {
        cd $output
    }
}
