# Install CHAOS RPG on Windows from the latest GitHub release.
#
#   irm https://raw.githubusercontent.com/Mattbusel/chaos-rpg/master/install.ps1 | iex
#
# Puts chaos-rpg.exe, chaos-rpg-graphical.exe and chaos-rpg-proof.exe in
# %LOCALAPPDATA%\Programs\chaos-rpg after checking the SHA-256 of the download,
# and adds that folder to your user PATH. Pin a version with $env:CHAOS_RPG_VERSION = "v2.2.1".
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$repo = 'Mattbusel/chaos-rpg'
$target = 'x86_64-pc-windows-msvc'
$dest = Join-Path $env:LOCALAPPDATA 'Programs\chaos-rpg'

if ($env:CHAOS_RPG_VERSION) {
    $tag = $env:CHAOS_RPG_VERSION
} else {
    $tag = (Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest" -Headers @{ 'User-Agent' = 'chaos-rpg-installer' }).tag_name
}
if ($tag -notmatch '^v\d') { throw "Could not find the latest CHAOS RPG release (got '$tag')." }

$name = "chaos-rpg-$tag-$target"
$url = "https://github.com/$repo/releases/download/$tag/$name.zip"
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("chaos-rpg-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp | Out-Null

try {
    Write-Host "Downloading CHAOS RPG $tag for Windows"
    $zip = Join-Path $tmp "$name.zip"
    Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $zip
    $sumFile = Join-Path $tmp "$name.zip.sha256"
    Invoke-WebRequest -UseBasicParsing -Uri "$url.sha256" -OutFile $sumFile

    $want = ((Get-Content $sumFile -Raw).Trim() -split '\s+')[0].ToLower()
    $got = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLower()
    if ($want -ne $got) { throw "Checksum mismatch (expected $want, got $got). Nothing was installed." }
    Write-Host "Checksum OK"

    Expand-Archive -Path $zip -DestinationPath $tmp -Force
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    foreach ($b in 'chaos-rpg', 'chaos-rpg-graphical', 'chaos-rpg-proof') {
        Copy-Item (Join-Path $tmp "$name\$b.exe") $dest -Force
    }
    # Keep an existing settings file; the game reads it from next to the .exe.
    $cfg = Join-Path $dest 'chaos_config.toml'
    if (-not (Test-Path $cfg)) { Copy-Item (Join-Path $tmp "$name\chaos_config.toml") $cfg }
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not $userPath) { $userPath = '' }
if (($userPath -split ';') -notcontains $dest) {
    $newPath = if ($userPath) { "$userPath;$dest" } else { $dest }
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    Write-Host "Added $dest to your user PATH (open a new terminal to use it)."
}
if (($env:Path -split ';') -notcontains $dest) { $env:Path = "$env:Path;$dest" }

Write-Host ""
Write-Host "Installed CHAOS RPG $tag to $dest"
Write-Host "  chaos-rpg-graphical   the game in its own window (start here)"
Write-Host "  chaos-rpg-proof       the Proof Engine version (preview)"
Write-Host "  chaos-rpg             the terminal version"
Write-Host ""
Write-Host "Windows may show 'unknown publisher' the first time: click More info, then Run anyway."
