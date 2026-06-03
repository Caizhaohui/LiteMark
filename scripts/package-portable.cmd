@echo off
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0package-portable.ps1" %*
exit /b %errorlevel%
