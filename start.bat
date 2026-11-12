@echo off
cd /d "%~dp0"
set MSYS_NO_PATHCONV=1
set "GITBASH=C:\Program Files\Git\bin\bash.exe"

echo ==== WoW RA Server launcher ====
echo.

if not exist "%GITBASH%" goto nogit
where docker >nul 2>nul
if errorlevel 1 goto nodocker
docker info >nul 2>nul
if errorlevel 1 goto dockeroff

echo Building and starting the server. The first time takes a long while.
echo.
"%GITBASH%" scripts/build.sh
if errorlevel 1 goto failed

echo.
echo ==== Server is starting ====
echo On the first run, wait 10 to 15 more minutes before logging in.
echo First time only: run console.bat and type these two lines:
echo    account create YOURNAME YOURPASSWORD
echo    account set gmlevel YOURNAME 3 -1
echo Leave the console with Ctrl+X then X.
echo.
echo Then set your game's Data\enUS\realmlist.wtf to:  set realmlist 127.0.0.1
echo and start the game.
pause
exit /b 0

:nogit
echo Git for Windows is not installed. Install it, then run this again.
pause
exit /b 1

:nodocker
echo Docker Desktop is not installed. Install it, then run this again.
pause
exit /b 1

:dockeroff
echo Docker Desktop is not running. Open it, wait until it says running, then run this again.
pause
exit /b 1

:failed
echo The build failed. Copy the last lines above and show them to Claude.
pause
exit /b 1
