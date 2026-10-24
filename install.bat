@echo off
rem Lanza install.ps1 con doble clic. Argumentos opcionales: install.bat -Build
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1" %*
echo.
pause
