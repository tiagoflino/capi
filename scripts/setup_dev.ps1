$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path $ScriptDir -Parent
$LibsDir = Join-Path $ProjectRoot "libs"
$OpenVinoDir = Join-Path $LibsDir "openvino"
$ZipPath = Join-Path $LibsDir "openvino_genai.zip"

$ConfPath = Join-Path $ProjectRoot "openvino.conf"
$ConfVars = @{}
Get-Content $ConfPath | ForEach-Object {
    $line = $_.Trim()
    if ($line -and -not $line.StartsWith("#")) {
        $parts = $line -split '=', 2
        $ConfVars[$parts[0].Trim()] = $parts[1].Trim()
    }
}
$OvVersion = $ConfVars["OPENVINO_VERSION"]
$OvShortVer = $ConfVars["OPENVINO_SHORT"]
$OpenVinoUrl = "https://storage.openvinotoolkit.org/repositories/openvino_genai/packages/$OvShortVer/windows/openvino_genai_windows_${OvVersion}_x86_64.zip"

Write-Host "OpenVINO version: $OvVersion (from openvino.conf)"

if (-not (Test-Path $LibsDir)) {
    New-Item -ItemType Directory -Path $LibsDir | Out-Null
    Write-Host "Created $LibsDir"
}

if (-not (Test-Path $OpenVinoDir)) {
    Write-Host "Downloading OpenVINO GenAI from $OpenVinoUrl..."
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

Write-Host "--------------------------------------------------"
Write-Host "Setup complete. To build, you need to set environment variables."
Write-Host "Run the following commands in your PowerShell session:"
Write-Host ""
Write-Host ('$env:OPENVINO_ROOT = "' + $OpenVinoDir + '"')
Write-Host '$env:Path += ";$env:OPENVINO_ROOT\runtime\bin\intel64"'
Write-Host '$env:Path += ";$env:OPENVINO_ROOT\runtime\3rdparty\tbb\bin"'
Write-Host ""
Write-Host "Or run scripts/build_windows.ps1 which sets them automatically."
Write-Host "--------------------------------------------------"
