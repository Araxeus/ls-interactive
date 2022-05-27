@echo off
for /f "tokens=*" %%i in ('%~dp0\ls-interactive.exe "%*"') do set output=%%i
cd %output%
