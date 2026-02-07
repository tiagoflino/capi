# scripts/build_windows.ps1

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path $ScriptDir -Parent
$LibsDir = Join-Path $ProjectRoot "libs"
$OpenVinoDir = Join-Path $LibsDir "openvino"

# Check if OpenVINO is installed
if (-not (Test-Path $OpenVinoDir)) {
    Write-Host "OpenVINO not found. Running setup..."
    & (Join-Path $ScriptDir "setup_dev.ps1")
}

# Set environment variables for build
$env:OPENVINO_ROOT = $OpenVinoDir
$env:Path += ";$env:OPENVINO_ROOT\runtime\bin\intel64"
$env:Path += ";$env:OPENVINO_ROOT\runtime\3rdparty\tbb\bin"

Write-Host "Environment configured. Building Rust backend..."

# Build Tauri App (this will also build the Rust backend)
Set-Location -Path $ProjectRoot
Write-Host "Building Tauri App..."
& npm run tauri build

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed!"
    exit 1
}

# Bundle for release
$ReleaseDir = Join-Path $ProjectRoot "target\release"
$BundleDir = Join-Path $ProjectRoot "target\release\bundle\msi" # Default Tauri MSI output, check if needed
$ZipName = "capi-windows-x64.zip"
$ZipPath = Join-Path $ProjectRoot $ZipName

Write-Host "Creating release bundle: $ZipName"

# Create a temporary directory for bundling
$TempBundle = Join-Path $ProjectRoot "temp_bundle"
if (Test-Path $TempBundle) { Remove-Item -Path $TempBundle -Recurse -Force }
New-Item -ItemType Directory -Path $TempBundle | Out-Null

# Copy Capi Executable
Copy-Item -Path (Join-Path $ReleaseDir "capi-ui.exe") -Destination (Join-Path $TempBundle "capi.exe")

# Copy OpenVINO DLLs
$OpenVinoBin = Join-Path $env:OPENVINO_ROOT "runtime\bin\intel64"
$TbbBin = Join-Path $env:OPENVINO_ROOT "runtime\3rdparty\tbb\bin"

Write-Host "Copying OpenVINO DLLs..."
Get-ChildItem -Path $OpenVinoBin -Filter "*.dll" | Copy-Item -Destination $TempBundle
Get-ChildItem -Path $TbbBin -Filter "*.dll" | Copy-Item -Destination $TempBundle
# Also copy plugins.xml if it exists in bin
if (Test-Path (Join-Path $OpenVinoBin "plugins.xml")) {
    Copy-Item -Path (Join-Path $OpenVinoBin "plugins.xml") -Destination $TempBundle
}


Write-Host "Zipping bundle..."
Compress-Archive -Path "$TempBundle\*" -DestinationPath $ZipPath -Force

# Cleanup
Remove-Item -Path $TempBundle -Recurse -Force

Write-Host "Build complete! Bundle created at $ZipPath"
