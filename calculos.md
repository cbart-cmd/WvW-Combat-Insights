# Auditoria de calculos: WvW Combat Insights frente a AxiBridge

## Estado y alcance

Auditoria inicial de v0.2.9, actualizada con los cambios de **v0.2.10**:
Down contribution adopta la metodologia nativa de AxiLog 1.16.0; las demas
metricas no se adaptan en esta version. Resumen incorpora su suma por color
en `down c` y Destacados permite alternar las armas desde el menu contextual.
Los tooltips muestran el nombre/valor, no la explicacion del algoritmo.
Se eliminan los ajustes top/top_dps/top_strips y el intervalo fijo de contribucion.

Desde v0.2.13, la vista **En directo · provisional** reutiliza este mismo motor sobre
eventos recibidos del bridge de arcdps (solo flujo de escuadron para importes).
Su poblacion y duracion dependen del tramo observado del combate propio y no son
equivalentes al EVTC definitivo. Desde v0.2.16 solo muestra nuestro equipo y calcula
hostilidad mediante IDs de equipo recibidos o afinidad hostil contra jugadores identificados,
sin depender del color; sin color propio verificado usa "Nuestro equipo" neutro.
La ruta de logs conserva colores y formulas anteriores. Sin eventos Healing Stats compatibles
no se inventan esos datos. Los informes provisionales no forman parte del historial.
Healing ya no recorta el ranking/copia a 4–10 filas; esto no altera sus formulas ni cobertura.

Referencias fijadas para que la comparacion sea reproducible:

- Base auditada: v0.2.9 (anterior al snapshot publico);
  implementacion actual de contribucion: v0.2.10, [modulo local](src/contribution.rs).
- AxiBridge: commit `44d58c555ffb3c4d9adae3522f8472985b9fc51c`, rama main.
- Parser nativo usado por esa revision: `@axiapps/axilog` 1.16.0,
  tag `v1.16.0`, commit `9bd82979dbb49a000b7af3b15713d956f011bb00`.
  Dependencia declarada en [package.json de AxiBridge][ab-package].

**Limite de la verificacion:** se han seguido lectores, acumuladores y parser
nativo. No se ha ejecutado una comparacion numerica de ambos programas sobre un
mismo EVTC. Por tanto, "misma base" no significa igualdad bit a bit demostrada.
Los ejemplos de este documento son consecuencias de las formulas, no resultados
medidos en un combate.

AxiBridge tambien acepta informes de Elite Insights/dps.report. Un campo con el
mismo nombre puede venir de ese parser o de AxiLog y tener otra metodologia.
Aqui se distingue la ruta nativa actual de los informes importados.

## Respuesta directa: nuestra columna Dano no es necesariamente la suya

**Mismo concepto basico, pero no la misma definicion completa.**

Ambas acumulan importes positivos de golpes directos y condiciones, con credito
de mascotas al propietario. Ambas pueden incluir dano contra un objetivo ya
derribado: Dano no es solamente dano previo al down.

Sin embargo, el lector habitual de AxiBridge obtiene Dano de
`dpsAll[0].damage`, no de una suma limitada a jugadores enemigos. En su ruta
nativa ese campo procede del total de AxiLog, que incluye NPC/gadgets hostiles
admitidos por su clasificacion de enemigos y tiene un recorrido especifico para
mascotas. Nuestro valor inicial `vs_players_only=true` exige que el destinatario
sea un jugador de otro color conocido.

Ejemplo: 100.000 a jugadores enemigos y 20.000 a un NPC hostil admitido por
AxiLog pueden producir **100.000 aqui y 120.000 alli**, sin que ninguno este
sumando DPS en lugar de dano.

Otros motivos de discrepancia:

1. **Eventos admitidos:** nosotros usamos listas explicitas de resultados;
   AxiLog excluye CC y dano de breakbar, pero admite el resto de resultados con
   importe positivo. No son predicados identicos, aunque coincidan para muchos
   eventos normales.
2. **Mascotas e identidades:** nosotros resolvemos el master con un mapa de
   instancias actualizado durante el recorrido; AxiLog usa un registro temporal
   y agrupa identidades. Reutilizacion de instancias o relogs pueden cambiar
   la atribucion.
3. **Poblacion:** nuestras filas abarcan los tres colores completos; AxiBridge
   agrega principalmente el escuadron del POV. Su lado enemigo tiene ademas
   reglas propias para dano saliente, no una repeticion simetrica de nuestro
   filtro por colores.
4. **Equipos desconocidos/cambiantes:** nuestras metricas ofensivas dependen del
   ultimo color reconocido; AxiLog combina equipos, afinidad `iff` y otras
   evidencias de pertenencia.

Desactivar nuestro filtro de solo jugadores tampoco garantiza igualdad: permite
destinos no jugadores, pero no reproduce las reglas de hostilidad de AxiLog.

Fuentes: [nuestro parser](src/evtc.rs), [lectores de AxiBridge][ab-dashboard],
[acumulador de dano][ax-damage], [clasificacion WvW][ax-wvw] y
[adaptador de salida][ax-ei].

## Inventario de metricas visibles o derivadas

| Metrica / superficie | Nuestro calculo | AxiBridge / AxiLog nativo | Veredicto |
|---|---|---|---|
| Dano por jugador, columna y tooltip | Directo + condiciones admitidos; solo destinos jugadores de otro color por defecto | `dpsAll[0].damage`; total nativo, con enemigos no jugadores y credito de mascotas | Misma base; alcance y atribucion diferentes |
| DPS, columna y tooltip | Dano / segundos del combate definidos por nosotros | `dpsAll[0].dps`; total nativo / duracion nativa, minimo 1 s, redondeado en el adaptador | Misma division; no igualdad garantizada |
| Dano directo / condiciones, tooltip | Desglose por `buff=0` / `buff=1` de los eventos admitidos | El lector de Dano usa el total; otras estadisticas/distribuciones no constituyen prueba de un desglose identico | Suma local verificada; equivalencia del desglose no certificada |
| Down contribution, columna `down c` y tooltip | Desde v0.2.10: ancla >=99%, anticipo 2 s, reset 2,1 s y atribucion temporal | Misma metodologia temporal y de eventos | Algoritmo adaptado; poblacion/agrupacion por cuenta siguen distintas |
| Corrups por jugador | Numero de retiradas completas de boons admitidas, con filtros propios y credito de mascotas | Lector normal `support[0].boonStrips`; conteo nativo de retiradas del actor, sin fold de mascotas en ese contador | Misma unidad basica; filtros y credito diferentes |
| Dano total por color, Resumen | Suma del dano de todos los agentes jugadores de ese color | Los totales de escuadron agregan sus miembros y sus propios totales | Suma equivalente como operacion; poblacion y sumandos distintos |
| Corrups totales por color, Resumen | Suma del mismo contador de corrups de las filas del color | Agregacion de `boonStrips` sobre la poblacion seleccionada | Misma operacion; no el mismo total garantizado |
| Down c total por color, Resumen | Suma del Down contribution local de los jugadores del color | Agregacion de contribucion sobre su poblacion | Misma suma y metodologia; poblacion distinta |
| Kills causadas, Resumen y tooltip | Resultado 8 contra jugador de otro color; credito al atacante/propietario | Resultado killing blow contra `enemies`; credito temporal al propietario, identidades agrupadas | Misma senal base; destinos y guardas distintos |
| Muertes sufridas, tooltip de equipo | Resultado 8 recibido por un jugador | `CHANGE_DEAD` del actor en el contador nativo de muertes | Diferente |
| Downs sufridos, tooltip de equipo | Resultado 9 recibido, `buff=0` | `defenses[0].downCount`, resultado downed recibido en la ruta nativa | Misma senal base; guardas e identidades distintas |
| Jugadores registrados por color | Cuenta agentes jugadores con color, deduplicados por direccion | Roster agrupado por cuenta o instancia segun identidad; filtros de escuadron/participacion segun vista | Diferente |
| Curas, Healing | Importes observados de extension hacia miembros del escuadron, incluidos receptores mascota resolubles a su propietario | `computeSquadHealing`: suma de `outgoingHealingAllies`, matriz de destinatarios jugadores | Misma fuente; alcance y sanitizacion diferentes |
| Barrera, Healing | Importes de extension marcados como barrera, con el mismo alcance local | `computeSquadBarrier`: suma de `outgoingBarrierAllies` | Misma fuente; alcance y sanitizacion diferentes |
| Total de Healing | Curas + barrera disponibles; si falta una, suma parcial identificada como limite inferior | Existen los dos componentes; no se ha acreditado un equivalente unico con nuestra semantica de ausencia | Combinacion local; componentes no equivalentes garantizados |
| Limpias, Healing | Una por condicion retirada completamente; escuadron al tiempo del evento, propias y mascotas conocidas | Predeterminado arcdps: suma de tres contadores nativos, con exclusiones especificas | Diferente |
| Aportacion por segundo, tooltip Healing | Metrica seleccionada / duracion local del combate | HPS/BPS del parser usan duracion nativa; agregaciones de varios logs pueden usar tiempo activo | Misma division basica; denominador/superficie distintos |
| Duracion del combate | Ultimo menos primer tiempo de eventos ordinarios `NONE` | Duracion del encounter nativo desde inicio del log hasta ultimo evento | Diferente |
| Puntuacion inicial oculta de Destacados | Dano + 2000 * corrups + Down contribution | No se ha identificado esa formula como metrica equivalente de AxiBridge | Propia, solicitada para este addon |
| Cobertura / ausencia en Healing | Parcial, ausente, incompatible o grupo no identificable; `-` no es 0 | Disponibilidad de extension y `hasHealAddon`; helpers devuelven 0 cuando faltan arrays | Semantica de disponibilidad distinta |

No confundir **downs sufridos** con **downs causados**: nuestro tooltip de equipo
expone los primeros. AxiBridge tambien dispone de `statsTargets[*][0].downed`
para los causados; no es el campo que corresponde a nuestro contador de victimas.

## Detalle de las diferencias

### 1. Eventos de dano y DPS

En [nuestro parser](src/evtc.rs):

- Golpe: `buff=0`, resultado en `0,1,2,5,8,9`, `value>0`.
- Condicion: `buff=1`, resultado `0` o `14..18`, `buff_dmg>0`.
- Sin activacion ni retirada de buff en la rama de dano.
- Actor con equipo reconocido. Con filtro inicial, destino jugador con equipo
  reconocido y distinto. Incluye por ello enfrentamientos entre los otros dos
  colores, no solo ataques contra el equipo del POV.
- El desglose directo/condiciones suma exactamente el dano local.
- Se suman los campos de importe; no hay un ajuste local separado de overkill,
  barrera o dano efectivo.

En [AxiLog][ax-damage], la rama directa de actores exige origen de su roster y
destino en `enemies`; descarta activaciones, retiradas, CC y breakbar, y suma
`max(buff_dmg,0)` si `buff=1`, o `max(value,0)` en otro caso. El credito de
mascotas usa otro recorrido: registro temporal del master y reglas de equipo/
`iff`, sin exigir el mismo conjunto `enemies` para todos sus destinos.

Esto es importante: no basta con copiar una lista de resultados y dejar todo lo
demas igual para afirmar paridad.

Nuestro DPS divide por `duration_ms/1000`, con 0 si no hay duracion. La duracion
se obtiene de eventos `NONE`, incluidos eventos ordinarios sin dano. AxiLog
calcula la duracion del encounter desde `raw.log_start_ms()` hasta el ultimo
evento; divide por `max(duracion/1000,1)` y el adaptador redondea el DPS a entero.
Veanse [modelo][ax-model], [orquestador][ax-analysis] y [adaptador][ax-ei].

Ademas, el [resumen de informes publicados de AxiBridge][ab-report] puede
recalcular DPS de varios combates con dano acumulado / tiempo acumulado de
actividad/participacion. No hay una unica cifra de DPS intercambiable entre
vista de un combate e informe de varios combates.

### 2. Identidades, colores y poblacion

En nuestro addon:

- Una direccion de agente jugador equivale a una fila. Un relog de la misma
  cuenta con otra direccion puede generar otra fila.
- Resumen cuenta todos los jugadores registrados con color; no exige dano ni
  presencia simultanea y no representa poblacion actual del mapa.
- Los totales por color no se limitan al escuadron ni a las filas visibles.
- El color final reconocido se aplica al conjunto de eventos del jugador.

En la [clasificacion WvW nativa][ax-wvw] se agrupan cuentas conocidas y, para
otros casos, instancias. La resolucion de equipo prefiere un identificador real;
entre registros validos conserva el primero, y la afinidad `iff` y la pertenencia
al grupo pueden corregir la particion amigo/enemigo. Hay NPC hostiles y filtros
de participacion para las listas de enemigos. La
[agregacion de jugadores de AxiBridge][ab-aggregation] selecciona miembros con
`!notInSquad` para las estadisticas de escuadron.

Por eso no hay que comparar la suma de nuestros tres colores con el total del
escuadron de AxiBridge, ni exigir que nuestro conteo coincida con su roster.

### 3. Down contribution

Desde v0.2.10 el [modulo local](src/contribution.rs) implementa el algoritmo
nativo descrito a continuacion, sustituyendo la ventana fija de v0.2.9.
Los downs proceden solo de eventos ordinarios con resultado 9, sin inferirlos
desde `CHANGE_DOWN`. Los eventos se indexan por destino y tiempo; salud y downs
se ordenan temporalmente. Cada intervalo se consulta con limites inclusivos.

Algoritmo de [AxiLog 1.16.0][ax-contribution]:

1. Detecta resultados downed dirigidos a la poblacion correspondiente.
2. Busca la ultima actualizacion de salud del objetivo con valor **>=99%**
   hasta ese down.
3. Inicio = `max(ancla - 2000 ms, inicio del log, reset anterior del objetivo)`.
   Sin ancla, usa inicio del log.
4. Final = instante del down, **inclusivo por timestamp**.
5. Para el siguiente down del objetivo, reset = down anterior + **2100 ms**.
6. Resuelve identidad/mascotas temporalmente y rechaza inconsistencias entre
   direccion e instancia antes de atribuir.

No es "los ultimos 2 segundos": esos 2 s preceden al ancla de salud, no al down.
Se admiten los eventos ordinarios sin activacion/retirada, salvo CC (12) y
breakbar (10), con `max(buff_dmg,0)` para buff=1 y `max(value,0)` en otro caso.
Se incluyen todos los eventos admisibles del milisegundo del down, independientemente
de su posicion respecto a una notificacion de estado. Sin ancla se usa el primer
evento del log; salud >200 % se ignora y salud 100-200 % equivale al clamp a 100 %.

La contribucion tiene su propio predicado nativo: no se fuerza a ser subconjunto
del Dano local, cuyo filtro de resultados todavia es distinto. Se comprueban
instancia/direccion de la fuente y master al tiempo del evento, excluyendo
registros de extension y direcciones nulas del registro temporal.

**Limite de la adaptacion:** conservamos filas por direccion y objetivos de otro
color conocido en los tres bandos. AxiBridge agrupa cuentas y trabaja con su
particion escuadron/enemigos. No se promete igualdad de totales entre poblaciones
distintas ni se modifica Dano, equipos o relogs como efecto secundario.
Las pruebas sinteticas cubren umbral, salud invalida/ausente, limites, condiciones,
mascotas/instancias recicladas, reset, eventos del mismo milisegundo y exclusiones.
Comparacion integral de dos ejecutables sobre un EVTC real todavia pendiente.

**Documentacion upstream no coincidente:** la [especificacion de AxiBridge][ab-spec]
describe la variante de Elite Insights basada en salud <=90% y sin mascotas.
Sin embargo, su [lector][ab-combat] prioriza `statsAll[0].downContribution`, y el
[adaptador nativo][ax-ei] coloca ahi el dano a downs de AxiLog, indicando
expresamente que no es el algoritmo de EI. Para logs nativos manda ese codigo;
un informe importado de EI puede usar la otra metodologia.

### 4. Corrups / boon strips

Nuestro nombre "corrups" no significa exclusivamente habilidades que convierten
boons en condiciones: cuenta **retiradas de boons**, sea cual sea la habilidad.
Una retirada completa cuenta 1; no el numero de stacks ni segundos eliminados.

Reglas locales en [el parser](src/evtc.rs):

- Lista de 11 boons. Retirada completa moderna (`statechange=72`) o legacy
  (`is_buffremove=1`).
- Exige importe distinto de cero, removedor no nulo e instancias origen/destino
  distintas.
- Victima en origen, removedor en destino; puede atribuir al master del
  removedor.
- Con filtro inicial, victima jugador de otro color. Resumen suma exactamente
  este mismo contador.

El lector normal de [AxiBridge][ab-dashboard] usa `boonStrips`, con modos count,
duracion o valor ponderado; el valor comparable a nuestra columna es **count**.
En [support de AxiLog][ax-support], ese contador usa retiradas completas de boons
sobre `enemies`, acredita al **removedor real agrupado por cuenta**, no a su
master, y no exige importe no nulo ni nuestro test de instancias distintas.
Hay ademas despacho segun era del formato EVTC.

**No confundir dos familias upstream:** AxiLog tambien calcula
`strips_arcdps`, que excluye la perdida de un solo stack de estabilidad y usa
`iff`. El helper normal de AxiBridge lee `boonStrips`, no esos contadores.
No se debe atribuir automaticamente la exclusion de estabilidad a la columna
normal sin comprobar la ruta seleccionada.

### 5. Kills, muertes y derribos sufridos

Nuestro conteo de kills usa el resultado 8 contra un jugador de otro color,
con credito de mascota conocido. No exige importe positivo. El conteo de
muertes suma ese resultado en la victima, y los downs sufridos usan el resultado
9; ambos con `buff=0`.

[AxiLog][ax-downs] usa killing blow/downed para las acciones causadas contra
`enemies`, con resolucion temporal del master y agrupacion de cuentas. Sus
muertes sufridas provienen de **`CHANGE_DEAD`**, no de un killing blow recibido.
Sus downs sufridos usan el resultado downed recibido; las guardas no son
identicas a las nuestras.

Consecuencia: una muerte con cambio de estado registrado pero sin killing blow
recibido puede contar alli y no aqui. Nuestros downs del tooltip son eventos
sufridos, no enemigos derribados por ese equipo. Tampoco se deduce que el numero
de kills de un color deba igualar el de muertes de otro en un combate a tres
bandos con visibilidad parcial.

### 6. Curas, barrera, total y disponibilidad

En [nuestro modulo de healing](src/healing.rs) y [la UI](src/ui.rs):

- Extension de Healing Stats, firma `0x9c9b3c99`, revisiones 1/2.
- Importes observados de curas y barrera; **no curacion efectiva demostrada**.
- Emisor y receptor, o su propietario conocido, deben pertenecer al escuadron
  del POV en ese momento: cuenta identificable, subgrupo 1-15 y sin color
  conocido distinto al del POV.
- Un receptor mascota puede resolverse a su propietario y contar. Los cambios
  de subgrupo se aplican al recorrer eventos; la lista mostrada usa pertenencia
  final.
- Se prefieren observaciones source-peer por jugador atribuido, instancia
  emisora y tipo cura/barrera; despues se eliminan duplicados de campos exactos.
- Datos de curas/barrera ausentes se muestran como `-`. Si solo se conoce un
  componente del total, tooltip y copia lo identifican como limite inferior
  (`>=`). No se declara cobertura completa.

[AxiLog healing][ax-healing] tambien interpreta importes de la extension, prefiere
source-peer y atribuye mascotas, pero sanitiza por **healer real antes de
atribuir al master**, con identidad temporal. Conserva la identidad del
destinatario, en vez de plegar automaticamente mascotas receptoras al master.

Los helpers [computeSquadHealing / computeSquadBarrier][ab-combat] suman los
arrays por destinatario `outgoingHealingAllies` / `outgoingBarrierAllies`. En el
[adaptador nativo][ax-ei] esos arrays recorren las entradas de `players[]`;
no equivalen al escalar de toda curacion saliente a cualquier destinatario.
Asi, no debemos comparar nuestro total con `outgoingHealing[0].healing` sin
mas, ni asumir que las curas a mascotas aparecen en esa matriz de jugadores.

La disponibilidad es otra diferencia: los helpers devuelven 0 si faltan arrays,
mientras que otras superficies de AxiBridge tienen `hasHealAddon`. Nuestro `-`
no se puede convertir a 0 para una comparacion numerica. El propio parser
upstream documenta discrepancias de sanitizacion respecto a EI; adoptar AxiLog
no implicaria igualdad universal con dps.report.

La aportacion por segundo del tooltip local divide la **metrica seleccionada**
por la duracion del combate: si se ordena por limpias, es limpias/s, no HPS.
El total mostrado es curas + barrera disponibles, no curas + limpias.

### 7. Limpias / condi cleanse

Reglas locales en [healing](src/healing.rs):

- EVTC ordinario; no requiere extension de healing.
- Lista de 14 condiciones. Retirada completa moderna o legacy, activacion 0 e
  importe positivo. No cuentan retiradas SINGLE/MANUAL.
- Una por condicion retirada completamente, no por stack.
- Removedor y receptor deben resolver a miembros del escuadron en ese momento;
  incluye propias y mascotas conocidas en ambos extremos.
- No implementa la exclusion especifica de ceguera autoconsumida ni la retirada
  del credito de la descarga de condiciones al caer down.
- El 0 de limpias para un miembro no certifica que el log haya observado todas
  sus retiradas; a diferencia de curas/barrera, no tiene cobertura individual.

El predeterminado arcdps del [helper de AxiBridge][ab-dashboard] suma:

`condiCleanseArcdps + condiCleanseArcdpsOnMinion + condiCleanseArcdpsByMinion`

Los calcula [arcdps_parity de AxiLog][ax-arcdps], por separado de los contadores
tipo EI. Usa afinidad `iff` para limpias a otros, resuelve mascotas temporalmente
y puede incluir amistosos ajenos al escuadron como destinatarios. Excluye
ceguera autoconsumida y revierte la cadena de autorretiradas asociada al down
con una ventana acotada de **100 ms** y reglas de recorrido. No es nuestro
simple conteo de retiradas de importe positivo.

Hay varias superficies, no un unico "total de limpias":

- Tipo EI: `condiCleanse` sobre otros jugadores; con self se suma
  `condiCleanseSelf`.
- Compatibilidad antigua sin nuevos contadores: AxiBridge suma lo anterior y,
  si existe, `condiCleanseMinions`. Es una aproximacion, no el algoritmo nativo
  arcdps.
- Helper arcdps actual: los tres nuevos contadores de la formula anterior.
- El [resumen de informes publicados][ab-report] aun suma los campos
  `condiCleanse + condiCleanseSelf + condiCleanseMinions`. No se debe usar esa
  ruta como prueba de que el helper actual usa la misma metodologia.

**Conclusion:** nuestra cifra no es equivalente garantizada ni a "Squad", ni a
"All" tipo EI, ni al predeterminado arcdps de AxiBridge.

### 8. Orden, filtros y datos auxiliares

[Destacados](src/danger.rs) ordena inicialmente por:

`score = dano + 2000 * corrups + down_contribution`

Es una puntuacion propia, no dano adicional real. El dano admitido por ambas
metricas recibe peso dos veces; desde v0.2.10 sus predicados no son identicos.
Mantenerla o cambiarla es una decision de producto separada de alinear metricas.

- Solo muestra jugadores del color con dano >0 o corrups >0.
- Ordenar por DPS equivale a ordenar por dano dentro de un combate, al compartir
  denominador. Los empates mantienen el orden de indices.
- Scroll, cache y clipping no cambian importes. Copia/chat de Destacados usa
  las mismas filas y el orden seleccionado, no un nuevo calculo.
- Healing ordena por la metrica elegida, inicialmente curas + barrera, y muestra
  hasta el limite configurado 4-10. Puede mostrar jugadores con solo limpias;
  las metricas ausentes van al final. No es el mismo filtro que Destacados.
- Nombres, rango, cuenta, profesion/especializacion, color y subgrupo son
  metadatos de identidad, no metricas de rendimiento. Las reglas de agrupacion
  y subgrupo explicadas arriba afectan su comparabilidad.
- Las armas y el uso subacuatico son inferencias locales a partir de habilidades
  y cambios de conjunto, no equipamiento confirmado. No se ha certificado
  equivalencia con la inferencia de armas de AxiBridge.
- Hora del historial, idioma y tiempo de parseo son metadatos del log o del
  procesamiento local; no se proponen como metricas de combate equivalentes.
- Las cifras abreviadas de pantalla no sirven para demostrar paridad: comparar
  importes completos, no textos redondeados o recortados.

## Estado de adaptacion y pasos pendientes

Aplicado en v0.2.10: algoritmo de Down contribution y eliminacion del ajuste fijo.
Las demas metricas permanecen pendientes de decision y verificacion.

1. **Fijar el contrato:** AxiBridge nativo con AxiLog 1.16.0 como referencia
   inicial, distinguiendo informes importados. Elegir explicitamente si Dano
   seguira siendo solo a jugadores o pasara al alcance de `dpsAll`.
2. **Identidad compartida:** registro temporal de instancias, propietarios,
   relogs y poblaciones. Evitar arreglar cada metrica con una resolucion de
   mascotas diferente.
3. **Dano y duracion/DPS:** igualar eventos y alcance elegidos; comprobar
   desglose y suma por equipo. No adaptar solo la formula de division.
4. **Down contribution, aplicado:** ancla >=99%, lead-in, reset y limites exactos.
   Ajuste fijo eliminado por peticion expresa; sus claves antiguas se ignoran
   y desaparecen al guardar, conservando las demas preferencias.
5. **Corrups y limpias:** seleccionar `boonStrips` count y cleanse arcdps, o
   identificar expresamente otra variante. No mezclar boonStrips de tipo EI con
   los contadores `strips_arcdps` creyendo que son lo mismo.
6. **Muertes/downs/kills:** alinear fuentes y diferenciar sufridos de causados.
7. **Healing:** igualar destinatarios, sanitizacion y atribucion sin perder
   nuestra distincion entre ausencia y cero ni presentar curacion efectiva no
   demostrada.
8. **Conservar UX propia salvo decision expresa:** tres colores, puntuacion
   ponderada, scroll y filtros de presentacion no necesitan copiar la UI de
   AxiBridge para que sus metricas base sean comparables.

## Verificacion numerica necesaria antes de afirmar paridad

Pendiente; no realizada en esta revision:

1. Procesar **el mismo archivo** con ambas versiones fijadas, sin remitir logs
   privados a servicios externos. Registrar hash del EVTC, opciones, version
   del formato y revision de extension.
2. Exportar enteros completos por identidad canonica y scope: dano, desglose,
   contribucion, strips, kills, downs, muertes, curas, barrera y limpias, junto
   con numerador/denominador de cada tasa y disponibilidad.
3. Comparar primero jugadores/objetivos comunes; despues explicar diferencias
   de poblacion, separando jugadores, mascotas, NPC y gadgets. No compensar
   errores opuestos con un total global parecido.
4. Exigir **diferencia 0** en contadores enteros del contrato elegido. Para DPS/
   HPS, igual numerador y duracion antes de aplicar el redondeo acordado.
   Distinguir diferencias intencionales de defectos sin ocultarlas tras
   tolerancias arbitrarias.
5. Cubrir logs antiguos y modernos, solo jugadores y NPC hostiles, combate a
   tres bandos, relogs/instancias recicladas, fuentes mascota, receptores
   mascota, equipos desconocidos/cambiantes y cambios de subgrupo.
6. Casos de contribucion: sin salud, salud exactamente 99%, limites del
   anticipo, mismo milisegundo, down solo por estado, rallies y dos downs
   separados menos de 2100 ms.
7. Casos de soporte: retirada de importe 0, stacks de estabilidad, ceguera
   propia, descarga al down, self/pet-self, destinatario amistoso fuera del
   escuadron, extension ausente/incompatible y observaciones de varios peers.
8. Verificar tambien suma por color, tooltips, orden y copia, conservando
   ausente/0/limite inferior. Solo entonces publicar una tabla de diferencias
   medidas y declarar la paridad que realmente se haya demostrado.

## Fuentes consultadas

Los enlaces externos siguientes fijan el commit, no una rama movil. Las
referencias de codigo prevalecen sobre comentarios/documentacion desactualizados.
No se han importado dependencias ni copiado implementaciones upstream.

- Local: [parser y estructuras](src/evtc.rs), [healing y limpias](src/healing.rs),
  [ranking](src/danger.rs), [ventanas/tooltips/copia](src/ui.rs),
  [valores iniciales](settings.default.json).
- AxiBridge: [dependencia][ab-package], [lectores de metricas][ab-dashboard],
  [contribucion/healing][ab-combat], [agregacion][ab-aggregation],
  [resumen de informes][ab-report], [especificacion][ab-spec].
- AxiLog: [dano/instancias][ax-damage], [orquestador][ax-analysis],
  [encounter/duracion][ax-model], [equipos/roster][ax-wvw],
  [down contribution][ax-contribution], [kills/downs/muertes][ax-downs],
  [support][ax-support], [contadores arcdps][ax-arcdps],
  [healing][ax-healing], [adaptador a campos EI][ax-ei].

[ab-package]: https://github.com/darkharasho/axibridge/blob/44d58c555ffb3c4d9adae3522f8472985b9fc51c/package.json
[ab-dashboard]: https://github.com/darkharasho/axibridge/blob/44d58c555ffb3c4d9adae3522f8472985b9fc51c/packages/bridge-metrics/src/dashboardMetrics.ts#L6
[ab-combat]: https://github.com/darkharasho/axibridge/blob/44d58c555ffb3c4d9adae3522f8472985b9fc51c/packages/bridge-metrics/src/combatMetrics.ts#L99
[ab-aggregation]: https://github.com/darkharasho/axibridge/blob/44d58c555ffb3c4d9adae3522f8472985b9fc51c/packages/bridge-metrics/src/computePlayerAggregation.ts#L604
[ab-report]: https://github.com/darkharasho/axibridge/blob/44d58c555ffb3c4d9adae3522f8472985b9fc51c/packages/bridge-metrics/src/reportMetrics.ts#L109
[ab-spec]: https://github.com/darkharasho/axibridge/blob/44d58c555ffb3c4d9adae3522f8472985b9fc51c/docs/metrics-spec.md
[ax-damage]: https://github.com/darkharasho/axilog/blob/9bd82979dbb49a000b7af3b15713d956f011bb00/crates/axilog-core/src/analysis/damage.rs#L219
[ax-analysis]: https://github.com/darkharasho/axilog/blob/9bd82979dbb49a000b7af3b15713d956f011bb00/crates/axilog-core/src/analysis/mod.rs#L470
[ax-model]: https://github.com/darkharasho/axilog/blob/9bd82979dbb49a000b7af3b15713d956f011bb00/crates/axilog-core/src/model/mod.rs#L466
[ax-wvw]: https://github.com/darkharasho/axilog/blob/9bd82979dbb49a000b7af3b15713d956f011bb00/crates/axilog-core/src/wvw/mod.rs#L630
[ax-contribution]: https://github.com/darkharasho/axilog/blob/9bd82979dbb49a000b7af3b15713d956f011bb00/crates/axilog-core/src/analysis/contribution.rs#L246
[ax-downs]: https://github.com/darkharasho/axilog/blob/9bd82979dbb49a000b7af3b15713d956f011bb00/crates/axilog-core/src/analysis/downs.rs#L63
[ax-support]: https://github.com/darkharasho/axilog/blob/9bd82979dbb49a000b7af3b15713d956f011bb00/crates/axilog-core/src/analysis/support.rs#L641
[ax-arcdps]: https://github.com/darkharasho/axilog/blob/9bd82979dbb49a000b7af3b15713d956f011bb00/crates/axilog-core/src/analysis/arcdps_parity.rs#L170
[ax-healing]: https://github.com/darkharasho/axilog/blob/9bd82979dbb49a000b7af3b15713d956f011bb00/crates/axilog-core/src/analysis/healing.rs#L426
[ax-ei]: https://github.com/darkharasho/axilog/blob/9bd82979dbb49a000b7af3b15713d956f011bb00/crates/axilog-ei/src/lib.rs#L1200
