@echo off
echo World server console. Leave it with Ctrl+X then X. NEVER press Ctrl+C.
echo If it says the container is not running, wait a few minutes and try again.
echo.
docker attach --detach-keys="ctrl-x,x" ac-worldserver
pause
