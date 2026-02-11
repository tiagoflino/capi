# install.ps1

param (
    [string]$Version = "latest",
    [string]$LocalPath = ""
)

$ErrorActionPreference = "Stop"

# Configuration
$AppName = "capi"
$InstallDir = Join-Path $env:LOCALAPPDATA $AppName
$BinDir = Join-Path $InstallDir "bin"
$RepoOwner = "tiagoflino"
$RepoName = "capi"
$ZipName = "capi-windows-x64.zip"

# Check Architecture
if ($env:PROCESSOR_ARCHITECTURE -ne "AMD64") {
    Write-Error "This installer only supports Windows x64."
    exit 1
}

Write-Host "Installing $AppName..."

# Create Install Directory
if (Test-Path $InstallDir) {
    Remove-Item -Path $InstallDir -Recurse -Force
}
New-Item -ItemType Directory -Path $BinDir -Force | Out-Null

$ZipDestination = Join-Path $InstallDir $ZipName

if ($LocalPath -ne "") {
    # Install from local file
    Write-Host "Installing from local file: $LocalPath"
    Copy-Item -Path $LocalPath -Destination $ZipDestination -Force
} else {
    # Determine Download URL
    $DownloadUrl = "https://github.com/$RepoOwner/$RepoName/releases/download/$Version/$ZipName" 
    if ($Version -eq "latest") {
        $DownloadUrl = "https://github.com/$RepoOwner/$RepoName/releases/latest/download/$ZipName"
    }

    Write-Host "Downloading $AppName from $DownloadUrl..."
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipDestination
    } catch {
        Write-Error "Failed to download release. Please check your internet connection or the version."
        Write-Error "Error details: $_"
        exit 1
    }
}

# Extract
Write-Host "Extracting to $BinDir..."
Expand-Archive -Path $ZipDestination -DestinationPath $BinDir -Force
Remove-Item $ZipDestination

# Add to PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    Write-Host "Adding $BinDir to PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
    $env:Path += ";$BinDir"
    Write-Host "PATH updated. You may need to restart your terminal."
} else {
    Write-Host "$BinDir is already in PATH."
}

Write-Host "--------------------------------------------------"
Write-Host "$AppName installed successfully!"
Write-Host "Run '$AppName' to start."
Write-Host "--------------------------------------------------"
