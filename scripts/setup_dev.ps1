# scripts/setup_dev.ps1

$ErrorActionPreference = "Stop"

$OpenVinoUrl = "https://storage.openvinotoolkit.org/repositories/openvino_genai/packages/2024.5/windows/openvino_genai_windows_2024.5.0.0_x86_64.zip"
$LibsDir = Join-Path (Get-Location) "libs"
$OpenVinoDir = Join-Path $LibsDir "openvino"
$ZipPath = Join-Path $LibsDir "openvino_genai.zip"

# Create libs directory
if (-not (Test-Path $LibsDir)) {
    New-Item -ItemType Directory -Path $LibsDir | Out-Null
    Write-Host "Created $LibsDir"
}

# Download OpenVINO if not present
if (-not (Test-Path $OpenVinoDir)) {
    Write-Host "Downloading OpenVINO GenAI from $OpenVinoUrl..."
    Invoke-WebRequest -Uri $OpenVinoUrl -OutFile $ZipPath
    
    Write-Host "Extracting OpenVINO..."
    Expand-Archive -Path $ZipPath -DestinationPath $LibsDir
    
    # Rename the extracted folder to 'openvino'
    $ExtractedFolder = Get-ChildItem -Path $LibsDir -Directory | Where-Object { $_.Name -like "openvino_genai_windows*" } | Select-Object -First 1
    if ($ExtractedFolder) {
        Rename-Item -Path $ExtractedFolder.FullName -NewName "openvino"
    }
    
    Remove-Item $ZipPath
    Write-Host "OpenVINO installed to $OpenVinoDir"
} else {
    Write-Host "OpenVINO already installed at $OpenVinoDir"
}

# Instructions for setting environment variables
Write-Host "--------------------------------------------------"
Write-Host "Setup complete. To build, you need to set environment variables."
Write-Host "Run the following commands in your PowerShell session:"
Write-Host ""
Write-Host '$env:OPENVINO_ROOT = "'$OpenVinoDir'"'
Write-Host '$env:Path += ";$env:OPENVINO_ROOT\runtime\bin\intel64"'
Write-Host '$env:Path += ";$env:OPENVINO_ROOT\runtime\3rdparty\tbb\bin"'
Write-Host ""
Write-Host "Or run scripts/build_windows.ps1 which sets them automatically."
Write-Host "--------------------------------------------------"
