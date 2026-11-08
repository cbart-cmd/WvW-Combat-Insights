# elitei

Copias de referencia del codigo de [Elite Insights](https://github.com/baaron4/GW2-Elite-Insights-Parser)
para contrastar como calcula cada metrica. No se compila ni forma parte del addon: solo sirve para saber
que regla exacta replica nuestro parser y no depender de la red al comparar.

Todos los `.cs` vienen de la rama `master`, descargados el 2026-11-08 con `fetch-ei.ps1`.
Licencia del proyecto original: MIT (ver su repositorio).

## Que mirar en cada archivo

| Archivo | Para que |
| --- | --- |
| `HealingStatsExtensionHandler.cs` | Que bytes de la extension son curas (`IsShields == 0`) y cuales barrera (`IsShields > 0`). |
| `EXTHealingEvent.cs`, `EXTHealingExtensionEvent.cs` | `HealingDone`, bits de peer en `is_offcycle`, `ToFriendly` = `IFF.Friend`. |
| `EXTActorHealingHelper.cs`, `EXTSingleActorHealingHelper.cs` | Las curas salientes incluyen mascotas y **cualquier** objetivo aliado, sin exigir escuadron. |
| `EXTFinalOutgoingHealingStat.cs` | La columna Healing es la suma de todo lo curado a aliados; `ActorHealing` excluye mascotas. |
| `SupportStatistics.cs`, `SupportPerAllyStatistics.cs` | Limpias (una por condicion retirada entera, sin contar las propias) y strips con `IsDownedBeforeNext90`. |
| `OffensiveStatistics.cs` | `DownContribution` = dano de salud desde que el objetivo sale de 90% hasta el derribo; `HealthDamage` excluye barrera. |
| `HealthDamageEvent.cs` | `NegateShieldDamage`: el dano a barrera no cuenta como dano de salud. |
| `DamageStatistics.cs` | Columnas de dano por tipo (potencia/condiciones/vida/escudo). |
| `SingleActorStatusHelper.cs` | `IsDownBeforeNext90` y estados de derribo/muerte usados por lo anterior. |
| `ArcDPSEnums.cs`, `CombatItem.cs` | `IFF` (0 = amigo, 1 = enemigo) y lectura de los bytes del evento. |

## Herramientas

- `fetch-ei.ps1`: vuelve a bajar uno o varios de estos archivos de `master` y sobrescribe la copia local.
