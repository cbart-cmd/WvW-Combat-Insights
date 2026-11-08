#Requires -Version 5.1
<#
.SYNOPSIS
Compila, publica y verifica una version del addon en el repo de distribucion de Nexus.

.DESCRIPTION
Pasos, en orden; se detiene en el primer fallo:
  1. Lee la version de Cargo.toml (con bump opcional).
  2. cargo fmt --check, tests de release y cargo build --release.
  3. GetAddonDef de la DLL compilada: nombre, version, firma, proveedor Direct y URL.
  4. Copia la DLL al repo de distribucion y escribe el sidecar MD5 (32 bytes, sin BOM ni salto).
  5. Comprueba .gitattributes, check-attr (filter/diff/merge/text unset) y que el blob
     preparado sea la DLL real (tamano y MZ).
  6. Commit y push en el repo fuente y en el de distribucion.
  7. Crea o actualiza la release <addon>-v<version> con DLL y MD5 y --latest=false.
  8. Deja solo las dos releases mas recientes de este addon.
  9. Descarga y verifica la URL directa, la antigua, el commit inmutable, el asset de la
     release y los dos sidecars (tamano, MD5 y cabecera MZ).

Sin copiar la DLL al juego: eso se hace a mano y con GW2 cerrado.

.PARAMETER Bump
patch, minor o major incrementan Cargo.toml antes de compilar. none usa la version actual.

.PARAMETER VerifyOnly
No compila ni publica: solo verifica lo que hay publicado para la version actual.

.PARAMETER NoPush
Compila y deja los commits hechos, pero no hace push ni crea la release.

.PARAMETER DistPath
Repo de distribucion. Por defecto ../gw2addonsreleases.

.PARAMETER WaitSeconds
Espera maxima para que la URL raw de main deje de servir el MD5 anterior en cache.

.EXAMPLE
./tools/publish.ps1 -Bump patch
./tools/publish.ps1 -VerifyOnly
#>
param(
    [ValidateSet('none', 'patch', 'minor', 'major')]
    [string] $Bump = 'none',
    [switch] $VerifyOnly,
    [switch] $NoPush,
    [string] $DistPath,
    [int] $WaitSeconds = 360,
    [string] $GitName,
    [string] $GitEmail
)

$ErrorActionPreference = 'Stop'
$Root = Split-Path $PSScriptRoot -Parent
if (-not $DistPath) { $DistPath = Join-Path (Split-Path $Root -Parent) 'gw2addonsreleases' }
$DistPath = (Resolve-Path $DistPath).Path

function Step($text) { Write-Host "`n=== $text" -ForegroundColor Cyan }
function Ok($text) { Write-Host "  ok  $text" -ForegroundColor Green }
function Warn($text) { Write-Host "  !!  $text" -ForegroundColor Yellow }
function Fail($text) { Write-Host "  xx  $text" -ForegroundColor Red; throw $text }

function Get-Field($text, $pattern) {
    $match = [regex]::Match($text, $pattern, 'Multiline')
    if (-not $match.Success) { Fail "patron no encontrado: $pattern" }
    $match.Groups[1].Value
}

function Read-Addon {
    $toml = Get-Content (Join-Path $Root 'addon.toml') -Raw
    [pscustomobject]@{
        Name    = Get-Field $toml '^name\s*=\s*"([^"]+)"'
        Signature = [int64](Get-Field $toml '^signature\s*=\s*(-?\d+)')
        Link    = Get-Field $toml '^update_link\s*=\s*"([^"]+)"'
        DataDir = Get-Field $toml '^data_directory\s*=\s*"([^"]+)"'
    }
}

function Read-Version { Get-Field (Get-Content (Join-Path $Root 'Cargo.toml') -Raw) '^version\s*=\s*"(\d+\.\d+\.\d+)"' }

function Set-Version($version) {
    $path = Join-Path $Root 'Cargo.toml'
    $text = Get-Content $path -Raw
    $text = [regex]::Replace($text, '(?m)^version\s*=\s*"\d+\.\d+\.\d+"', "version = `"$version`"")
    [IO.File]::WriteAllText($path, $text)
    Ok "Cargo.toml -> $version"
}

function Next-Version($version, $bump) {
    $parts = $version.Split('.') | ForEach-Object { [int]$_ }
    switch ($bump) {
        'major' { $parts[0]++; $parts[1] = 0; $parts[2] = 0 }
        'minor' { $parts[1]++; $parts[2] = 0 }
        'patch' { $parts[2]++ }
    }
    $parts -join '.'
}

function Download-Bytes($url, $attempts = 1) {
    for ($i = 1; ; $i++) {
        $client = New-Object System.Net.WebClient
        try { return , $client.DownloadData($url) }
        catch {
            # Los assets de una release recien creada pueden tardar unos segundos en servirse.
            if ($i -ge $attempts) { throw }
            Start-Sleep -Seconds 5
        }
        finally { $client.Dispose() }
    }
}

function Get-Md5([byte[]] $bytes) {
    $md5 = [Security.Cryptography.MD5]::Create()
    try { ([BitConverter]::ToString($md5.ComputeHash($bytes))).Replace('-', '').ToLower() } finally { $md5.Dispose() }
}

function Get-Slug($url) {
    $url = $url -replace '^git@github\.com:', '' -replace '^https://github\.com/', '' -replace '^https?://[^/]+/', '' -replace '\.git$', ''
    if ($url -notmatch '^[^/]+/[^/]+$') { Fail "remoto no reconocido como github.com/<owner>/<repo>: $url" }
    $url.Trim()
}

# Nombre y email de los commits: por defecto el login de gh y el email noreply de
# GitHub, para no publicar la direccion real ni depender de git config.
function Resolve-Identity {
    $login = try { gh api user --jq .login } catch { $null }
    if (-not $GitName) { $script:GitName = if ($login) { $login } else { 'cbart-cmd' } }
    if (-not $GitEmail) {
        $configured = git -C $Root config user.email
        $script:GitEmail = if ($configured -like '*@users.noreply.github.com') {
            $configured
        } elseif ($login) {
            "$login@users.noreply.github.com"
        } else {
            Fail 'no hay email de commit; pasa -GitEmail <usuario@users.noreply.github.com>'
        }
    }
    if ($script:GitEmail -notlike '*@users.noreply.github.com') {
        Warn "el email $($script:GitEmail) no es de GitHub y quedara visible en el repositorio publico"
    }
}

function Commit($repo, $message) {
    git -C $repo -c user.name=$script:GitName -c user.email=$script:GitEmail commit -q -m $message
    if ($LASTEXITCODE) { Fail "git commit ($repo)" }
}

# --- DLL real y su GetAddonDef -------------------------------------------------
$probeSource = @'
using System;
using System.Runtime.InteropServices;
public static class AddonProbe {
    [StructLayout(LayoutKind.Sequential)]
    public struct AddonVersion { public short major, minor, build, revision; }
    [StructLayout(LayoutKind.Sequential)]
    public struct AddonDefinition {
        public int signature; public int apiVersion; public IntPtr name; public AddonVersion version;
        public IntPtr author; public IntPtr description; public IntPtr load; public IntPtr unload;
        public uint flags; public int provider; public IntPtr updateLink;
    }
    [DllImport("kernel32", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr LoadLibraryEx(string path, IntPtr zero, uint flags);
    [DllImport("kernel32", CharSet = CharSet.Ansi, SetLastError = true)]
    public static extern IntPtr GetProcAddress(IntPtr module, string name);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate IntPtr Definition();
    public static string Probe(string path) {
        IntPtr module = LoadLibraryEx(path, IntPtr.Zero, 0x1 | 0x8);
        if (module == IntPtr.Zero) throw new Exception("LoadLibraryEx fallo: " + Marshal.GetLastWin32Error());
        IntPtr fn = GetProcAddress(module, "GetAddonDef");
        if (fn == IntPtr.Zero) throw new Exception("GetAddonDef no encontrado");
        AddonDefinition d = Marshal.PtrToStructure<AddonDefinition>(
            Marshal.GetDelegateForFunctionPointer<Definition>(fn)());
        return string.Join("|", new string[] {
            Marshal.PtrToStringAnsi(d.name),
            string.Format("{0}.{1}.{2}.{3}", d.version.major, d.version.minor, d.version.build, d.version.revision),
            d.signature.ToString(), d.provider.ToString(), Marshal.PtrToStringAnsi(d.updateLink)
        });
    }
}
'@
if (-not ('AddonProbe' -as [type])) { Add-Type -TypeDefinition $probeSource -Language CSharp }

function Test-Dll($path, $addon, $version) {
    if (-not (Test-Path $path)) { Fail "no existe la DLL: $path" }
    $bytes = [IO.File]::ReadAllBytes($path)
    if ($bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) { Fail "la DLL no empieza por MZ: $path" }
    $fields = [AddonProbe]::Probe($path).Split('|')
    $dllVersion = ($fields[1].Split('.')[0..2] -join '.')
    if ($fields[0] -ne $addon.Name) { Fail "nombre '$($fields[0])' != '$($addon.Name)' de addon.toml" }
    if ($dllVersion -ne $version) { Fail "version DLL $dllVersion != Cargo $version" }
    if ([int64]$fields[2] -ne $addon.Signature) { Fail "firma $($fields[2]) != $($addon.Signature)" }
    if ([int]$fields[3] -ne 3) { Fail "proveedor $($fields[3]) != 3 (Direct)" }
    if ($fields[4] -ne $addon.Link) { Fail "update_link '$($fields[4])' != '$($addon.Link)'" }
    Ok "GetAddonDef: $($fields[0]) v$dllVersion, firma $($fields[2]), Direct, URL exacta"
    [pscustomobject]@{ Bytes = $bytes; Length = $bytes.Length; Md5 = Get-Md5 $bytes }
}

function Get-DistSlug {
    Get-Slug (git -C $DistPath remote get-url origin)
}

# --- Verificacion de lo publicado ---------------------------------------------
function Test-Published($dll, $md5, $addon, $version, $slug, $tag, $dllName, $distCommit) {
    Step "Verificando lo publicado para v$version"
    $deadline = (Get-Date).AddSeconds($WaitSeconds)
    $targets = [ordered]@{
        'URL exacta de update_link' = $addon.Link
        'URL antigua (refs/heads/main)' = "https://github.com/$slug/raw/refs/heads/main/$dllName"
        'Commit inmutable de distribucion' = "https://raw.githubusercontent.com/$slug/$distCommit/$dllName"
        'Asset de la release' = "https://github.com/$slug/releases/download/$tag/$dllName"
    }
    $failed = @()
    foreach ($name in $targets.Keys) {
        $url = $targets[$name]
        try {
            $bytes = Download-Bytes $url 12
            $got = Get-Md5 $bytes
            $okSize = $bytes.Length -eq $dll.Length
            $okMd5 = $got -eq $md5
            $okMz = $bytes[0] -eq 0x4D -and $bytes[1] -eq 0x5A
            if ($okSize -and $okMd5 -and $okMz) {
                Ok "$name : $($bytes.Length) bytes, MD5 y MZ correctos"
            } else {
                $failed += "$name (len $($bytes.Length), md5 $got, mz $okMz)"
                Warn "$name no coincide: $($bytes.Length) bytes, md5 $got, mz=$okMz"
            }
        } catch {
            $failed += "$name ($($_.Exception.Message))"
            Warn "$name no descarga: $($_.Exception.Message)"
        }
    }
    $sidecars = [ordered]@{
        'Sidecar raw de main' = "$($addon.Link).md5"
        'Sidecar de la release' = "https://github.com/$slug/releases/download/$tag/$dllName.md5"
    }
    foreach ($name in $sidecars.Keys) {
        $url = $sidecars[$name]
        while ($true) {
            try {
                $bytes = Download-Bytes $url 12
                $text = [Text.Encoding]::ASCII.GetString($bytes)
                if ($bytes.Length -eq 32 -and $text -eq $md5) { Ok "$name : 32 bytes y MD5 correcto"; break }
                if ((Get-Date) -lt $deadline) {
                    Warn "$name sirve '$text' (32=$($bytes.Length -eq 32)); cache de GitHub, reintento en 30 s"
                    Start-Sleep -Seconds 30
                    continue
                }
                $failed += "$name (valor '$text')"
                Warn "$name sigue sin coincidir tras esperar la cache"
                break
            } catch {
                $failed += "$name ($($_.Exception.Message))"
                Warn "$name no descarga: $($_.Exception.Message)"
                break
            }
        }
    }
    if ($failed.Count -gt 0) { Fail "verificacion incompleta:`n  - " + ($failed -join "`n  - ") }
    Ok "Publicacion verificada: DLL, URLs y sidecars"
}

# --- Flujo principal ----------------------------------------------------------
$addon = Read-Addon
$dllName = "$($addon.DataDir).dll"
$tagPrefix = $addon.DataDir.Replace('_', '-')
$version = Read-Version
$sourceSlug = Get-Slug (git -C $Root remote get-url origin)
Resolve-Identity

if (-not $VerifyOnly) {
    if ($Bump -ne 'none') {
        $version = Next-Version $version $Bump
        Step "Publicando v$version (bump $Bump)"
        Set-Version $version
    } else {
        Step "Publicando v$version (sin bump)"
    }
    Ok "fuente: github.com/$sourceSlug"
    Ok "commits: $GitName <$GitEmail>"
    if ((git -C $Root status --porcelain) -and $Bump -eq 'none') {
        Warn "hay cambios sin commitear; se incluyen solo los archivos ya versionados"
    }

    Step 'Formato, tests y compilacion release'
    Push-Location $Root
    try {
        cargo fmt --check; if ($LASTEXITCODE) { Fail 'cargo fmt --check' }
        Ok 'fmt'
        cargo test --release 2>&1 | Tee-Object -Variable testOut | Out-Null
        if ($LASTEXITCODE) { $testOut | Select-Object -Last 30 | Out-Host; Fail 'cargo test --release' }
        ($testOut | Select-String 'test result: ') | ForEach-Object { Ok $_.Line.Trim() }
        cargo build --release; if ($LASTEXITCODE) { Fail 'cargo build --release' }
        Ok 'build release'
    } finally { Pop-Location }
}

$built = Join-Path $Root "target\release\$dllName"
Step "Comprobando la DLL compilada"
$local = Test-Dll $built $addon $version

if (-not $VerifyOnly) {
    Step "Actualizando $DistPath"
    $distDll = Join-Path $DistPath $dllName
    Copy-Item $built $distDll -Force
    $sidecar = "$distDll.md5"
    [IO.File]::WriteAllBytes($sidecar, [Text.Encoding]::ASCII.GetBytes($local.Md5))
    $side = [IO.File]::ReadAllBytes($sidecar)
    if ($side.Length -ne 32 -or [Text.Encoding]::ASCII.GetString($side) -ne $local.Md5) { Fail 'sidecar MD5 incorrecto' }
    Ok "DLL y sidecar MD5 ($($local.Md5)) copiados"

    $attrLine = "$dllName -filter -diff -merge -text"
    $attributes = Get-Content (Join-Path $DistPath '.gitattributes') -Raw
    if (-not $attributes.Contains($attrLine)) { Fail ".gitattributes no contiene '$attrLine'" }
    Ok '.gitattributes con la excepcion de esta DLL'

    foreach ($attribute in 'filter', 'diff', 'merge', 'text') {
        $value = (git -C $DistPath check-attr $attribute -- $dllName) -replace '.*: ', ''
        if ($value -ne 'unset') { Fail "check-attr $attribute = $value (debe ser unset)" }
    }
    Ok 'check-attr: filter/diff/merge/text unset'

    git -C $DistPath add $dllName "$dllName.md5"
    $staged = git -C $DistPath cat-file -s ":$dllName"
    if ([int]$staged -ne $local.Length) { Fail "blob preparado ($staged) != DLL compilada ($($local.Length))" }
    Ok "blob preparado identico a la DLL ($staged bytes)"

    if ($NoPush) {
        Warn '-NoPush: no se hace commit, push ni release'
    } else {
        Step 'Commits y push'
        Push-Location $Root
        try {
            git add -u
            git diff --cached --quiet
            if (-not $LASTEXITCODE) {
                Ok 'fuente: sin cambios que commitear'
            } else {
                Commit $Root "v$version`: publicar $($addon.Name)"
                Ok "fuente: $(git rev-parse --short HEAD)"
            }
            git push origin HEAD; if ($LASTEXITCODE) { Fail 'git push (fuente)' }
            Ok 'fuente subida'
        } finally { Pop-Location }
        $untracked = git -C $Root status --porcelain | Where-Object { $_ -like '??*' }
        if ($untracked) { Warn "archivos sin versionar fuera del commit: $($untracked -join ', ')" }

        git -C $DistPath add $dllName "$dllName.md5"
        git -C $DistPath diff --cached --quiet
        if ($LASTEXITCODE) {
            Commit $DistPath "v$version`: publicar $($addon.Name)"
            Ok "distribucion: $(git -C $DistPath rev-parse --short HEAD)"
        } else {
            Ok 'distribucion: sin cambios que commitear'
        }
        git -C $DistPath push origin HEAD:main; if ($LASTEXITCODE) { Fail 'git push (distribucion)' }
        Ok 'distribucion subida'

        Step "Release $tagPrefix-v$version"
        $slug = Get-DistSlug
        $assets = @($distDll, (Join-Path $DistPath "$dllName.md5"))
        foreach ($asset in $assets) { if (-not (Test-Path $asset)) { Fail "falta el asset $asset" } }
        $existing = gh release list --repo $slug --limit 100 --json tagName --jq '.[].tagName'
        if ($existing -contains "$tagPrefix-v$version") {
            gh release upload "$tagPrefix-v$version" @assets --repo $slug --clobber | Out-Null
            if ($LASTEXITCODE) { Fail "gh release upload $tagPrefix-v$version" }
            Ok 'release existente actualizada'
        } else {
            gh release create "$tagPrefix-v$version" @assets --repo $slug `
                --title "$($addon.Name) v$version" --latest=false `
                --notes "v$version. Ver README.md del addon para el detalle." | Out-Null
            if ($LASTEXITCODE) { Fail "gh release create $tagPrefix-v$version" }
            Ok "release $tagPrefix-v$version creada (--latest=false)"
        }

        Step 'Dejando solo las dos releases mas recientes de este addon'
        $mine = @(gh release list --repo $slug --limit 200 --json tagName,createdAt |
            ConvertFrom-Json |
            Where-Object { $_.tagName -like "$tagPrefix-v*" } |
            Sort-Object { [version]($_.tagName -replace '^.*-v', '') } -Descending)
        $mine | Select-Object -Skip 2 | ForEach-Object {
            gh release delete $_.tagName --repo $slug --yes | Out-Null
            if ($LASTEXITCODE) { Fail "gh release delete $($_.tagName)" }
            Ok "eliminada release $($_.tagName) (tag conservado)"
        }
        $mine | Select-Object -First 2 | ForEach-Object { Ok "se conserva $($_.tagName)" }
    }
}

$distCommit = git -C $DistPath rev-parse --short HEAD
$slug = Get-DistSlug
if ($NoPush) {
    Warn 'Sin push: no se verifica la publicacion'
} else {
    Test-Published $local $local.Md5 $addon $version $slug "$tagPrefix-v$version" $dllName $distCommit
}

Step 'Resumen'
Ok "version $version"
Ok "DLL $($local.Length) bytes, MD5 $($local.Md5)"
Ok "fuente $(git -C $Root rev-parse --short HEAD), distribucion $distCommit"
Warn 'No se ha copiado la DLL al juego: hazlo con GW2 cerrado y solo con la DLL verificada'
