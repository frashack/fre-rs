set NAME_CLS=NativeExtension
set NAME_LIB=native_extension
set NAME_ANE=NativeExtension


chcp 65001
cd /d %~dp0


REM ┣━━━━━━━━━━━━━━━━ Build AIR Native Extension ━━━━━━━━━━━━━━━━┫
call cargo build --target x86_64-pc-windows-msvc
if %ERRORLEVEL% neq 0 exit /b %ERRORLEVEL%

mkdir temp

call "%AIR_HOME%\bin\acompc" -source-path .\ -include-classes src.%NAME_CLS% -output temp\lib.swc
if %ERRORLEVEL% neq 0 exit /b %ERRORLEVEL%

tar -xf temp\lib.swc -C temp library.swf
copy target\x86_64-pc-windows-msvc\debug\%NAME_LIB%.dll temp\lib.dll
mkdir extensions

call "%AIR_HOME%\bin\adt" -package -target ane extensions\%NAME_ANE%.ane extension.xml -swc temp\lib.swc -platform Windows-x86-64 -C temp library.swf lib.dll
if %ERRORLEVEL% neq 0 exit /b %ERRORLEVEL%

REM ┣━━━━━━━━━━━━━━━━ Build the application ━━━━━━━━━━━━━━━━┫
call "%AIR_HOME%\bin\amxmlc" ^
	-source-path=%~dp0 ^
	-external-library-path+="extensions\%NAME_ANE%.ane" ^
	-output=application.swf ^
	-verbose-stacktraces=true ^
	-debug=true ^
	src\Main.as
if %ERRORLEVEL% neq 0 exit /b %ERRORLEVEL%

REM ┣━━━━━━━━━━━━━━━━ Run from the AIR Debug Launcher ━━━━━━━━━━━━━━━━┫
mkdir temp\%NAME_ANE%.ane
tar -xf extensions\%NAME_ANE%.ane -C temp\%NAME_ANE%.ane

"%AIR_HOME%\bin\adl64" -profile extendedDesktop application.xml -extdir temp

rmdir /s /q temp

pause

