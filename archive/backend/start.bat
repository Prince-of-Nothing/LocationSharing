@echo off
:: Pinpoint — Start the backend + web client (Windows)
::
:: Usage: start.bat [port]
::   PORT defaults to 8000
::
:: Prerequisites:
#   - Go 1.23+ installed and on PATH
::
setlocal enabledelayedexpansion

set "PORT=%PORT:~1%"
if "%PORT%"=="" set "PORT=8000"

set DB_DRIVER=memory
set SEED_DEMO_DATA=true
set WEB_ROOT=..\web\public
set COOKIE_INSECURE=true

echo ==============================================
echo   Pinpoint — Starting Backend + Web Client
echo ==============================================
echo.
echo   Backend:  http://localhost:%PORT%/
echo   Web App:  http://localhost:%PORT%/
echo   API:      http://localhost:%PORT%/api/v1/
echo.
echo   Demo accounts:
echo     alice@example.com  / DemoPass123!
echo     bob@example.com    / DemoPass456!
echo     admin@example.com  / ChangeMe123! (must change password)
echo.
echo   Press Ctrl+C to stop
echo ==============================================
echo.

go run ./cmd/server