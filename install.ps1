$ErrorActionPreference = "Stop"

$Repo    = "duwei0227/apimock"
$Archive = "mock-windows-x86_64.zip"
$Binary  = "mock.exe"

# Install to user's local bin; create it if needed
$InstallDir = "$env:USERPROFILE\.local\bin"
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

Write-Host "Fetching latest release..."
$Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$Tag     = $Release.tag_name

if (-not $Tag) {
    Write-Error "Could not fetch latest release."
    exit 1
}

Write-Host "Downloading $Binary $Tag..."
$Tmp = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
$ZipPath = Join-Path $Tmp $Archive

try {
    Invoke-WebRequest "https://github.com/$Repo/releases/download/$Tag/$Archive" -OutFile $ZipPath
    Expand-Archive $ZipPath -DestinationPath $Tmp
    Move-Item (Join-Path $Tmp $Binary) (Join-Path $InstallDir $Binary) -Force
} finally {
    Remove-Item $Tmp -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "Installed: $InstallDir\$Binary"

# Add to PATH for current user if not already there
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$CurrentPath", "User")
    Write-Host "Added $InstallDir to your user PATH."
    Write-Host "Restart your terminal for the change to take effect."
}

Write-Host "Done. Run: mock --help"
