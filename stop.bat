@echo off
cd /d "%~dp0"
set MSYS_NO_PATHCONV=1
"C:\Program Files\Git\bin\bash.exe" scripts/dc.sh stop
pause
