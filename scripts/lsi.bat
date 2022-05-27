@echo off
for /f "tokens=*" %%i in ('%~dp0\ls-interactive.exe "%*"') do set output=%%i
IF DEFINED output cd %output%
