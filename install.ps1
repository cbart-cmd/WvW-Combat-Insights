<#
.SYNOPSIS
  Compila (opcional) e instala WvW Combat Insights (fight_analysis_danger.dll) en la carpeta addons de Guild Wars 2.
.DESCRIPTION
  Copia fight_analysis_danger.dll a la carpeta addons. Requiere Nexus y, opcionalmente, WvW Fight Analysis.
.EXAMPLE
  .\install.ps1            # copia la DLL ya compilada
  .\install.ps1 -Build     # compila y copia
  .\install.ps1 -Dest "D:\Games\Guild Wars 2\addons"
#>
param(
    [string]$Dest = "C:\Program Files\Guild Wars 2\addons",
    [switch]$Build
)

$ErrorActionPreference = "Stop"
$name = "fight_analysis_danger.dll"
$source = Join-Path $PSScriptRoot "target\release\fight_analysis_danger.dll"

if ($Build) {
    $cargoHome = Join-Path $PSScriptRoot "..\.cargo"; if (Test-Path $cargoHome) { $env:CARGO_HOME = (Resolve-Path $cargoHome).Path; $env:RUSTUP_HOME = (Resolve-Path (Join-Path $PSScriptRoot "..\.rustup")).Path; $env:Path = "$env:CARGO_HOME\bin;$env:Path" } else { $env:Path += ";$env:USERPROFILE\.cargo\bin" }
    cargo test --manifest-path (Join-Path $PSScriptRoot "Cargo.toml")
    if ($LASTEXITCODE -ne 0) { throw "Los tests han fallado: no se instala nada." }
    cargo build --release --manifest-path (Join-Path $PSScriptRoot "Cargo.toml")
    if ($LASTEXITCODE -ne 0) { throw "cargo build ha fallado." }
}

if (-not (Test-Path $source)) {
    throw "No existe $source. Ejecuta el script con -Build o compila con 'cargo build --release'."
}
if (-not (Test-Path $Dest)) {
    throw "No existe la carpeta de destino: $Dest. Usa -Dest para indicar la carpeta addons del juego."
}
if (Get-Process -Name "Gw2-64" -ErrorAction SilentlyContinue) {
    throw "Guild Wars 2 esta abierto y mantiene la DLL bloqueada. Cierra el juego y vuelve a ejecutar el script."
}

$target = Join-Path $Dest $name
try {
    Copy-Item -Path $source -Destination $target -Force
} catch [System.UnauthorizedAccessException] {
    # Program Files exige permisos de administrador: se relanza el script elevado.
    $principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
    if ($principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw }
    Write-Host "Se necesitan permisos de administrador para escribir en $Dest. Relanzando..."
    $argList = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -Dest `"$Dest`""
    Start-Process -FilePath (Get-Process -Id $PID).Path -ArgumentList $argList -Verb RunAs -Wait
    return
}

$info = Get-Item $target
Write-Host "Copiado: $($info.FullName) ($([math]::Round($info.Length / 1KB)) KB, $($info.LastWriteTime))"

Write-Host "Abre Nexus (Ctrl+O) y carga 'WvW Combat Insights' en la lista de addons."
