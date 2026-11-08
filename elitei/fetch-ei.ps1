<#
.SYNOPSIS
Descarga archivos de codigo de Elite Insights (rama master) a esta carpeta.

.EXAMPLE
./fetch-ei.ps1 SupportStatistics.cs
./fetch-ei.ps1 -Path GW2EI.Library/GW2EI.Services/GW2EIEvtcParser/EIData/Statistics/SupportStatistics.cs
#>
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string[]] $Path,
    [string] $Out = $PSScriptRoot
)

$root = 'https://raw.githubusercontent.com/baaron4/GW2-Elite-Insights-Parser/master/'
$known = @{
    'SupportStatistics.cs'            = 'EIData/Statistics/SupportStatistics.cs'
    'SupportPerAllyStatistics.cs'     = 'EIData/Statistics/SupportPerAllyStatistics.cs'
    'OffensiveStatistics.cs'          = 'EIData/Statistics/OffensiveStatistics.cs'
    'DamageStatistics.cs'             = 'EIData/Statistics/DamageStatistics.cs'
    'StatisticsHelper.cs'             = 'EIData/Statistics/StatisticsHelper.cs'
    'SingleActorStatusHelper.cs'      = 'EIData/Actors/ActorsHelper/SingleActorStatusHelper.cs'
    'HealthDamageEvent.cs'            = 'ParsedData/CombatEvents/DamageEvents/HealthDamageEvent.cs'
    'CombatItem.cs'                   = 'CombatItem.cs'
    'ArcDPSEnums.cs'                  = 'ParserHelpers/ArcDPSEnums.cs'
    'EXTHealingEvent.cs'              = 'Extensions/ExtensionCombatData/ExtensionCombatEvents/HealingStats/EXTHealingEvent.cs'
    'EXTHealingExtensionEvent.cs'     = 'Extensions/ExtensionCombatData/ExtensionCombatEvents/EXTHealingExtensionEvent.cs'
    'EXTFinalOutgoingHealingStat.cs'  = 'Extensions/ExtensionStatistics/HealingStats/EXTFinalOutgoingHealingStat.cs'
    'EXTActorHealingHelper.cs'        = 'Extensions/ExtensionActorHelpers/HealingStats/EXTActorHealingHelper.cs'
    'EXTSingleActorHealingHelper.cs'  = 'Extensions/ExtensionActorHelpers/HealingStats/EXTSingleActorHealingHelper.cs'
    'HealingStatsExtensionHandler.cs' = 'Extensions/ExtensionHandlers/HealingStats/HealingStatsExtensionHandler.cs'
}

foreach ($item in ($Path -split ',')) {
    $relative = if ($item.Contains('/')) { $item } else { $known[$item] }
    if (-not $relative) { Write-Error "Ruta desconocida: $item"; continue }
    if (-not $relative.StartsWith('GW2EI')) {
        $relative = "GW2EI.Library/GW2EI.Services/GW2EIEvtcParser/$relative"
    }
    $name = Split-Path $relative -Leaf
    $url = $root + $relative
    try {
        $text = Invoke-RestMethod -Uri $url -UseBasicParsing
        [IO.File]::WriteAllText((Join-Path $Out $name), $text)
        Write-Host "ok $name ($($text.Length) bytes)"
    }
    catch {
        Write-Error "fallo $name : $($_.Exception.Message)"
    }
}
