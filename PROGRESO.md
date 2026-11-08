# Progreso: fight_analysis_danger

## Unificacion del repositorio publico (v0.2.29)
- Desde v0.2.21 el desarrollo se estaba subiendo solo al repositorio privado `Fight-Analysis-Danger` y este repositorio publico se quedo en v0.2.20.
- Este commit unifica el estado actual (v0.2.29) sobre la historia publica existente, conservando `LICENSE`, CI, plantillas de issues, `CONTRIBUTING`, `SECURITY`, `CHANGELOG` y los avisos de terceros que solo existian aqui.
- A partir de ahora el desarrollo y las publicaciones se hacen desde este repositorio; `Fight-Analysis-Danger` queda como copia privada de respaldo y `gw2addonsreleases` sigue siendo el repo de distribucion de la DLL.
- `tools/publish.ps1` fija la identidad de los commits con el email noreply de GitHub (`-GitName`/`-GitEmail` para cambiarla), avisa si no es noreply y muestra el repositorio de origen antes de empujar, para no publicar por error en otro sitio.

## v0.2.29: fondo del color de clase en Destacados y Healing
- Nueva opcion `class_colors` (activada por defecto) en **Estilo** del menu contextual de Destacados y Healing, tambien en ventanas adicionales. Activa, cada fila pinta un fondo casi opaco con el color de la profesion, como arcdps, y el nombre y las cifras pasan a blanco o casi negro segun el contraste (luminancia > 0.6 usa texto oscuro).
- Desactivada, las filas vuelven al aspecto anterior: sin fondo, nombre en el color de clase y cifras en blanco. El ajuste es comun, no por ventana, y se guarda como `class_colors=1/0`; los archivos anteriores sin la clave quedan con el fondo activado.
- El resumen por colores no cambia: sus filas son de equipo, no de jugador.
- Validacion: `cargo fmt --check`, 103 tests Rust y 2 de configuracion; `cargo build --release`. DLL MZ de 1.125.888 bytes, MD5 `24136aa1df8f1ba8f63804df7f35d4f4`. `GetAddonDef` confirma v0.2.29, firma -1178682436, proveedor Direct y URL exacta.
- Publicado con `tools/publish.ps1`: fuente `b4261e4`/`9a305f8`, distribucion `81d18ed`, release `fight-analysis-danger-v0.2.29` con DLL y MD5 y `--latest=false`. Quedan solo las releases 0.2.29 y 0.2.28; Latest de Timers v0.3.35 intacto. URL directa, antigua, commit inmutable, asset de release y los dos sidecars coinciden en tamaño, MD5 y `MZ`.
- Pendiente la prueba visual dentro de GW2 y la copia manual de la DLL al juego.

## Herramientas de publicacion (sin cambio de version)
- `tools/publish.ps1` encadena todos los pasos: bump opcional, `cargo fmt --check`, tests y build release, `GetAddonDef` de la DLL real, copia de DLL y sidecar MD5 al repo de distribucion, comprobacion de `.gitattributes` y de `check-attr`, blob preparado igual a la DLL, commits y push en ambos repos, release con `--latest=false`, dejar solo las dos releases de este addon y verificacion de las cuatro URL y los dos sidecars.
- `tools/verify-release.ps1` repite solo la verificacion (DLL local con `GetAddonDef`, cuatro URL, tamaño, MD5, `MZ` y sidecars de 32 bytes) sin compilar ni publicar.
- Primer uso real en v0.2.29: se corrigio el envio de assets a `gh release create` (rutas absolutas, antes fallaba en silencio), la comprobacion de codigos de salida de `gh`, los commits sin cambios y el reintento de assets recien creados.
- `elitei/` guarda las fuentes de referencia de Elite Insights usadas para fijar las reglas de curas, dano y soporte, y `elitei/fetch-ei.ps1` las vuelve a descargar de `master`.
- Documentacion actualizada con los cambios de calculo de v0.2.28: `calculos.md` (secciones 1, 3, 6, inventario y pendientes), `WALKTHROUGH.md`, `README.md` y `readme.html`.
- Limpieza aplicada: en `gw2addonsreleases` quedan solo las releases `fight-analysis-danger-v0.2.28` y `v0.2.27`; se elimino la de v0.2.26 conservando su tag. Latest de Timers v0.3.35 intacto. Las URLs no cambian.

## v0.2.28: curas como Elite Insights y sin dano a barrera
- Curas y barrera siguen la regla de EI: se cuenta todo evento de la extension cuyo byte IFF marque objetivo aliado (`IFF.Friend`), sin exigir escuadron, subgrupo ni POV identificable. Antes se perdian las curas a aliados fuera del escuadron y, sin grupo identificable, no se mostraba nada.
- Se elimina `HealingCoverage::UnknownGroup` y su aviso: ya no hay datos de curas que dependan de conocer el grupo. Se conservan `Missing`, `Unsupported` y `Partial`.
- El dano absorbido por barrera (`is_shields`) deja de sumar en dano, dano directo, condiciones y Down contribution, como `HealthDamage` de EI. Afecta a logs y al directo, porque ambos pasan por el mismo analisis.
- Validacion: `cargo fmt --check`, 101 tests Rust y 2 de configuracion en debug y release; `cargo build --release`. DLL MZ de 1.122.816 bytes, MD5 `91f486ec5a4d92276e75f901b97ca959`. `GetAddonDef` confirma v0.2.28, firma -1178682436, proveedor Direct y URL exacta.
- Referencias de EI guardadas en `elitei/` (fuentes `.cs` de master y `fetch-ei.ps1` para volver a bajarlas).
- Fuente `db19b29` y distribucion `f6df9ad` publicadas; release `fight-analysis-danger-v0.2.28` con DLL y MD5, `--latest=false`. Blob preparado sin LFS e identico a la DLL compilada (1.122.816 bytes); sidecar de 32 bytes exactos. URL directa, antigua, commit inmutable y asset de release coinciden en `MZ`, tamano y MD5.
- Pendiente: contraste numerico con un log real al lado de Elite Insights; no se probo en GW2 ni se copio la DLL al juego.

## v0.2.27: el log sustituye al provisional aunque no lo contenga
- El log real solo sustituia al combate provisional si su intervalo de eventos contenia el del directo. El directo empieza en la primera interaccion PvP y puede seguir recibiendo eventos despues de que arcdps cierre el log, asi que el intervalo no quedaba dentro y el mismo combate aparecia dos veces en el historial (`LIVE-n` y el log).
- Ahora basta con que los intervalos se solapen, que es lo que ocurre siempre en el mismo combate; dos combates distintos siguen sin solaparse y se conservan. Afecta tambien a una seleccion fijada en el provisional, que pasa al log.
- Validacion: `cargo fmt --check`, 100 tests Rust y 2 de configuracion en debug y release; `cargo build --release`. DLL MZ de 1.123.328 bytes, MD5 `581300ee0acfd2087696af7acdedc347`. `GetAddonDef` confirma v0.2.27, firma -1178682436, proveedor Direct y URL exacta.
- Pendiente la prueba en GW2: el caso concreto era un combate de 1 min 52 s en el log y 1 min 10 s en el directo, empezando este ultimo 42 s despues.
- Fuente `951884b` y distribucion `72528c4` publicadas; release `fight-analysis-danger-v0.2.27` con DLL y MD5, `--latest=false`. Latest de Timers v0.3.35 intacto. Blob preparado sin LFS e identico a la DLL compilada (1.123.328 bytes); sidecar de 32 bytes exactos.
- La URL directa, la antigua, la del commit `72528c4` y el asset de release coinciden en `MZ`, tamano y MD5. El sidecar raw de `main` sirvio inicialmente el MD5 de v0.2.26 por la cache (`max-age=300`) y paso al nuevo al expirar; release y raw coinciden ahora en 32 bytes exactos. Cliente/parser de Nexus y prueba dentro de GW2 pendientes; no se copio la DLL al juego.

## v0.2.26: cierre del directo por inactividad
- El directo reutiliza `[session] wvwinactive` de arcdps; usa 15 segundos si falta un valor valido y acepta 0 como cierre inmediato. Cada interaccion PvP entre jugadores reinicia el plazo; el dano a NPC no.
- `EXIT_COMBAT` ya no cierra el informe al instante: se conserva hasta que vence la inactividad y entonces pasa al historial como combate provisional finalizado.
- Validacion: `cargo fmt --check`, 99 tests Rust y 2 de configuracion; `cargo build --release`. DLL MZ de 1.123.328 bytes, MD5 `fa1b5cd185dee2bb4445ab4247ee100a`. `GetAddonDef` confirma v0.2.26, firma -1178682436, proveedor Direct y URL exacta.
- Distribucion publicada en `gw2addonsreleases`, commit `59b5d57`, release `fight-analysis-danger-v0.2.26`; quedan solo las releases v0.2.25 y v0.2.26, y Latest de Timers sigue intacto. Blob real sin LFS, mismo tamano que la DLL; sidecar de 32 bytes exactos.
- Asset de release y DLL de URLs directa y antigua coinciden en `MZ`, tamano y MD5. Los sidecars de release y URL raw coinciden con el MD5; el raw de `main` devolvio inicialmente el valor cacheado anterior y se actualizo al expirar el `max-age=300`. Cliente/parser de Nexus y prueba dentro de GW2 pendientes; no se copio la DLL al juego.

## v0.2.25: kills, muertes y derribos alineados con Elite Insights
- Verificado con Elite Insights v3.31 en logs reales: las kills del equipo ya coinciden con su columna `Killed`; su ~20 era `Dead`/`Downs`.
- Muertes y derribos sufridos pasan a contar los cambios de estado `CHANGE_DEAD` y `CHANGE_DOWN`, con resolucion por instid para agentes fuera de rango. Antes se perdian muertes sin killing blow y aliados fuera de escuadron.
- Resultado en `20261107-184621`: Verde 10 kills / 9 muertes / 20 derribos = Elite Insights 10 / 9 / 20. En `20261107-185345`: 19 / 2 / 18 identicos.
- El directo tambien usa estos cambios de estado (arcdps los limita al escuadron en realtime).
- Formato, 97 tests del addon y 2 de configuracion correctos; DLL release de 1.121.792 bytes y MD5 `5e9a5c906aa77da9b45518865a8e41e4`.
- GetAddonDef de la DLL real confirma v0.2.25, firma -1178682436, proveedor Direct y URL exacta.
- Fuente `5a7eadf` (mas version en `Cargo.toml`) y distribucion `d2b3084` publicadas; release `fight-analysis-danger-v0.2.25` con DLL y MD5, `--latest=false`. Quedan solo las releases 0.2.25 y 0.2.24 de este addon; el tag huerfano 0.2.23 se conserva y Latest de Timers v0.3.35 permanece intacto.
- Blob preparado identico a la DLL compilada y sin LFS (1.121.792 bytes). URLs directa exacta, antigua, inmutable y asset de release coinciden en MZ, tamano y MD5; los sidecars tienen 32 bytes ASCII en minusculas sin salto de linea. La comprobacion con el cliente HTTP y parser MD5 de Nexus queda pendiente.
- Pendiente la prueba de actualizacion dentro de GW2; no se ha copiado la DLL al juego.

## v0.2.24: compilacion mas compacta
- Se configura `codegen-units=1` en release, manteniendo `opt-level=2`, `lto=true` en el manifiesto y las funcionalidades existentes.
- Formato, 97 tests del addon y 2 de configuracion correctos tanto en debug como en release. DLL real: 1.121.280 bytes (-2,23 %), MD5 `18c38de003fcbe14f6d6bd4446acd3c6`; GetAddonDef confirma v0.2.24, firma, Direct y URL.
- Cargo no pasa LTO global al compilar juntos `cdylib` y `rlib`; ocurria tambien con las 16 unidades anteriores. Se conserva la configuracion existente, sin cambiar los tipos de biblioteca.
- Fuente `f3e4296`, distribucion `50191d0` y release `fight-analysis-danger-v0.2.24` publicadas. Se conservan las releases y tags 0.2.24 y 0.2.23 de este addon; se retiro solo la 0.2.22 y Latest de Timers v0.3.33 permanece intacto.
- Blob preparado identico a la DLL compilada y sin LFS. URLs directa exacta, antigua, inmutable y asset de release coinciden en MZ, tamano y MD5; sus sidecars tienen 32 bytes exactos y coinciden. La release 0.2.23 conserva su DLL y MD5 validos.
- Cliente HTTP y parser MD5 originales de Nexus (checkout `f99003c`), probados fuera del juego: URLs directa y antigua entregan la DLL y el MD5 nuevos correctamente.
- Pendiente la prueba de actualizacion dentro de GW2; no se ha copiado la DLL al juego.

## Analisis de tamano de DLL v0.2.23
- La DLL local y la publicada coinciden: 1.146.880 bytes (1,09 MiB). La seccion de codigo ocupa 775.168 bytes; hay 34.792 bytes de iconos de clase/especializacion y ningun PNG de armas incluido.
- `strip=symbols` no reduce el tamano. Con `codegen-units=1`, sin cambiar `opt-level=2` ni `lto=true` en el manifiesto, la DLL experimental ocupa 1.121.280 bytes (-2,23 %); en tres logs la mediana de analisis paso de 274,5/225,7/183,6 a 255,3/199,5/161,1 ms.
- `opt-level=s` y una unidad baja a 1.011.200 bytes (-11,83 %), con un log un 4,7 % mas lento y los otros dos alrededor de un 1 % mas rapidos. `opt-level=z` baja a 954.880 bytes (-16,74 %), pero los tres logs tardan entre un 14 % y un 18 % mas.
- Las tres variantes pasaron 97 tests del addon y 2 de configuracion; conservaron GetAddonDef e iconos y produjeron informes identicos en tres logs (7 mediciones por variante y log). No se probaron la UI ni el directo en GW2.
- Se adopta `codegen-units=1` en v0.2.24 por su menor riesgo de rendimiento; se conservan `opt-level=2` y la configuracion `lto=true`.

## v0.2.23: rendimiento y ajustes
- Formato, 97 tests del addon y 2 de configuracion correctos; DLL release de 1.146.880 bytes y MD5 `6defa7a651c6dcc9ab05117072cf4268`.
- GetAddonDef de la DLL real confirma v0.2.23, firma -1178682436, proveedor Direct y URL exacta.
- Fuente `d172e8f`, distribucion `75f8941` y release `fight-analysis-danger-v0.2.23` publicadas; en esa publicacion se conservaron las releases y tags 0.2.23 y 0.2.22, con Latest de Timers v0.3.33 intacto.
- Blob preparado identico a la DLL compilada y sin LFS. URLs directa exacta, antigua, inmutable y asset de release coinciden en MZ, tamano y MD5; sus sidecars tienen 32 bytes exactos y coinciden. La release anterior conserva su DLL y MD5 validos.
- Cliente HTTP y parser MD5 originales de Nexus (checkout `f99003c`), probados fuera del juego: URLs directa y antigua entregan la DLL y el MD5 nuevos correctamente.
- Pendiente la prueba del directo y de la actualizacion dentro de GW2; no se ha copiado la DLL al juego.
- `inflate.rs`: tabla Huffman directa de 10 bits con peek y copia por bloques; DEFLATE ~2,5x mas rapido (log de 61 MB: 481 -> 191 ms).
- `evtc.rs`/`contribution.rs`/`healing.rs`: hasher multiplicativo `FastMap` para direcciones/instancias del log local; parse ~2x (61 MB: 245 -> ~90-125 ms). Resultados por jugador identicos (hash comparado en 3 logs reales).
- `live.rs`: snapshots incrementales (solo eventos nuevos, el hilo de analisis guarda el buffer por generacion), sin bloquear el estado de la UI con LIVE tomado y sin clonar el agente por evento si no hay combate activo. Test nuevo del delta.
- `ui.rs`: sin clonar `Settings` por ventana y frame; tooltips perezosos; Healing reutiliza textos y omite filas fuera de vista; `clipped_name` sin `format!` por caracter.
- Decisiones aplicadas: se elimina `summary.columns` de los ajustes guardados (y se ignora el valor antiguo); los anchos predeterminados antiguos migran a 259/300/234 px; README ya no etiqueta la estructura comun con una version obsoleta; Healing distingue limpias EVTC de curas/barrera Healing Stats y aclara que el valor por segundo es la metrica de orden actual.

## Guia funcional del codigo fuente
- Creado `WALKTHROUGH.md` en espanol: estructura de carpetas, flujo desde `GetAddonDef` y `lifecycle::load`, ruta de logs/EVTC, explicacion de strips y ficha funcional de cada archivo fuente y script.
- Enlaces agregados a README.md y readme.html. Documentacion revisada; no requiere compilacion ni tests.

## v0.2.22: seguimiento automatico al cargar
- Se adelanta la activacion al ciclo de carga del addon, antes de iniciar el hilo de analisis; no depende de que la UI renderice primero en un mapa WvW.
- Se conserva el filtro de dano/strips contra jugadores, que no forma parte de este cambio.
- Validacion local: formato, 95 tests Rust y 2 de configuracion, build release y GetAddonDef real correctos (version, firma, Direct y URL).
- Fuente `7bba1ec`, distribucion `8a813a4` y release `fight-analysis-danger-v0.2.22` publicadas; Latest de Timers conservado.
- DLL MZ de 1.147.392 bytes, MD5 `bf55fff330a5a00921eb1d7b393d00d3`; blob real sin LFS. URL directa, URL antigua de DLL y asset de release coinciden en tamano/hash; sidecars raw directo y release son 32 bytes y coinciden.
- Aviso de cache: la URL antigua de GitHub para `fight_analysis_danger.dll.md5` todavia sirvio el MD5 de v0.2.21 (`8b1d54b0c3942b8ae245f2ca6fe13e28`), aunque su DLL entrega v0.2.22. No considerar validada la actualizacion por esa ruta antigua hasta que el sidecar tambien coincida.
- Pendiente prueba en GW2 y con el cliente/parser de Nexus; no copiar la DLL al juego.

## v0.2.21: seguimiento automatico al iniciar
- Al cargar el addon, antes de arrancar el hilo de analisis, fuerza y guarda `follow_latest=true` en las tres ventanas, incluso si estaba desactivado; el usuario puede apagarlo durante la sesion y se vuelve a activar al siguiente inicio.
- Nuevas instalaciones tambien lo reciben activado por defecto. El filtro de dano/strips contra jugadores no cambia.
- Validacion: 95 tests Rust y 2 de configuracion, formato y build release correctos; GetAddonDef de la DLL real confirma v0.2.21, firma, Direct y URL conservados.
- Fuente `1b5f639` y distribucion `6db2189` publicadas; release `fight-analysis-danger-v0.2.21`, Latest de Timers conservado.
- DLL MZ de 1.146.368 bytes, MD5 `8b1d54b0c3942b8ae245f2ca6fe13e28`; blob real sin LFS. Endpoint directo, URL antigua y assets de release, junto a sus tres sidecars de 32 bytes, coinciden en tamano/hash.
- Pendiente prueba en el proceso de GW2 y con el cliente/parser propio de Nexus; no se pudo verificar la autoactualizacion dentro del juego.

## v0.2.20: prueba de autoactualizacion desde v0.2.19
- Solo incremento de version, sin cambios funcionales, de endpoint, identidad ni preferencias.
- No se copia al juego: se conserva la instalacion existente para comprobar la actualizacion mediante Nexus.
- Formato, 94 tests unitarios y 2 de configuracion correctos en ambos checkouts; licencias correctas, DLL release compilada y GetAddonDef real v0.2.20 con firma/Direct/URL conservados.
- Fuente publica `2909959`, distribucion `3734a37`, release `fight-analysis-danger-v0.2.20` con DLL/MD5/SHA256, fuente y licencias. Latest Timers v0.3.26 conservado.
- DLL de 1.146.368 bytes, MD5 `e286fcd8bb708a71f785183143a6f408`, distinto del MD5 de v0.2.19 `a1119ddd011afac0b402c829165cfec3`.
- Atributos sin LFS y blob preparado identico; endpoint exacto, antiguo, inmutable y asset verificados en MZ/tamano/MD5. Cuatro sidecars de exactamente 32 bytes sin BOM/salto y demas assets comprobados.
- Cliente HTTP/parser originales de Nexus descargan la DLL nueva y reconocen el MD5 diferente de v0.2.19; pendiente que el usuario pulse Buscar actualizaciones en Nexus para comprobar el flujo dentro de GW2.

## v0.2.19: endpoint Direct compatible con Nexus
- update_link cambiado al host directo raw.githubusercontent.com; conservados proveedor Direct, firma, identidad, rama, DLL, rutas de datos y ajustes.
- Regresion del endpoint y proveedor generados; reglas de publicacion persistentes actualizadas en AGENTS.md y nexusbase/README.md.
- Las DLL instaladas conservan su URL antigua y pueden necesitar una sustitucion manual autorizada con GW2 cerrado; no se copia al juego.
- Formato, 94 tests unitarios y 2 de configuracion correctos; release compilada en ambos checkouts. Self-check/inventario de licencias correctos; CI publico Windows correcto: https://github.com/cbart-cmd/WvW-Combat-Insights/actions/runs/40102879911.
- GetAddonDef de la DLL real: v0.2.19 (revision interna -1 del SDK), firma -1178682436, nombre WvW Combat Insights, proveedor Direct (3), URL exacta directa y callbacks validos.
- Fuente publica `8053e01`, tag `v0.2.19`; distribucion `d770046`, release `fight-analysis-danger-v0.2.19` con DLL/MD5/SHA256, fuente y licencias. Latest Timers v0.3.26 conservado.
- DLL local: 1.146.368 bytes, MD5 `a1119ddd011afac0b402c829165cfec3`, SHA-256 `507c0668762afeca0a355cb270324f491e504acf38cc9cd2b416474bbfc6a5e9`.
- Los cuatro atributos LFS estan unset; blob preparado identico a la DLL local por hash Git y tamano. Sidecar exactamente 32 bytes ASCII sin BOM/salto.
- URLs exacta nueva, antigua, inmutable y asset coinciden en MZ/tamano/MD5; sus cuatro sidecars son exactos. Fuente, licencias y SHA-256 de release descargados tambien coinciden, sin cache antigua en la comprobacion.
- Reutilizados httplib 0.14.0, OpenSSL y MD5.cpp originales del checkout Nexus `f99003c` (2026-02-17): URL directa descarga DLL de 1.146.368 bytes con MZ y MD5 identico, parser devuelve los 16 bytes esperados. URL antigua reproduce `Failed to read connection` y devuelve checksum vacio con ese mismo cliente, aunque HttpClient verifica que sigue sirviendo los archivos correctos.
- Reglas guardadas sin incorporar los cambios previos de instrucciones/workspace al commit. No se instala en el juego ni se modifican ajustes; pendiente solo el flujo dentro del proceso real de GW2.

## v0.2.18: inicio PvP de equipo e historial provisional
- Directo activado por dano PvP propio o del equipo/escuadron mientras el POV esta en combate, tambien para quien solo cura; NPCs y peleas ajenas fuera de combate no bastan.
- Primer refresco sin umbral de autoapertura de logs; salida detectada tambien por Mumble si no llega EXIT_COMBAT.
- Informe finalizado conservado en el historial de sesion, seleccionable sin archivo; su log real lo sustituye por intervalo de eventos, incluso en seleccion fijada. Sin escritura de combates en disco.
- Equipo propio inicial al cambiar de informe; color manual conservado en refrescos. Migracion unica del antiguo primer enemigo a propio, sin cambiar otros equipos explicitos.
- Conservacion tambien si otro combate o cambio de mapa llega antes del siguiente refresco; cola limitada al maximo de historial.
- Validacion: formato, 94 tests unitarios y 2 de configuracion; compilaciones release independientes en ambos checkouts. Self-check de licencias e inventario correctos; CI publico Windows correcto: https://github.com/cbart-cmd/WvW-Combat-Insights/actions/runs/40102071045.
- Fuente publica autorizada y publicada en `69203d8`, tag `v0.2.18`; distribucion `91faf9e` y release `fight-analysis-danger-v0.2.18` con DLL/MD5/SHA256, fuente y licencias. Latest Timers v0.3.25 conservado.
- DLL release local: 1.146.368 bytes, MD5 `7a04b553cdd5208812639948aa8e207d`, SHA-256 `08c3fd0a1a052d8e4d5df213231b719634de4a4a8b93ce243e0b9c748d0a1059`.
- Atributos sin LFS y blob preparado real verificados; URLs exacta Nexus, antigua, inmutable y asset coinciden en MZ/tamano/MD5. Los cuatro sidecars, SHA-256 de release, fuente y licencias descargadas coinciden.
- Pendiente prueba dentro de GW2: los eventos de curas/barrera siguen dependiendo del bridge y Healing Stats. Sin tocar ajustes personales ni cambios previos de instrucciones/workspace.

## Restauracion de release v0.2.17 (2026-10-24)
- Recreada la release borrada accidentalmente: https://github.com/cbart-cmd/gw2addonsreleases/releases/tag/fight-analysis-danger-v0.2.17.
- Recuperadas DLL original, MD5 y fuente correspondiente con licencias; sin recompilar, cambiar version, etiqueta existente, endpoint ni ajustes.
- Descargas de Nexus actual/antigua, commit inmutable y asset verificadas: MZ, 1.142.272 bytes y MD5 `25cae2cf41953fc492e8b5e0f863e1b4`; los tres sidecars comprobados coinciden.
- ZIP fuente descargado identico al original por SHA-256; atributos DLL sin LFS y blob del indice del mismo tamano. Latest compartido conservado en Timers v0.3.23.
- Sin cambios funcionales ni nuevas pruebas en GW2; cambios previos de AGENTS.md/workspace conservados.

## Publicacion independiente del codigo v0.2.17
- Nuevo repositorio publico: https://github.com/cbart-cmd/WvW-Combat-Insights; este repositorio conserva su historial y sigue privado.
- Copia local independiente en `..\WvW-Combat-Insights`, con un unico commit inicial `f4dfc61` y etiqueta `v0.2.17`; sin importar el historial ni el workspace de otros proyectos.
- Codigo propio bajo GPL-3.0-or-later, con permiso limitado de integracion GW2/Nexus/arcdps; recursos de ArenaNet y dependencias con sus condiciones originales.
- Instrucciones para agentes incluidas; documentacion publica, changelog, soporte y avisos/licencias de terceros preparados. No se copio este historial interno.
- CI Windows de solo lectura correcto: formato, 92 tests Rust, self-check de avisos, inventario de licencias, release y empaquetado de fuente/licencias/checksums.
- Gitleaks no detecto secretos en el unico commit publico; habilitados reportes privados, deteccion/proteccion de secretos y alertas de dependencias. Main exige build y bloquea force-push/borrado para usuarios sujetos a la proteccion; administradores pueden omitirla.
- Release existente ampliada con fuente correspondiente y enlace al repositorio publico. DLL y MD5 conservados, sin cambiar version ni endpoint; SHA-256 documentado. Latest compartido de Timers conservado.
- Codigo funcional, assets y generadores originales identicos; DLL release compilada en la copia nueva. No se modificaron ajustes personales ni cambios previos de AGENTS.md/workspace.
- No sustituye una auditoria completa de seguridad ni una revision juridica; pendiente prueba manual dentro de GW2.

## v0.2.17: estructura comun nexusbase
- Identidad, firma, update_link, idioma y rutas en addon.toml, validados al compilar e incorporados en la DLL; misma firma, endpoint y ajustes existentes.
- Infraestructura local en nexusbase: ciclo de vida, ArcDPS tipado con limpieza, idioma/plantillas y Documentos. Logica de combate y textos propios conservados en src.
- Datos oficiales e iconos movidos a nexusbase/gw2 y nexusbase/assets; generadores y referencias actualizados.
- Contrato para futuros addons/agentes en nexusbase/README.md y enlace en AGENTS.md; ambos README actualizados.
- Validacion: 90 tests unitarios y 2 de configuracion correctos; ejemplos compilados. DLL release compilada.
- Verificado GetAddonDef de la DLL real: nombre, firma, version, proveedor, URL y callbacks correctos. 1.142.272 bytes, MD5 final `25cae2cf41953fc492e8b5e0f863e1b4`.
- Codigo base `7058c7c` y compatibilidad publica `1829903` publicados; distribucion conjunta `8cd856a` y DLL final `b171b9a`, release `fight-analysis-danger-v0.2.17` con DLL/MD5 y latest=false.
- Atributos sin LFS y blob real verificados. Endpoint exacto Nexus, antiguo, inmutable y asset release coinciden en MZ, tamano y MD5; los cuatro sidecars coinciden.
- La recompilacion final requirio renovar DLL/MD5. Detectada cache CDN de la primera DLL en las URLs de rama; esperada su caducidad real (max-age=300) y verificadas de nuevo las URLs exactas, sin cambiar endpoints ni version.
- API publica i18n conservada mediante reexports. Pendiente solo prueba manual en GW2; sin tocar ajustes personales ni cambios previos de instrucciones/workspace.

## v0.2.16: dano en directo sin dependencia de colores y solo equipo propio
- Corregido descarte de dano/corrups/contribucion live cuando no llegan WVW_TEAMS/ID_TO_GUID: IDs crudos de equipos o afinidad hostil contra jugadores identificados, sin asignar colores enemigos ficticios.
- Grupo propio live identificado por equipo crudo o escuadron registrado; si falta su color se presenta como Nuestro equipo neutro. Actualizados datos de identidad/equipo al recibir callbacks posteriores.
- Resumen, Destacados, pestanas y menu de color live restringidos al propio equipo; aplicado a adicionales. Los logs fuera de combate conservan los tres colores.
- Ambos README y calculos.md actualizados. Formato y 87 tests de todos los targets correctos; regresiones de dano/DPS/corrups/contribucion sin colores, afinidad sin IDs, exclusion de NPCs/aliados y render headless seleccionando solo Nuestro equipo.
- DLL release compilada: 1.149.952 bytes, MD5 `88bfae0e27fdeebc953ecb0fde550a48`.
- Codigo `9d39469`, distribucion `112ec27` y release `fight-analysis-danger-v0.2.16` publicados con DLL/MD5 y latest=false; latest Timers v0.3.18 conservado.
- Atributos sin LFS y blob preparado identico; endpoint exacto Nexus, antiguo, inmutable y asset release coinciden en tamano/MD5/MZ. Los cuatro sidecars coinciden, sin cache antigua en la comprobacion.
- Sin tocar ajustes personales ni cambios ajenos. Pendiente validar en GW2 los eventos reales del combate reportado; las regresiones reproducen el fallo de ausencia de colores, no certifican otros problemas de cobertura del bridge.

## v0.2.15: NPCs solos no activan combate del addon
- Directo y Ocultar en combate exigen dano observado entre POV/mascota propia identificada y jugador enemigo identificado; ENTER_COMBAT, NPCs, aliados y peleas ajenas no bastan.
- Tras primer contacto PvP se mantiene hasta salir de combate de GW2, segun eleccion del usuario; NPCs solos conservan el informe anterior sin iniciar snapshot.
- Peticion adicional: Follow latest vuelve al ultimo log inmediatamente al salir de combate, sin snapshot congelado; al llegar el log nuevo se actualiza. Sin logs queda en espera. Aplicado tambien a ventanas adicionales y probado conservando seleccion manual.
- Deteccion compartida sin bloquear el render con el mutex de eventos. Sin bridge no se inventa PvP a partir de Mumble.
- Actualizados ambos README; formato y 84 tests de todos los targets correctos, incluidos NPCs/aliados/peleas ajenas, primer dano entrante, mascotas propias, condiciones y salida de combate.
- DLL release compilada tras ambas peticiones: 1.146.880 bytes, MD5 `25bca40472d458c2eb06eb220e9a65ed`.
- Codigo `89cd5ba`, distribucion `8a6a327` y release `fight-analysis-danger-v0.2.15` publicados con DLL/MD5 y latest=false; latest de Timers v0.3.18 conservado.
- Atributos sin LFS, blob preparado identico; URL exacta de Nexus, antigua, inmutable y asset release coinciden en tamano/MD5/MZ y los cuatro sidecars MD5 verificados, sin cache antigua en esta comprobacion.
- Sin modificar ajustes personales ni cambios ajenos. Pendiente comprobacion en GW2 de la entrega real de eventos del bridge y transiciones visuales.

## v0.2.14: ocultar ventanas durante cargas
- El render compartido exige gameplay activo en Nexus ademas de mapa WvW: oculta ventanas principales y adicionales durante cargas/seleccion de personaje, aunque Mumble conserve el mapa anterior.
- Sin modificar visibilidad guardada, posiciones, historial ni seguimiento; reaparecen al volver al juego si estaban visibles.
- Actualizados ambos README; formato y 82 tests de todos los targets correctos; DLL release 1.145.344 bytes, MD5 `596008769d8ad4a3b2fb95bde3dc5b79`.
- Codigo `ce35f99`, distribucion `a26cd5f` y release `fight-analysis-danger-v0.2.14` publicados con DLL/MD5; latest Timers v0.3.18 conservado.
- DLL exacta de Nexus, antigua, asset release e inmutable verificadas: mismo tamano, MD5 y MZ; atributos sin LFS y blob preparado identico. Sidecars antiguo/release/inmutable correctos; sidecar del endpoint exacto aun cacheado con MD5 de v0.2.13 al verificar, aunque su DLL ya es v0.2.14.
- A las 23:59 del 22/10, tras aviso de que Nexus seguia en v0.2.13, repetida descarga exacta: DLL 1.145.344 bytes, MZ y MD5 `596008769d8ad4a3b2fb95bde3dc5b79`; sidecar exacto ya renovado con el mismo MD5 (cache MISS). Sidecar antiguo tambien coincide. Distribucion remota verificada; deteccion/recarga en el cliente pendiente.
- Sin modificar ajustes personales ni cambios ajenos. Comprobacion visual dentro de GW2 pendiente.

## v0.2.13: Healing completo y seguimiento provisional en directo
- Eliminado el recorte de Healing a 4–10 jugadores, incluido portapapeles; todas las filas con datos disponibles y scroll con el alto existente. healing_rows antiguo ignorado y retirado de defaults/guardado.
- Activada la superficie arc del SDK Nexus existente; sin nueva dependencia directa. Suscripciones locales/de escuadron y replay de roster al cargar, sin retener punteros de callbacks.
- Follow latest muestra snapshots provisionales en las tres ventanas y adicionales; seleccion manual mantiene el historial separado. Flujos solapados no se suman dos veces.
- Motor compartido con EVTC para dano, corrups, downs, contribucion, armas, limpias y eventos Healing Stats compatibles; no se inventan colores ni cobertura de curas.
- Reinicio por combate, congelacion al salir, limpieza al cambiar instancia y sustitucion por log reciente; limite explicito de 1.000.000 eventos con aviso.
- Actualizados ambos README y calculos.md. Formato y 81 tests de todos los targets correctos, sin avisos; regresiones de flujos solapados, limite explicito, pertenencia temporal, coincidencia del log y scroll real de 60 healers.
- DLL release compilada: 1.144.832 bytes, MD5 `c683c9e79250630fe428db59b929929b`.
- Codigo publicado en `88063ad`; distribucion `695a3ff` y release `fight-analysis-danger-v0.2.13` con DLL/MD5 y `--latest=false`; latest de Timers v0.3.17 conservado.
- Verificados atributos sin LFS y blob preparado real. URL exacta de Nexus, antigua, inmutable y asset release coinciden en tamano, MD5 y MZ; los tres sidecars MD5 coinciden. El sidecar del endpoint exacto requirio esperar la caducidad de cache, sin cambiar URL ni version.
- No modificados ajustes personales, DLL instalada ni cambios ajenos del workspace/AGENTS.md; cambios concurrentes de Timers no incluidos en nuestros commits.
- Pendiente validacion dentro de GW2 de los eventos realmente entregados por ArcDPS Integration, especialmente metadatos de colores y Healing Stats; tests sinteticos no certifican esa cobertura en juego.

## v0.2.12: defaults actuales y tooltip Healing compacto
- Leido settings.txt personal en Documentos, sin modificarlo. Defaults replican todos sus valores salvo locked=false en las tres ventanas; fixed_position ya era false.
- Resumen: posicion 391/1, ancho 259, alto 78. Destacados: 233/624, ancho 300, alto 112. Healing: 0/624, ancho 234, alto 116.
- Conservados opacidades 49/38/32 %, columnas, visibilidad, autoapertura, seguimiento y opciones generales actuales.
- Actualizados settings.default.json, expectativas de tests y ambos README; preferencias existentes siguen teniendo prioridad.
- Eliminada del tooltip de jugador la explicacion larga sobre limpias registradas al escuadron; conservadas cifra de limpias, demas metricas y avisos de cobertura.
- Formato y tests de todos los targets correctos: 76 pasados, 0 ignorados; DLL release compilada sin avisos, 1.097.216 bytes y MD5 `beb95e9a1354f76428e77f8d7d6c7838`.
- Distribucion `e8961e1` y release `fight-analysis-danger-v0.2.12` publicadas con DLL/MD5 y `--latest=false`; latest de Timers v0.3.16 preservado.
- Atributos sin LFS y blob preparado real del tamano local; URL exacta de Nexus, antigua, inmutable y asset de release coinciden en tamano, MD5 y MZ. Tres sidecars MD5 verificados.
- Sin modificar ajustes personales, DLL instalada ni cambio previo del workspace. Comprobacion visual dentro de GW2 pendiente.

## v0.2.11: ingles temporal con Ctrl derecho
- Ctrl derecho mantenido muestra todo el addon en ingles mediante la seleccion central de idioma; al soltar vuelve al idioma automatico, sin guardar preferencias ni reprocesar logs.
- Consulta del estado real de VK_RCONTROL, sin consumir eventos; solo aplica si el proceso del juego tiene la ventana de primer plano. No afecta Ctrl izquierdo ni queda enganchado al perder foco.
- IDs estables en pestanas de colores para conservar el equipo seleccionado al cambiar temporalmente el idioma.
- Eliminada la prueba ignorada dependiente del archivo privado FAD_WEAPON_LOG; conservadas las regresiones sinteticas de armas/reflexiones/submarinas.
- Actualizados README.md y readme.html. Formato y tests de todos los targets correctos: 76 pasados, 0 ignorados; render headless conserva equipo al alternar idiomas.
- DLL release compilada sin avisos: 1.097.216 bytes, MD5 `dea41baa5d19403ca2de5313b211082c`.
- Distribucion `fb5ba83` y release `fight-analysis-danger-v0.2.11` publicadas con DLL/MD5 y `--latest=false`; latest de Timers v0.3.16 preservado.
- Atributos sin LFS y blob preparado real del tamano local. URL exacta de Nexus, antigua, inmutable y asset release verificados por tamano, MD5 y MZ; tres sidecars MD5 coincidentes.
- No modificados ajustes personales, DLL instalada ni cambio previo del workspace. Pendiente comprobacion fisica del teclado dentro de GW2.

## v0.2.10: contribucion nativa AxiLog y opciones compactas
- Down contribution cambia a ancla de salud >=99 % con anticipo de 2000 ms, fallback al inicio del log y reset por objetivo de 2100 ms; limites inclusivos por tiempo.
- Resultado downed como fuente de downs, sin inferencia por estados; eventos de salud invalidos descartados y condiciones/mascotas con resolucion temporal e identidad comprobada.
- Se conserva alcance por colores/filas de agente y el resto de metricas; no se afirma paridad de totales entre poblaciones distintas.
- Eliminados top/top_dps/top_strips y down_contribution_secs de defaults/modelo/serializacion; claves antiguas ignoradas, restantes preferencias conservadas.
- Eliminados slider y explicacion larga de tooltips: cabeceras muestran solo "Down contribution"; valores conservados en jugador/equipo/copia.
- Incluidos cambios preparados de mostrar armas en menu contextual y down c total por color en Resumen.
- Pruebas de biblioteca: 75 correctas y 1 privada ignorada; incluye render headless de armas ocultas/visibles, cuatro idiomas y columnas. Sin dependencias nuevas.
- Formato, tests de todos los targets y compilacion release correctos, sin avisos; DLL 1.096.704 bytes, MD5 `f66294b3fd94f3386b25b49e56525a40`.
- Distribucion `c00534b` y release `fight-analysis-danger-v0.2.10` publicadas con DLL/MD5 y `--latest=false`; latest de Timers v0.3.16 preservado.
- Atributos filter/diff/merge/text unset y blob preparado del mismo tamano que la DLL local, sin LFS.
- URL exacta de Nexus, URL antigua, URL inmutable y asset de release descargados como bytes reales: mismo tamano, MD5 y cabecera MZ. Los tres MD5 remotos tambien coinciden.
- Ajustes personales, DLL instalada y cambio previo del workspace no modificados. Comparacion de totales entre parsers sobre un mismo EVTC real y comprobacion visual en GW2 pendientes.

## Cambios preparados: mostrar armas y Down contribution en Resumen
- Destacados incorpora "Mostrar armas" en el menu contextual, reutilizando el ajuste existente, activo por defecto y guardado tambien desde opciones de Nexus.
- Desactivarlo elimina la fila completa de armas, sin hueco; compartido con ventanas adicionales y sin sobrescribir preferencias guardadas.
- Resumen incorpora columna "down c": suma del Down contribution por color, cifras compactas y total completo en tooltip del equipo. Cabecera con nombre completo e intervalo al pasar el raton.
- Sin cambios en formulas, ranking ni atribucion; actualizados README.md, readme.html y calculos.md.
- Preparadas pruebas de suma por color/saturacion, persistencia/default de armas y eliminacion de la segunda linea en render headless.
- Verificados formato Rust, whitespace, default JSON y documentacion sin compilar; DLL release anterior intacta (MD5 4026ff4239c2cd07f59b8d7fd0ef3a41).
- Por peticion del usuario: sin compilar, ejecutar tests que compilen, incrementar version, commit/push ni publicar DLL. Validacion de ejecucion pendiente.

## Auditoria documental de metricas frente a AxiBridge
- Creado calculos.md con inventario de columnas, tooltips, totales, tasas, ranking y disponibilidad; enlazado desde README.md y readme.html.
- Referencias fijadas: addon v0.2.9 (9eb411b), AxiBridge 44d58c5 y AxiLog v1.16.0 (9bd8297), dependencia real del parser nativo.
- Confirmado que Dano comparte la suma base, pero no garantiza igualdad: NPC hostiles, eventos admitidos, mascotas, identidades y poblaciones distintas.
- Documentadas diferencias de duracion/DPS, contribucion por salud frente a ventana fija, strips, limpias arcdps frente a conteo local, muertes por estado frente a killing blow y alcance/sanitizacion de healing.
- Separadas ruta nativa, informes EI importados y superficies upstream con metodologias distintas; no tomada la especificacion desactualizada como prueba del algoritmo nativo.
- Comprobados enlaces locales, 16 referencias externas fijadas por commit y whitespace; sin ejecutar tests/compilacion al no haber cambios funcionales.
- Pendiente: comparacion numerica reproducible con el mismo EVTC y aprobacion del contrato de metricas antes de adaptar calculos.
- Solo documentacion: sin cambios de codigo, ajustes, version, DLL ni publicacion Nexus.

## v0.2.9: cabecera compacta y resumen sin asterisco
- Cabecera Down contribution abreviada a "down c" en todos los idiomas, con ancho calculado usando el texto corto.
- Tooltip especifico al pasar el raton por esa columna: "Down contribution" y explicacion del intervalo usado.
- Conservados nombres completos en menus, opciones, tooltips de jugador y copia; sin cambios en calculo ni orden.
- Resumen sin asterisco del equipo propio; conservados orden del equipo propio primero y su identificacion en tooltip.
- Mantenido el default de 3 s ya presente en settings.default.json por confirmacion del usuario; alineados parser CLI, pruebas y documentacion. No se cambian preferencias guardadas.
- Actualizados README.md y readme.html. Formato correcto y 70 tests pasados; 1 prueba privada ignorada. Cabecera corta, ancho y hover comprobados en render headless.
- DLL release compilada: 1.105.408 bytes, MD5 `4026ff4239c2cd07f59b8d7fd0ef3a41`.
- Distribucion `661e60e` y release `fight-analysis-danger-v0.2.9` publicadas con DLL/MD5 y `--latest=false`; latest de Timers preservado.
- Verificados atributos sin LFS, blob preparado real, URL exacta de Nexus, URL antigua, URL inmutable y asset release: mismo tamano, MD5 y cabecera MZ. Los tres MD5 publicados coinciden.
- No modificados ajustes personales, DLL instalada ni cambios previos del workspace. Comprobacion visual dentro de GW2 pendiente.

## v0.2.8: Down contribution y filtro de inactivos
- Destacados oculta jugadores con dano y corrups a cero, tambien en copia/chat.
- Nueva columna Down contribution: dano a jugadores enemigos durante los 5 s previos a cada derribo, incluido el golpe que derriba.
- Ventana configurable de 1 a 60 s en opciones de Destacados; se guarda y recalcula el historial en el worker.
- Excluidos dano a derribados, aliados, NPC y objetivos sin derribo. Se incluyen condiciones/mascotas con propietario conocido; sin duplicar aportaciones tras rally ni notificaciones duplicadas de derribo.
- Orden inicial oculto: dano total + corrups * 2000 + Down contribution, sin multiplicador adicional.
- Conservados cache, clipping, scroll, orden manual, copia/chat, equipos y ventanas adicionales.
- Migracion layout_version=4 incorpora la columna una vez, conservando geometria y permitiendo ocultarla despues.
- Actualizados defaults, README.md y readme.html; sin dependencias nuevas.
- Formato y 70 tests correctos, 1 prueba privada ignorada. Verificados limites exactos de tiempo, condiciones/mascotas, rallies, notificaciones en ambos ordenes, filtro, puntuacion y migracion.
- Render headless en cuatro idiomas, 15 combinaciones de columnas para Destacados/Healing y scroll de 180 jugadores conservados.
- DLL release compilada: 1.104.384 bytes, MD5 `65c88d7a39c602390ba588288ec14032`.
- Publicada distribucion `80234e8` y release `fight-analysis-danger-v0.2.8` con DLL/MD5 y `--latest=false`; latest de Timers preservado (`v0.3.16`).
- Verificados atributos sin LFS y blob preparado real del tamano local. URL exacta de Nexus, URL antigua, URL inmutable y asset de release coinciden en tamano, MD5 y cabecera MZ.
- Los tres MD5 publicados coinciden exactamente. No modificados ajustes personales, DLL instalada ni cambios previos del workspace.
- Pendiente unicamente la comprobacion visual dentro de GW2.

## v0.2.7: listas completas, orden ponderado y limpias
- Destacados incluye todos los jugadores del equipo con scroll, orden inicial oculto dano + corrups * 2000 y desempate estable.
- Ranking/cifras en cache por combate, equipo y orden; filas fuera de vista sin generar iconos, textos ni tooltips de jugador.
- Conservados orden manual por columnas, armas, equipos, ventanas adicionales e historial; copia/chat usa la lista completa en el orden actual.
- Healing incorpora limpias como tercera columna visible y criterio de orden opcional; total de curas/barrera sigue disponible.
- Limpias leidas del EVTC ordinario, una por condicion retirada completamente, no por stack; incluye propias/mascotas conocidas y respeta escuadron en el momento del evento.
- Los jugadores con solo limpias pueden aparecer sin Healing Stats; curas/barreras ausentes siguen como no disponibles.
- Resumen incorpora corrups totales por equipo (mismo contador de strips que Destacados).
- Migracion a layout_version=3 anade limpias una vez, conserva geometria/opacidad y permite ocultar la columna despues.
- Actualizados README.md, readme.html y defaults; sin dependencias nuevas ni cambios en ajustes personales.
- Formato correcto, 65 tests pasados y 1 prueba privada ignorada; render headless en cuatro idiomas, 15 combinaciones de columnas Healing y scroll hasta el ultimo jugador con 180 participantes.
- DLL release compilada: 1.094.656 bytes, MD5 `01aa2c2f70f4428e4f0e7ce6b098f5c0`.
- Publicada distribucion `de15d5a` y release `fight-analysis-danger-v0.2.7` con DLL/MD5 y `--latest=false`; latest de Timers preservado (`v0.3.16`).
- Atributos filter/diff/merge/text unset y blob preparado del mismo tamano que la DLL local; sin puntero LFS.
- Verificados URL exacta de Nexus, URL antigua, URL inmutable y asset de release: mismo tamano, MD5 y cabecera MZ; los tres MD5 remotos coinciden exactamente.
- No modificados ajustes personales, DLL instalada ni cambios previos en el workspace. Comprobacion visual dentro de GW2 pendiente.

## v0.2.6: mantener abierto el historial al seleccionar logs
- Las entradas de logs no cierran el menu contextual ni el submenu Historial al seleccionarlas.
- Conservadas la seleccion compartida entre todas las ventanas y la desactivacion del seguimiento automatico.
- Actualizados README.md y readme.html.
- Prueba ImGui con clic derecho real, seleccion consecutiva de dos logs sin reabrir, persistencia del submenu y cierre con clic fuera.
- Formato correcto y 60 tests pasados; 1 prueba privada ignorada. DLL release compilada: 1.102.848 bytes, MD5 `dac8a163f3b4e661398d839b2516b051`.
- Publicada distribucion `0832bc5`, integrando los cambios remotos de README sin sobrescribirlos; release `fight-analysis-danger-v0.2.6` con DLL/MD5 y `--latest=false`.
- Verificados URL exacta de Nexus, URL antigua, URL inmutable y asset release: mismo tamano, MD5 y cabecera MZ. Ambos MD5 remotos exactos, DLL real sin LFS y latest de Timers preservado (`v0.3.16`).
- No modificados los ajustes personales ni la DLL instalada. Comprobacion visual dentro de GW2 pendiente.

## v0.2.5: valores iniciales desde JSON del proyecto
- Capturada la ultima configuracion guardada, incluidos cambios del menu derecho: anchos 210/233/186, altos 78/161/77, opacidades 49/38/32 %, sin ocultacion en combate.
- settings.default.json es la fuente unica real de valores iniciales generales y por ventana, embebida al compilar.
- Conservadas posiciones, columnas 6, visibilidad, apertura automatica y seguimiento desactivado; locked/fixed_position son false en los valores iniciales.
- El archivo personal solo se lee; no se cambia ni su contenido ni su formato. Preferencias guardadas siguen teniendo prioridad.
- Incorporados serde/serde_json para parseo tipado estricto, sin parser JSON artesanal ni valores iniciales duplicados.
- Corregida propagacion de seguimiento desde opciones de Nexus: debe aplicarse fuera del selector de equipo.
- Formato y 60 tests correctos (1 prueba privada ignorada), incluido JSON completo/estricto, limites, defaults reales y prioridad de ajustes guardados.
- Comparados todos los campos del JSON con los ajustes personales actuales: coinciden salvo desbloqueo solicitado. SHA256 personal sin cambios durante el trabajo.
- DLL release compilada: 1.102.336 bytes, MD5 `16dd4630dec94c4deb1ee0d6068b1201`.
- Publicados fuente (`4af5445`) y distribucion (`416ca6d`), release `fight-analysis-danger-v0.2.5` con DLL/MD5 y `--latest=false`; latest de Timers preservado.
- Verificados asset release, URL de commit, URL exacta de Nexus y URL antigua: mismo tamano/MD5/MZ. Ambos MD5 publicados exactos (32 caracteres), blob de indice real y atributos sin LFS.
- SHA256 de ajustes personales identico tras publicar. No modificada la DLL instalada; comprobacion visual dentro de GW2 pendiente.

## v0.2.4: preferencias iniciales elegidas y etiquetas claras
- Leidos ajustes actuales sin modificarlos: tres ventanas habilitadas/visibles, apertura automatica y ocultacion en combate, sin seguimiento automatico.
- Nuevas instalaciones: anchos 211.32288/211.32288/191.70404 px, opacidades 21/38/32 %, posiciones 391,1 / 0,472 / 603,1.
- Por defecto ninguna ventana bloqueada ni con posicion fija. Alto automatico y columnas dano+strips/curas+barrera se mantienen al faltar en los ajustes leidos.
- Etiquetas aclaradas: Habilitar ventana / Visible ahora. Reset de estilo utiliza los nuevos valores por ventana.
- Las preferencias existentes no se sobrescriben y la migracion historica de anchos anteriores se mantiene.
- Formato y 59 tests correctos (1 prueba privada ignorada), incluidos valores nuevos, persistencia de preferencias previas y render compacto/resize.
- DLL release compilada: 1.020.928 bytes, MD5 `35959b4baf8fbbbd87612e5a0bec01d2`.
- Publicados fuente (`56ba76c`) y distribucion (`a393934`), release `fight-analysis-danger-v0.2.4` con DLL/MD5 y `--latest=false`; latest de Timers preservado (`v0.3.16`).
- Asset release y URL inmutable verificados con tamano/MD5/MZ. Las URLs exacta de Nexus y antigua sirvieron inicialmente v0.2.3 por cache; tras esperar y reintentar ambas coincidieron con v0.2.4, junto al MD5 remoto.
- No modificados los ajustes instalados ni la DLL del juego. Validacion visual dentro de GW2 pendiente.

## v0.2.3: pestanas y columnas minimalistas, historial sincronizado
- Destacados: pestanas solo de colores presentes; DPS/dano/strips combinables, dano + strips por defecto.
- Healing: columnas combinables, curas + barrera por defecto; sin >= en cifras compactas, conservando aviso de cobertura y detalle.
- Resumen: jugadores, kills y dano total por equipo.
- Eliminada la fila de historial cargado en las tres ventanas; se conservan errores/estados relevantes.
- Todas las ventanas redimensionables desde la esquina inferior derecha, con ancho/alto persistentes.
- Cambiar combate o seguimiento desde cualquier ventana afecta a todas, incluidas las adicionales.
- Formato y 58 tests correctos (1 prueba privada ignorada). Render headless a 175 px, cuatro idiomas, fuente 100/150 %, siete combinaciones de columnas, clics de pestanas y resize real en X/Y verificados.
- Verificados historial sincronizado incluso en ventanas ocultas/adicionales, dano por color y ausencia de la fila de historial cargado.
- DLL release compilada: 1.022.464 bytes, MD5 `0b683551acf481edbae054768f0b1275`.
- Publicados fuente (`d8ce765`) y distribucion (`2ffe7ce`), release `fight-analysis-danger-v0.2.3` con DLL/MD5 y `--latest=false`.
- Verificados URL exacta de Nexus, URL antigua, asset release y URL inmutable de commit: mismo tamano/MD5 y cabecera MZ. Ambos MD5 remotos exactos (32 caracteres), atributos sin LFS y latest de Timers preservado (`v0.3.16`).
- Pendiente unicamente la comprobacion visual dentro de GW2; no se han modificado la DLL instalada ni los ajustes del usuario.

## v0.2.2: armas en texto minimalista
- Por eleccion del usuario, todas las armas se muestran con nombres traducidos en una linea compacta, sin iconos.
- Conservado el azul para uso subacuatico identificado y el tooltip con la lista completa y la advertencia de inferencia.
- Mantenidos iconos de clase/especializacion, anchos, opacidad y deteccion de armas existentes.
- Formato, 56 tests automaticos (render 175 px/cuatro idiomas/azul acuatico incluido) y build release correctos; prueba privada de parser no ejecutada en este cambio solo visual.
- DLL release: 1.000.448 bytes, MD5 `580dcb321b7bcacd472ab81707495177`.
- Publicados fuente y distribucion (`21c977d`), release `fight-analysis-danger-v0.2.2` con DLL/MD5 y `--latest=false`.
- Verificados URL exacta de Nexus, URL antigua, asset release y URL de commit: mismo tamano/MD5 y cabecera MZ. Verificados ambos MD5 publicados y atributos sin LFS.
- Pendiente unicamente la comprobacion visual dentro de GW2.

## v0.2.1: interfaz minimalista tipo arcdps
- Tres ventanas con ancho inicial de 175 px, fondo/titulo al 75 % de opacidad y sin flecha para colapsar.
- Clic derecho abre submenus Historial, Color, Estadisticas, Estilo y Posicion; eliminados controles permanentes del cuerpo.
- Resumen con celdas de la altura del texto; Destacados con una metrica seleccionable y Healing con una sola linea por jugador, sin barras.
- Conservados tooltips detallados, copia/chat, historial independiente y ventanas adicionales.
- Mary Vai en 20261019-212443: lanza y mandoble, sin uso subacuatico. Sus cuatro Muros de reflejo causaban atribucion de impactos de baculo, daga y cetro; ahora se priorizan lanzamientos reales (AnimationStart 67) y paletas de profesion, sin limitar artificialmente el numero de armas.
- Cambios de conjunto 4/5 terrestres y 0/1 acuaticos; iconos azules solo para uso acuatico identificado.
- Incluidos 19 iconos oficiales de objetos-arma GW2, fuentes/atribucion/hashes y script stdlib de regeneracion.
- Validadas ventanas ImGui a 175 px, opacidad real 75 %, alturas de texto, Healing con 4/10 filas, cuatro idiomas, escala de fuente 100/150 % y menu contextual con clic derecho en el titulo.
- Validacion completa: 57 tests automaticos correctos y prueba privada adicional de Mary Vai correcta; formato y build release correctos. En el log real siguen iguales jugadores, dano, kills y cobertura de healing.
- DLL release: 1.080.832 bytes, MD5 `288c612bbfe8040f602da9bcc3a0f8e6`, cabecera MZ verificada.
- Publicados fuente y distribucion; release `fight-analysis-danger-v0.2.1` con DLL/MD5 y `latest=false`, sin sustituir el latest de Timers.
- Descargados endpoint exacto de Nexus, endpoint antiguo, asset de release y URL inmutable del commit: todos coinciden en tamano, MD5 y cabecera MZ. Comprobados ambos checksums publicados de 32 caracteres.
- Pendiente de comprobacion manual: aspecto/interaccion dentro de GW2. No se ha modificado la instalacion ni los ajustes del juego.

## v0.2.0: WvW Combat Insights
- Nuevo nombre visible; se mantienen DLL, firma, endpoint y carpeta de ajustes.
- Tres ventanas independientes y tres pestanas de opciones: Resumen, Jugadores destacados y Healing.
- Posicion X/Y, bloqueo, ancho y seguimiento de logs por ventana; instantaneas adicionales independientes.
- Resumen de tres colores con equipo propio primero, jugadores/kills y tooltips de eventos.
- Destacados compacto con iconos de especializacion y pictogramas originales de armas; ranking aliado disponible.
- Healing de 4 a 10 filas, icono de clase, curas/barreras/total y barras opcionales.
- Parser Healing Stats firma 0x9c9b3c99 revisiones 1/2: pertenencia al escuadron registrada, mascotas, ecos de peers y cobertura conservadoramente parcial; metrica ausente = "-", total incompleto = ">=".
- El filtro de dano/strips no afecta a kills de jugadores; al cambiarlo se reprocesa el historial, manteniendo las instantaneas adicionales.
- Validados formato, 50 tests (incluido render ImGui sin juego en es/en/fr/de y anchos exactos) y build release.
- Log real 20261019-212443: 20 jugadores (7 rojos, 13 verdes), 12 kills rojas / 12 muertes verdes; healing parcial y 7 miembros de escuadron identificados. No se afirma equivalencia numerica con Elite Insights.
- DLL release: 1.034.240 bytes, MD5 `6cefc3b756bcc6b13e06bd692ecd6f53`, cabecera MZ comprobada.
- Publicados fuente y distribucion; release `fight-analysis-danger-v0.2.0` con DLL y MD5 (32 caracteres), sin cambiar el latest de Timers.
- Verificados endpoint exacto de Nexus, endpoint antiguo, asset de release y URL del commit inmutable: los cuatro coinciden en tamano, MD5 y cabecera MZ. Comprobados tambien ambos MD5 publicados.
- Pendiente de comprobacion manual: aspecto e interaccion dentro de GW2; no se ha instalado ni manipulado la DLL del juego durante esta tarea.

## v0.1.8: resumen de enemigos y eventos por color
- El titulo mantiene visibles al contraer la ventana los totales de jugadores de los tres colores: Rojo, Verde y Azul.
- Al desplegarla, una tabla por color muestra jugadores, muertes y derribos de jugadores registrados en el EVTC, conservando el recuento de aliados e independientemente del filtro de objetivos de dano/strips.
- Actualizados README.md y readme.html. Validacion: `cargo fmt --all`, 36 tests con `cargo test --all-features` y `cargo build --release` correctos.
- DLL release de 869.376 bytes publicada y verificada (MD5 `1e0e5d08b22f5f79f875dd21a314500d`) en los dos endpoints de actualizacion y en el asset de `fight-analysis-danger-v0.1.8`; los tres responden con `MZ` y el mismo MD5.

## v0.1.7: fecha legible para el nombre del log
- La cabecera convierte nombres con formato `AAAAMMDD-HHMMSS` en una fecha y hora legibles, con etiquetas localizadas para hoy/ayer y conservando el nombre original si no es una fecha válida.
- Actualizados README.md y readme.html.
- Validación: `cargo fmt --all`, 34 tests con `cargo test --all-features` y `cargo build --release` correctos.
- DLL de 867.840 bytes publicada en `gw2addonsreleases`, con MD5 `95a440b78ef5c62737c4dee1b54950c5` y release `fight-analysis-danger-v0.1.7` (sin cambiar el latest de Timers).
- Verificados ambos endpoints de actualización y el asset de release: tamaño y MD5 coinciden, cabecera `MZ`.

## v0.1.6: totales Rojo, Verde, Azul y Aliados
- Eliminado el contador Grupo + squad y añadido Azul; conservado el cálculo de Aliados (otros subgrupos conocidos de tu color, excluyendo tu subgrupo).
- Aclarado en tooltip y ambos README que los totales por color incluyen jugadores dentro/fuera del grupo y al propio jugador; Aliados no equivale a la squad completa.
- Ampliada la prueba de recuentos para incluir Azul y ausencia de datos de POV/subgrupo.
- Validación: formato, 32 tests y build release correctos. DLL preparada en target/release/fight_analysis_danger.dll.

## v0.1.5: etiqueta Grupo + squad
- Por confirmacion del usuario, cambiado solo el texto Grupo por Grupo + squad, manteniendo los recuentos existentes de subgrupo y aliados.
- Actualizadas las cuatro traducciones, el tooltip y ambos README para explicar que no cambia el alcance del recuento.

## Instrucciones de distribucion segura
- Documentada en AGENTS.md la prevencion del puntero Git LFS, las excepciones para ambas DLL, la comprobacion del blob preparado y la validacion de descargas por tamano, cabecera MZ y MD5.
- Incluidas comprobaciones de endpoints antiguos, releases del repo compartido, diferencias entre LFS y cache y recuperacion manual sin tocar ajustes. Cambio solo documental, sin nueva DLL.

Objetivo: addon de Nexus (complemento de [WvW Fight Analysis](https://github.com/jake-greygoose/WvW-Fight-Analysis-Addon))
que tras cada combate muestre, por cada color enemigo, los N (por defecto 2) jugadores más peligrosos
por **DPS** y por **strips**.

## Decisiones tomadas

- **Rust + nexus-rs** (mismo commit que `gw2-timers`), sin dependencias de terceros salvo `nexus` y `windows`.
- `EV_LOG_PARSED` de WvW Fight Analysis entrega estructuras C++ sin datos por jugador: se usa solo como
  **disparador** (nombre del archivo) y el `.zevtc` se analiza con un parser propio. Respaldo: vigilancia de la carpeta.
- **Sin guild**: en WvW arcdps solo registra la guild de los aliados (evento GUILD = 29) y el "nombre" de los enemigos
  es su rango WvW. Se muestra especialización + rango.
- Strips: desde 2025 arcdps registra las retiradas de boons como statechange 72 (BuffRemoveAll); se cuentan esos
  (y el formato antiguo `is_buffremove == 1`).
- Agentes duplicados en el log: se conserva uno por dirección.

## Hecho

- [x] v0.1.4: corregida la autoactualizacion que descargaba un puntero Git LFS de 131 bytes. Mismo endpoint que Timers y DLL excluida de LFS en distribucion para reparar tambien el enlace de versiones anteriores. Documentada la recuperacion manual si la DLL instalada ya fue sustituida.
- [x] v0.1.4 validada con formato, 32 tests y build release. Blob de distribucion comprobado: 861.696 bytes, no un puntero LFS. MD5 `b2ac9f929e9614d976c97dece87d2b13`.
- [x] Resumen compacto al terminar el combate: duración en horas/minutos/segundos y totales del log para rojo, verde, subgrupo propio y aliados en otros subgrupos, con ayuda contextual sobre el alcance/ambigüedad de esos datos.
- [x] Copia al chat y ranking con daño total: se muestra el daño total del jugador en lugar del promedio de DPS sobre toda la duración del combate (que resultaba en valores extrañamente bajos y poco claros).
- [x] Idioma automatico es/en/fr/de: RealTime de Nexus, evento de Unofficial Extras y respaldo con LANGUAGE (14) del ultimo EVTC. Nexus/ingles solo cuando no hay datos del juego. Sin selector.
- [x] Traducidas ventana, opciones, ayudas, estados/errores, ranking y chat; IDs de ImGui estables al cambiar de idioma.
- [x] Catalogo oficial integrado de 9 profesiones, 36 especializaciones elite y 105 rangos WvW en cuatro idiomas, regenerable con `tools/update_game_names.py`. Sin red al jugar; armas traducidas porque la API no localiza sus identificadores.
- [x] Activada la funcion opcional `rtapi` del SDK existente; no se anaden dependencias directas.
- [x] Validado el log real de 18:37:55 en es/en/fr/de: mismos 429/324/90 DPS y 3 filas; LANGUAGE=4 detecta espanol sin indicar idioma. 28 tests OK y DLL release preparada.
- [x] Ventana limitada siempre a mapas WvW mediante Mumble, con los mismos 8 tipos de mapa de `gw2-timers`, sin opcion para desactivarlo. No aparece sin enlace Mumble ni con mapa cero; conserva historial y visibilidad fuera de WvW.
- [x] Corregido el dano de condiciones: resultados actuales 14–18 (ticks y fuera de ciclo), conservando el formato antiguo (0), mascotas y filtros de objetivos.
- [x] Revisados los logs reales del 12/10/2026: el de 18:33:56 detecta un enemigo verde sin dano/strips propios registrados; no se anade ninguna seccion de enemigos sin actividad.
- [x] Anadidos tests del parser y ranking para condiciones, resultados sin dano, mascotas y filtros.
- [x] Validacion: 19 tests OK; totales exactos contrastados con un lector independiente en 7 logs reales (incluido el nuevo de 18:37:55). En 18:21:45 el Thief verde pasa de 597 a 6032 de dano (5435 de condiciones).
- [x] `src/inflate.rs`, `src/zip.rs` – descompresión propia
- [x] `src/evtc.rs` – agentes, equipos (WvWTeams / IDToGUID / TeamChange), POV, daño (con mascotas) y strips
- [x] `src/danger.rs` – ranking top N por color enemigo, resumen y líneas para el chat
- [x] Jugadores compartidos por los tops de DPS y strips unificados en ventana, copia y chat, sin rellenar huecos
- [x] Cantidades independientes de DPS y strips (1–5, 2 por defecto en cada una), compatibles con el ajuste antiguo
- [x] `src/settings.rs` – ajustes `clave=valor` en `Documentos\Guild Wars 2\addons\fight_analysis_danger\settings.txt`
- [x] `src/logs.rs` – carpeta de logs (`arcdps.ini` → `boss_encounter_path`, o Documentos) y búsqueda de logs
- [x] `src/app.rs` – hilo de análisis (evento, vigilancia cada 3 s, "analizar último", reanálisis al cambiar opciones)
- [x] Historial circular configurable (1–20, por defecto 10) y acción para reprocesar los logs recientes
- [x] Armas estimadas a partir de habilidades de arma usadas y catálogo oficial de habilidades de GW2
- [x] Probado dentro del juego (versión anterior)
- [x] Iconos de especialización y colores por profesión en la ventana
- [x] `src/ui.rs` – ventana con tablas por color, historial, copiar ranking y resumen compacto de chat, opciones de Nexus, ocultar en combate (Mumble)
- [x] `src/lib.rs` – registro en Nexus, evento `EV_LOG_PARSED`, atajo ALT+SHIFT+P
- [x] `examples/analyze.rs` – prueba desde consola (sin ruta usa el log más reciente)
- [x] 32 tests unitarios: condiciones, WvW, idioma del EVTC, prioridad del idioma real, traducciones completas, errores, ranking/chat, recuentos de jugadores por color/subgrupo y duracion compacta; probado con logs reales (log de 12 MB en ~0,3 s)
- [x] `install.ps1`, `install.bat`, `README.md`, `readme.html`, `.gitignore`, `agents.md`
- [x] Compilado en release: `target\release\fight_analysis_danger.dll`

## Pendiente / ideas

- [ ] Revalidar dentro del juego historial, chat, armas, iconos, colores y cambios de idioma (es/en/fr/de).
- [x] Repositorio GitHub publicado: https://github.com/cbart-cmd/Fight-Analysis-Danger
- [x] Version 0.1.3: duracion compacta y recuentos de jugadores al terminar el combate; `cargo fmt --all -- --check`, `cargo test --all-features` (32 tests) y `cargo build --release` correctos.
