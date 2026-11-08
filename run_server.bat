@echo off
rem builds and starts the beta173 server, extra arguments go to MinecraftServer.main
setlocal
cd /d "%~dp0"

if not defined B173_PROFILE set "B173_PROFILE=release"
cargo build --profile %B173_PROFILE% --bin beta173_server || exit /b 1
target\%B173_PROFILE%\beta173_server.exe %*
exit /b %errorlevel%
