# add the following function to your Microsoft.PowerShell_profile.ps1
# you can open your profile using one of the following commands:

# notepad $profile
# gedit $profile

function lsi {
    $output = (ls-interactive -x "$args")
    if (!$output) { return }
    if (Test-Path -Path $output -PathType leaf) { micro $output }
    else { Set-Location $output }
}
