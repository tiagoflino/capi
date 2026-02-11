# scripts/build_windows.ps1

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path $ScriptDir -Parent
$LibsDir = Join-Path $ProjectRoot "libs"
$OpenVinoDir = Join-Path $LibsDir "openvino"
$SidecarBinDir = Join-Path $ProjectRoot "capi-ui\src-tauri\bin"

# --- OpenVINO Setup ---
$OvVersion = "2025.4.1.0"
$OvShortVer = "2025.4.1"
$OpenVinoUrl = "https://storage.openvinotoolkit.org/repositories/openvino_genai/packages/$OvShortVer/windows/openvino_genai_windows_${OvVersion}_x86_64.zip"

if (-not (Test-Path $OpenVinoDir)) {
    Write-Host "Downloading OpenVINO $OvVersion..."
    New-Item -ItemType Directory -Path $LibsDir -Force | Out-Null
    $ZipPath = Join-Path $LibsDir "openvino_genai.zip"
    Invoke-WebRequest -Uri $OpenVinoUrl -OutFile $ZipPath

    Write-Host "Extracting OpenVINO..."
    Expand-Archive -Path $ZipPath -DestinationPath $LibsDir

    $ExtractedFolder = Get-ChildItem -Path $LibsDir -Directory | Where-Object { $_.Name -like "openvino_genai_windows*" } | Select-Object -First 1
    if ($ExtractedFolder) {
        Rename-Item -Path $ExtractedFolder.FullName -NewName "openvino"
    }
    Remove-Item $ZipPath
    Write-Host "OpenVINO installed to $OpenVinoDir"
} else {
    Write-Host "OpenVINO already installed at $OpenVinoDir"
}

# Set environment variables for build
$env:OPENVINO_ROOT = $OpenVinoDir
$env:Path += ";$env:OPENVINO_ROOT\runtime\bin\intel64"
$env:Path += ";$env:OPENVINO_ROOT\runtime\3rdparty\tbb\bin"

Write-Host "Environment configured."

# --- Build capi-server sidecar ---
Write-Host "Building capi-server sidecar..."
Set-Location -Path $ProjectRoot
& cargo build --release --bin capi-server

if ($LASTEXITCODE -ne 0) {
    Write-Error "capi-server build failed!"
    exit 1
}

# Copy sidecar to where Tauri expects it
New-Item -ItemType Directory -Path $SidecarBinDir -Force | Out-Null
Copy-Item -Path (Join-Path $ProjectRoot "target\release\capi-server.exe") -Destination (Join-Path $SidecarBinDir "capi-server-x86_64-pc-windows-msvc.exe")
Write-Host "Sidecar prepared at $SidecarBinDir"

# --- Build Tauri App ---
Write-Host "Building Tauri App..."
Set-Location -Path (Join-Path $ProjectRoot "capi-ui")
& npm ci
& npm run tauri build

if ($LASTEXITCODE -ne 0) {
    Write-Error "Tauri build failed!"
    exit 1
}

Set-Location -Path $ProjectRoot

# --- Create portable zip ---
$ReleaseDir = Join-Path $ProjectRoot "target\release"
$ZipName = "capi-windows-x64.zip"
$ZipPath = Join-Path $ProjectRoot $ZipName

Write-Host "Creating release bundle: $ZipName"

$TempBundle = Join-Path $ProjectRoot "temp_bundle"
if (Test-Path $TempBundle) { Remove-Item -Path $TempBundle -Recurse -Force }
New-Item -ItemType Directory -Path $TempBundle | Out-Null

# Copy Capi UI Executable
Copy-Item -Path (Join-Path $ReleaseDir "capi-ui.exe") -Destination (Join-Path $TempBundle "capi.exe")

# Copy capi-server
Copy-Item -Path (Join-Path $ReleaseDir "capi-server.exe") -Destination $TempBundle

# Copy OpenVINO DLLs
$OpenVinoBin = Join-Path $env:OPENVINO_ROOT "runtime\bin\intel64"
$TbbBin = Join-Path $env:OPENVINO_ROOT "runtime\3rdparty\tbb\bin"

Write-Host "Copying OpenVINO DLLs..."
Get-ChildItem -Path $OpenVinoBin -Filter "*.dll" | Copy-Item -Destination $TempBundle
Get-ChildItem -Path $TbbBin -Filter "*.dll" | Copy-Item -Destination $TempBundle
if (Test-Path (Join-Path $OpenVinoBin "plugins.xml")) {
    Copy-Item -Path (Join-Path $OpenVinoBin "plugins.xml") -Destination $TempBundle
}

Write-Host "Zipping bundle..."
Compress-Archive -Path "$TempBundle\*" -DestinationPath $ZipPath -Force

# Cleanup
Remove-Item -Path $TempBundle -Recurse -Force

Write-Host "Build complete! Bundle created at $ZipPath"
