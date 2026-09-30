@echo off
setlocal enabledelayedexpansion

echo ================================================================================
echo                     BUILDING LOGIC LAB ANDROID APK
echo ================================================================================

set "JAVA_HOME=D:\DLCD\tools\jdk17\jdk-17.0.20.1+1"
set "PATH=%JAVA_HOME%\bin;%PATH%"
set "ANDROID_SDK=C:\Users\tusha\AppData\Local\Android\Sdk"
set "ANDROID_JAR=%ANDROID_SDK%\platforms\android-36\android.jar"
set "BUILD_TOOLS=%ANDROID_SDK%\build-tools\34.0.0"

set "AAPT2=%BUILD_TOOLS%\aapt2.exe"
set "D8=%BUILD_TOOLS%\d8.bat"
set "ZIPALIGN=%BUILD_TOOLS%\zipalign.exe"
set "APKSIGNER=%BUILD_TOOLS%\apksigner.bat"

if not exist "%ANDROID_JAR%" (
    echo Error: android.jar not found at %ANDROID_JAR%
    exit /b 1
)

:: Prepare build folders
if exist build rmdir /s /q build
mkdir build
mkdir build\gen
mkdir build\obj
mkdir build\dex

if not exist "..\dist" mkdir "..\dist"

echo [1/7] Compiling Android resources with AAPT2...
"%AAPT2%" compile --dir res -o build\compiled_res.zip
if %errorlevel% neq 0 (
    echo Error: AAPT2 compile failed.
    exit /b %errorlevel%
)

echo [2/7] Linking resources and AndroidManifest...
"%AAPT2%" link -I "%ANDROID_JAR%" --manifest AndroidManifest.xml build\compiled_res.zip -o build\app-unaligned.apk --java build\gen
if %errorlevel% neq 0 (
    echo Error: AAPT2 link failed.
    exit /b %errorlevel%
)

echo [3/7] Compiling Java source files...
set "EXTRA_JAVA="
if exist build\gen\com\logiclab\app\R.java set "EXTRA_JAVA=build\gen\com\logiclab\app\R.java"

javac -cp "%ANDROID_JAR%" -d build\obj src\com\logiclab\app\MainActivity.java !EXTRA_JAVA!
if %errorlevel% neq 0 (
    echo Error: javac compilation failed.
    exit /b %errorlevel%
)

echo [4/7] Generating classes.dex with D8...
call "%D8%" --lib "%ANDROID_JAR%" --output build\dex build\obj\com\logiclab\app\*.class
if %errorlevel% neq 0 (
    echo Error: D8 dex generation failed.
    exit /b %errorlevel%
)

echo [5/7] Adding classes.dex and assets to APK...
cd build\dex
"%JAVA_HOME%\bin\jar.exe" -uf "..\app-unaligned.apk" classes.dex
if %errorlevel% neq 0 (
    echo Error: jar failed to add classes.dex.
    cd ..\..
    exit /b %errorlevel%
)
cd ..\..
"%JAVA_HOME%\bin\jar.exe" -uf "build\app-unaligned.apk" assets
if %errorlevel% neq 0 (
    echo Error: jar failed to add assets.
    exit /b %errorlevel%
)

echo [6/7] Running zipalign on APK...
"%ZIPALIGN%" -f -p 4 build\app-unaligned.apk build\app-aligned.apk
if %errorlevel% neq 0 (
    echo Error: zipalign failed.
    exit /b %errorlevel%
)

echo [7/7] Generating signing keystore and signing APK...
if not exist "build\debug.keystore" (
    "%JAVA_HOME%\bin\keytool.exe" -genkeypair -v -keystore build\debug.keystore -storepass android -alias androiddebugkey -keypass android -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=Logic Lab,OU=Engineering,O=LogicLab,C=US"
)

call "%APKSIGNER%" sign --ks build\debug.keystore --ks-pass pass:android --ks-key-alias androiddebugkey --key-pass pass:android --out "..\dist\LogicLab.apk" build\app-aligned.apk
if %errorlevel% neq 0 (
    echo Error: apksigner failed.
    exit /b %errorlevel%
)

echo.
echo Verifying APK signature...
call "%APKSIGNER%" verify "..\dist\LogicLab.apk"
if %errorlevel% neq 0 (
    echo Warning: APK verification check returned non-zero.
)

echo ================================================================================
echo               BUILD SUCCESSFUL: LogicLab.apk is ready in dist\
echo ================================================================================
dir "..\dist\LogicLab.apk"
