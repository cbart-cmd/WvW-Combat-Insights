#Requires -Version 5.1
<#
.SYNOPSIS
Comprueba, sin publicar, la DLL compilada y lo que hay publicado en el repo de distribucion.

.DESCRIPTION
Lee la version de Cargo.toml, valida GetAddonDef de target\release y descarga la URL exacta de
update_link, la URL antigua, el commit actual de distribucion y el asset de la release para
comparar tamano, MD5 y cabecera MZ. Tambien comprueba los dos sidecars MD5.

Requiere una compilacion release reciente: ./tools/publish.ps1 -Bump patch la hace.

.EXAMPLE
./tools/verify-release.ps1
./tools/verify-release.ps1 -WaitSeconds 600
#>
param(
    [string] $DistPath,
    [int] $WaitSeconds = 360
)

$params = @{ VerifyOnly = $true; WaitSeconds = $WaitSeconds }
if ($DistPath) { $params.DistPath = $DistPath }

try {
    & (Join-Path $PSScriptRoot 'publish.ps1') @params
    if (-not $?) { exit 1 }
} catch {
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}
