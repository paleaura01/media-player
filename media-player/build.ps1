# build.ps1
$ErrorActionPreference = "Stop"

Set-Location $PSScriptRoot
if (Test-Path build) { Remove-Item -Recurse -Force build }
New-Item -ItemType Directory build
Set-Location build

cmake .. -DCMAKE_TOOLCHAIN_FILE="../vcpkg/scripts/buildsystems/vcpkg.cmake" -A x64
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cmake --build . --config Release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
