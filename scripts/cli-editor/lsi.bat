:: Place both this script and ls-interactive.exe in one of the folders in your %PATH%
:: You can open your environment variables by running "rundll32.exe sysdm.cpl,EditEnvironmentVariables"

@echo off
set "output="
for /f "tokens=*" %%i in ('%~dp0\ls-interactive.exe -x "%*"') do set "output=%%i"
IF DEFINED output CALL :exec "%output%"
GOTO :EOF

:exec
    SET "file_attribute=%~a1"
    if "%file_attribute:~0,1%"=="d" (
        cd /d %1
    ) else (
        micro %1
        call "%~f0" "%~dp1."
    )
    GOTO :EOF
