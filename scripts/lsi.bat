:: Place both this script and ls-interactive.exe in one of the folders in your %PATH%
:: You can open your environment variables by running "rundll32.exe sysdm.cpl,EditEnvironmentVariables"

@echo off
for /f "tokens=*" %%i in ('%~dp0\ls-interactive.exe "%*"') do set output=%%i
IF DEFINED output cd %output%
