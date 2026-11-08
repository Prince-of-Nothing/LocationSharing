@echo off

set VSCMD_ARG_HOST_ARCH=x64
set VSCMD_ARG_TGT_ARCH=x64

set VCToolsVersion=14.51.36231
set WindowsSDKVersion=10.0.28000.0\

set VCToolsInstallDir=%~dp0VC\Tools\MSVC\14.51.36231\
set WindowsSdkBinPath=%~dp0Windows Kits\10\bin\

set PATH=%~dp0VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64;%~dp0Windows Kits\10\bin\10.0.28000.0\x64;%~dp0Windows Kits\10\bin\10.0.28000.0\x64\ucrt;%PATH%
set INCLUDE=%~dp0VC\Tools\MSVC\14.51.36231\include;%~dp0Windows Kits\10\Include\10.0.28000.0\ucrt;%~dp0Windows Kits\10\Include\10.0.28000.0\shared;%~dp0Windows Kits\10\Include\10.0.28000.0\um;%~dp0Windows Kits\10\Include\10.0.28000.0\winrt;%~dp0Windows Kits\10\Include\10.0.28000.0\cppwinrt
set LIB=%~dp0VC\Tools\MSVC\14.51.36231\lib\x64;%~dp0Windows Kits\10\Lib\10.0.28000.0\ucrt\x64;%~dp0Windows Kits\10\Lib\10.0.28000.0\um\x64
