# Download and extract the pinned JPEXS portable zip into src-tauri/resources/jpexs/
# Mirrors src-tauri/build.rs — keep URL and hash in sync when upgrading.

$ErrorActionPreference = "Stop"

$ReleaseTag = "version26.0.0"
$ZipName = "ffdec_26.0.0.zip"
$Url = "https://github.com/jindrapetrik/jpexs-decompiler/releases/download/$ReleaseTag/$ZipName"
$ExpectedSha256 = "e13509d0ed11c6d77bb1588701d803eb2c706e2ba4eaab8bc4477255cfc718f4"

$Root = Split-Path -Parent $PSScriptRoot
$Out = Join-Path $Root "src-tauri\resources\jpexs"

$Tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("stardelta-" + $ZipName + "-" + [Guid]::NewGuid().ToString())

try {
    if ($env:STARDELTA_FFDEC_ZIP) {
        Copy-Item -Force $env:STARDELTA_FFDEC_ZIP $Tmp
    } else {
        Write-Host "Downloading $Url ..."
        Invoke-WebRequest -Uri $Url -OutFile $Tmp -UseBasicParsing
    }

    $hash = Get-FileHash -Algorithm SHA256 $Tmp
    if ($hash.Hash.ToLowerInvariant() -ne $ExpectedSha256) {
        Write-Error "SHA-256 mismatch. expected $ExpectedSha256 got $($hash.Hash.ToLowerInvariant())"
    }

    if (Test-Path $Out) {
        Remove-Item -Recurse -Force $Out
    }
    New-Item -ItemType Directory -Path $Out | Out-Null
    Write-Host "Extracting to $Out ..."
    Expand-Archive -Path $Tmp -DestinationPath $Out -Force

    $Jar = Join-Path $Out "ffdec.jar"
    if (-not (Test-Path $Jar)) {
        Write-Error "ffdec.jar not found after extract"
    }
    Write-Host "OK: $Jar"
} finally {
    if (Test-Path $Tmp) {
        Remove-Item -Force $Tmp
    }
}
