# add the following function to your Microsoft.PowerShell_profile.ps1
# you can open your profile using one of the following commands:

# notepad $profile
# gedit $profile

function lsi {
    $output = (ls-interactive "$args")
    if ($output) { Set-Location $output }
}
