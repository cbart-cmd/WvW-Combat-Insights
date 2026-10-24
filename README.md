# WvW Combat Insights

**English:** Unofficial Guild Wars 2 WvW addon for Nexus: team overview, player
highlights and squad healing. English, Spanish, French and German UI. Official
names and icons are bundled; displaying them does not require the wiki or API.

- **[Download](https://github.com/cbart-cmd/gw2addonsreleases/releases/)**
  (choose `fight_analysis_danger.dll`, not the Timers DLL).
- [Report a bug](https://github.com/cbart-cmd/WvW-Combat-Insights/issues) |
  [Contributing](CONTRIBUTING.md) | [Security](SECURITY.md) | [Changelog](CHANGELOG.md).
- Project-owned code: **GPL-3.0-or-later**, with a
  [limited integration permission](ADDITIONAL-PERMISSION.md) for GW2/Nexus/arcdps.
  Game assets and dependencies retain
  their own terms: [license and third-party notices](THIRD-PARTY-NOTICES.md).

**Quick install:** install [Nexus](https://raidcore.gg/Nexus) and
[arcdps](https://www.deltaconnected.com/arcdps/), enable WvW combat logging,
close GW2, download the DLL above and copy it to the game's `addons` folder.
Reopen GW2 and load **WvW Combat Insights** in Nexus (Ctrl+O).
Healing Stats is needed to supply healing/barrier data; condition cleanses do not
require that extension. WvW Fight Analysis is optional.

**Instalacion rapida:** instala Nexus y arcdps, activa los logs WvW, cierra GW2 y
copia la DLL de la release a `addons` del juego. No necesitas compilar ni instalar
el codigo fuente. Al volver al juego, carga el addon desde Nexus (Ctrl+O).

Resumen de equipos, jugadores destacados y healing de tus combates WvW.
Addon de [Nexus](https://raidcore.gg/Nexus) para Guild Wars 2, compatible con
[WvW Fight Analysis](https://github.com/jake-greygoose/WvW-Fight-Analysis-Addon).

Antes **Fight Analysis Danger**. Se conserva `fight_analysis_danger.dll`, la firma del addon,
el endpoint de actualizacion y la carpeta de ajustes: no hace falta renombrar ni reinstalar nada.
El historial en memoria se reconstruye desde los logs; no se guarda entre reinicios.

Definiciones y diferencias frente a AxiBridge/AxiLog:
[auditoria de calculos](calculos.md). Down contribution adopta desde v0.2.10
la metodologia nativa de AxiLog; las demas diferencias siguen documentadas.

## Estructura comun de desarrollo (v0.2.17)

La infraestructura de Nexus, ArcDPS, idioma, rutas y recursos oficiales esta en
[nexusbase](nexusbase/README.md), con el mismo contrato que Timers y copias locales
independientes. La identidad del addon, incluida `update_link`, se configura en
[addon.toml](addon.toml), validado al compilar e incorporado en la DLL.
La version sigue en Cargo.toml; los ajustes personales, formatos y funciones no cambian.
Guia para nuevos addons y agentes de IA: [nexusbase/README.md](nexusbase/README.md).

## Tres ventanas independientes

En **Nexus → Addons → WvW Combat Insights** hay tres pestanas. Cada persona puede activar una,
dos o las tres ventanas. En nuevas instalaciones las tres estan habilitadas y visibles. Cerrar una ventana no cierra las demas;
si tiene apertura automatica, volvera a abrirse al terminar un combate valido.

### Resumen

- Ancho inicial de **259 px** para Resumen, **300 px** para Destacados y **234 px** para Healing; configurable entre **125 y 1000 px**.
- Tres filas y cinco columnas: **jugadores registrados**, **kills causadas contra jugadores**, **dano total registrado**, **corrups totales** y **down c** por equipo.
  Corrups usa el mismo contador de retiradas de boons (strips) que Destacados.
  Down c suma el Down contribution de todos los jugadores del color; la cabecera muestra su nombre completo
  al pasar el raton, y el tooltip de equipo conserva el total sin abreviar.
- El equipo propio aparece primero, sin asterisco; los demas conservan un orden estable.
- Celdas del color del equipo **con la altura exacta del texto**, sin botones gruesos, y cifras blancas de alto contraste. El tooltip identifica el color,
  el equipo propio y los totales de kills, muertes y derribos registrados.
- Los jugadores incluyen todos los agentes de ese color en el log, no solo el grupo/escuadron.
  No es una estimacion de poblacion actual del mapa.
- Clicar la celda de color o las kills muestra **Jugadores destacados** para ese equipo y combate.
  Esta accion activa esa ventana, aunque antes estuviera desactivada.

### Jugadores destacados

- Pestanas de color para cambiar de equipo, incluido el aliado: solo aparecen los colores presentes en el combate.
  El menu **Color** ofrece la misma seleccion; **Historial** cambia el combate de todas las ventanas a la vez.
- Todos los jugadores del equipo con **dano o corrups registrados**, una sola vez, con **scroll**.
  Se ocultan quienes tienen ambos a cero. Orden inicial descendente por
  **dano total + corrups × 2000 + Down contribution**, calculado sin columna de puntuacion visible.
  El ranking y sus cifras se guardan en cache;
  las filas fuera de la vista no generan textos, iconos ni tooltips de jugador.
- Icono de especializacion delante del nombre/rango; nombre con color de profesion.
- **Armas en texto traducido**, sin iconos: por ejemplo, `Lanza · Mandoble`.
  Una linea discreta de la altura del texto mantiene la estetica minimalista.
  Si no cabe, se recorta con puntos suspensivos; el tooltip conserva la lista completa.
  Las armas con uso subacuatico identificado se muestran en **azul**.
- **Clic derecho → Mostrar armas**, activado por defecto, permite ocultar toda la segunda linea,
  sin dejar espacio vacio. Comparte el ajuste guardado de la pestana Destacados de Nexus
  y se aplica tambien a las ventanas adicionales; se respeta una preferencia ya guardada.
- Las armas son **estimadas a partir de habilidades utilizadas**, no equipamiento confirmado:
  pueden faltar armas que no se usaron. No se inventan conjuntos ni manos.
- Se priorizan los lanzamientos reales de habilidades y sus paletas de profesion;
  se evita inferir armas desde proyectiles reflejados, condiciones, procs o ataques de mascotas.
  Sin lanzamientos disponibles (habitual en enemigos), la inferencia de golpes directos es conservadora:
  una reflexion de la misma profesion puede seguir siendo ambigua.
  Los cambios de conjunto 4/5 identifican tierra y 0/1 agua; una lanza terrestre no se pinta azul solo por ser lanza.
  Fuentes, atribucion y hashes de los 19 iconos: [nexusbase/assets/weapons/sources.json](nexusbase/assets/weapons/sources.json).
- Columnas numericas compactas, sin bordes: **dano, corrups y Down contribution** por defecto.
  La cabecera de esta ultima se abrevia **down c**; al pasar el raton muestra solo **Down contribution**.
  **Estadisticas → Columnas** y la pestana de opciones permiten combinar DPS, dano, corrups y Down contribution.
  Tooltip con las cuatro metricas, nombre completo y dano directo/condiciones.
  Clicar una cabecera numerica ordena por esa metrica; repetir el clic invierte el orden.
  **Estadisticas → Dano + corrups × 2000 + Down contribution** recupera el orden ponderado.
- **Down contribution** sigue la metodologia nativa de AxiBridge/AxiLog 1.16.0:
  cuenta dano a jugadores de otro color desde la ultima salud registrada >=99 % menos 2 s
  hasta el resultado de derribo, ambos limites inclusivos. Sin ancla usa el inicio del log;
  tras cada down, el siguiente intervalo empieza como minimo 2,1 s despues de ese down.
  Incluye condiciones y mascotas resueltas temporalmente, excluye CC/breakbar y no infiere
  downs solo desde cambios de estado. No hay intervalo configurable.
  Conservamos nuestros tres colores y filas por agente, por lo que esto no garantiza
  igualdad de totales con un informe de escuadron de AxiBridge; detalles en [calculos.md](calculos.md).
  El orden inicial sigue sumando esta contribucion sin multiplicador. Los tooltips no explican la formula.
- **Clic derecho → Copiar** conserva la lista completa y el orden actual; **Chat** copia la lista compacta del equipo seleccionado.
  Las listas largas pueden superar el limite de caracteres del chat de GW2.
- **Clic derecho → Abrir otra ventana** crea otra instancia con su propio equipo, columnas, posicion y tamano.
  La navegacion de combates y el seguimiento del ultimo log se comparten con las demas ventanas.

### Healing

- Tooltip de jugador compacto: conserva las cifras de limpias, sin la explicacion larga de retiradas al escuadron.
- Ranking de miembros identificados del grupo/escuadron, con **icono de clase** delante del nombre.
- **Sin recorte de filas**: muestra todos los jugadores con datos disponibles, con scroll al superar el alto de la ventana; no se anaden filas ficticias.
- **Una sola linea por jugador**: icono de clase, nombre y columnas de **curas, barrera y limpias**, las tres por defecto.
  **Estadisticas → Columnas** y las opciones permiten combinarlas o anadir el total de curas + barrera. Sin barras ni bordes.
- Orden inicial por **curas + barreras**. El menu de estadisticas conserva el selector de orden;
  clicar una cabecera numerica ordena por ella y repetir el clic invierte el orden.
- Tooltip con curas, barrera, limpias, total exacto, aportacion por segundo y subgrupo; copia desde el menu contextual.
- **Limpias** cuenta retiradas completas de condiciones realizadas sobre miembros registrados del escuadron,
  incluidas las propias: una por condicion, no por acumulacion. Se excluyen expiraciones, retiradas sinteticas y objetivos enemigos.
  Se atribuyen mascotas con propietario conocido y se respeta el subgrupo en el momento del evento.
  Es una cifra observada en EVTC normal: **no requiere Healing Stats** y permite mostrar jugadores con solo limpias,
  sin inventar curas o barreras. Tambien se puede ordenar por limpias.
- Aviso compacto **Parcial** en la cabecera con explicacion completa al pasar el raton.
- Tambien permite abrir ventanas adicionales desde el menu contextual, con historial compartido.

**Los logs EVTC normales no contienen por si solos todos los datos de healing.**
Se necesitan eventos compatibles de la extension de estadisticas de curacion de arcdps.
La interfaz distingue datos ausentes/incompatibles, grupo no identificable y datos parciales.
No presenta la ausencia de datos como cero ni como un ranking completo.
Las cifras registradas no deben interpretarse como curacion efectiva tras descontar sobrecuracion.
Solo compartir color no demuestra pertenencia al escuadron.
Se admiten las revisiones 1 y 2 de la extension con firma `0x9c9b3c99`
([protocolo](https://github.com/Krappa322/arcdps_healing_stats/blob/master/src/AddonVersion.h)).
Se exige pertenencia registrada al escuadron del POV tanto para quien aporta como para quien recibe.
Si el log no permite identificarla, no se inventa un grupo a partir del color.
La cobertura se considera conservadoramente parcial, nunca completa por ausencia de errores.
Una metrica sin eventos disponibles aparece como `-`. Las cifras compactas no llevan `>=`;
si falta una parte del total, la advertencia se conserva en el tooltip y en la copia detallada.
Al ordenar, los valores no disponibles quedan al final, no se tratan como ceros.
Se atribuyen mascotas con propietario identificable y se eliminan ecos identicos entre peers.
Los IDs/propietarios irresolubles se omiten; ticks genuinos indistinguibles de un eco pueden quedar
infracontados. La validacion con logs reales confirma la lectura, no equivalencia numerica con Elite Insights.

## Posicion, historial y controles

### Datos provisionales en directo (v0.2.13)

Con **Seguir el ultimo combate / Follow latest fight**, las tres ventanas principales y las adicionales
muestran el combate en curso, actualizado aproximadamente cada segundo y marcado **En directo · provisional**.
Requiere arcdps y **ArcDPS Integration** de Nexus, que publica los eventos locales/de escuadron y las altas/bajas
([bridge](https://github.com/RaidcoreGG/GW2-Arcdps-Integration)).
No consulta estadisticas acumuladas ni reconstruye eventos anteriores a la carga del addon.
Entrar en combate contra NPCs no inicia ni sustituye el directo y no activa
**Ocultar en combate**. Desde v0.2.18 basta con observar dano PvP de tu personaje, su mascota
identificada o un jugador de tu equipo/escuadron mientras tu estas en combate, aunque solo cures.
Una pelea ajena mientras tu estas fuera de combate no lo activa. Una vez detectado PvP, cuenta como combate
hasta salir del combate de GW2, aunque despues solo pegues a NPCs. Sin eventos del bridge
no se puede detectar PvP: no se ocultan ventanas por el estado generico de combate.
El flujo de escuadron alimenta las metricas; el local solo detecta el combate y aporta metadatos,
sin sumar dos veces sus eventos compartidos. Se reutilizan los calculos del parser EVTC.
El directo abre las ventanas con autoapertura en el primer refresco (aproximadamente un segundo),
sin esperar los 20 segundos de duracion minima, que solo se aplican a los logs.

Durante el directo de **Follow latest fight**, Resumen y Destacados muestran **solo nuestro equipo**,
sin pestanas ni filas de otros colores, tampoco en ventanas adicionales ni en el menu de color.
Los logs reales conservan los tres colores y seleccionan inicialmente el propio al cambiar de combate.
La antigua preferencia inicial de primer enemigo migra una vez a equipo propio; se puede volver a elegir
manualmente en opciones. Una seleccion manual de color se conserva durante los refrescos del mismo informe.
Desde v0.2.16 el dano/DPS, corrups y contribucion no dependen de recibir la correspondencia de colores:
usan los IDs de equipo observados o la afinidad hostil de arcdps contra jugadores identificados.
Si el color propio aun no se conoce, aparece **Nuestro equipo** en tono neutro, sin inventar un color.
Las identidades del bridge se actualizan al recibir informacion nueva, sin quedarse con un ID de equipo inicialmente ausente.
Curas/barreras solo si el bridge entrega
eventos y metadatos compatibles de Healing Stats; tener la extension instalada no garantiza recibirlos.
Las metricas ausentes siguen como `-`. El limite de seguridad es de 1.000.000 eventos por combate:
si se alcanza, aparece un aviso de directo incompleto y se espera al log, sin descartar silenciosamente el inicio.

Al salir de combate se conserva el informe como **Combate finalizado · provisional** en el
**historial de esta sesion**, con el mismo limite de combates que los logs y seleccionable como ellos.
**Seguir el ultimo combate** lo mantiene visible, sin volver al log anterior. Si llega un log que
cubre sus eventos, lo sustituye sin duplicarlo; tambien actualiza una seleccion fijada en ese provisional.
El siguiente combate empieza desde cero. Estos informes no se guardan en disco y se pierden al cerrar GW2.
Un log antiguo no borra un combate nuevo en curso. Elegir un log desactiva el directo
y conserva ese combate, sin mezclarlo con datos actuales. Sin eventos del bridge, sigue funcionando el seguimiento de logs.

Cada pestana guarda por separado activacion, visibilidad, apertura automatica, ocultacion en combate,
posicion **X/Y**, posicion fija, bloqueo de movimiento, ancho, alto, columnas y **opacidad**.
La seleccion del combate y el seguimiento del ultimo log son comunes a todas las ventanas.
Bloquear una ventana **no** bloquea sus botones. **Restablecer posicion** permite recuperarla;
las posiciones se ajustan al area visible tras cambiar de resolucion.

- **ALT+SHIFT+P**: mostrar/ocultar las tres ventanas principales activadas; atajo configurable en Nexus.
- **Clic derecho sobre la ventana o su titulo → Historial** muestra los logs, del mas reciente al mas antiguo, con fecha/hora
  y duracion. Seleccionar un combate lo muestra en **todas** las ventanas, incluidas las adicionales,
  y desactiva el seguimiento automatico para todas. El selector permanece abierto para elegir otros logs sin volver a abrirlo;
  **Seguir el ultimo combate** vuelve a activar el seguimiento globalmente.
- Menu contextual estilo arcdps: **Historial**, **Color** (Destacados), **Estadisticas**, **Estilo** y **Posicion**.
  No hay desplegable de logs, boton de tres puntos ni boton de colapsar ocupando espacio en las ventanas.
  Tampoco se muestra la fila «Historial: N combates cargados»; los errores y estados de analisis se conservan.
- **Estilo** ajusta ancho y opacidad; **Posicion** ofrece X/Y, bloqueo y recuperacion.
  Opacidad inicial del fondo y titulo: **49 %** Resumen, **38 %** Destacados y **32 %** Healing, sin atenuar texto ni iconos.
- Arrastrar la **esquina inferior derecha** redimensiona cualquiera de las ventanas y guarda su ancho y alto.
  El bloqueo de movimiento o la posicion fija no impiden redimensionar. Restablecer estilo recupera el alto compacto inicial.
- **Analizar ultimo log** y **Reprocesar ultimos N combates** estan en Historial y en opciones.
- Las opciones compartidas de analisis/historial estan debajo de las tres pestanas.
- Todas las ventanas solo aparecen en mapas WvW, incluidas salas y mapas auxiliares.
  Se ocultan durante pantallas de carga y seleccion de personaje usando el estado de gameplay de Nexus,
  tambien las ventanas adicionales; reaparecen al volver al juego sin cambiar preferencias ni informes.
  Fuera de WvW se mantienen datos y preferencias, y siguen accesibles las opciones de Nexus.

| Ajuste | Por defecto |
|---|---|
| Ventanas habilitadas / visibles | Las tres |
| Apertura automatica / ocultar en combate | Si / No, por ventana |
| Posicion fija / bloquear movimiento | No, por ventana |
| Ancho Resumen / Destacados / Healing | 259 / 300 / 234 px |
| Alto Resumen / Destacados / Healing | 78 / 112 / 116 px |
| Opacidad Resumen / Destacados / Healing | 49 / 38 / 32 % |
| Posicion X/Y Resumen / Destacados / Healing | 391,1 / 233,624 / 0,624 |
| Seguir el ultimo combate | No |
| Filas Healing | Todas las disponibles, con scroll |
| Destacados | Con dano o corrups; dano + corrups × 2000 + Down contribution |
| Down contribution | Metodologia nativa AxiLog; sin intervalo configurable |
| Columnas Destacados | Dano + corrups + Down contribution |
| Columnas Healing | Curas + barrera + limpias |
| Historial | 10 combates (1–20) |
| Duracion minima para apertura automatica de logs (no del directo) | 20 s |
| Dano y strips solo sobre jugadores | Si |
| Carpeta de logs | La de arcdps |

Los ajustes siguen en `Documentos\Guild Wars 2\addons\fight_analysis_danger\settings.txt`.
Se migran las preferencias antiguas de visibilidad, apertura y ocultacion sin tocar otros ajustes.
Los anteriores anchos predeterminados (380/480/520 px) pasan a 175 px; otros anchos personalizados
se conservan. **Estilo → Tamano y opacidad por defecto** recupera el nuevo aspecto sin borrar ajustes.

Los valores iniciales se han actualizado desde la configuracion del juego del 22/10/2026, incluidos tamanos,
columnas, opacidad y visibilidad en combate cambiados desde el menu derecho, pero sin bloqueo de movimiento ni posicion fija.
Solo se aplican a nuevas instalaciones o ajustes ausentes; no sustituyen preferencias ya guardadas.
**Habilitar ventana** permite usarla y abrirla automaticamente; **Visible ahora** solo controla si se muestra en ese momento.

La fuente unica de valores iniciales es [settings.default.json](settings.default.json), incluida en la DLL
al compilar. `windows` contiene, por orden, Resumen, Destacados y Healing. `columns` es una mascara:
1 = DPS/total, 2 = dano/curas, 4 = strips/barrera, 8 = Down contribution/limpias.
Destacados y Healing usan 14 por defecto. Al leer ajustes anteriores a `layout_version=3`,
se anade la columna limpias una sola vez; despues se puede ocultar y se conserva esa eleccion.
Los anteriores a `layout_version=4` incorporan Down contribution en Destacados una sola vez,
tambien ocultable. Desde v0.2.10 se eliminan `top`, `top_dps`, `top_strips`
y `down_contribution_secs`: se ignoran al cargar archivos antiguos y no se vuelven a escribir
al guardar. Se conservan posiciones, tamanos, opacidades y otras preferencias.
Desde v0.2.13 tambien se ignora y deja de guardar `healing_rows`: ya no limita el ranking ni la copia.
El ejemplo CLI mantiene su argumento independiente de limite de filas; no es un ajuste del addon.
Editar ese JSON requiere recompilar; no se copia sobre el archivo personal ni cambia el formato `clave=valor`
de los ajustes instalados. La deserializacion tipada usa `serde`/`serde_json` para validar el JSON sin implementar
un parser propio; los tests comprueban campos, limites y compatibilidad con ajustes existentes.
Las ventanas adicionales son temporales; la configuracion persistente corresponde a las tres principales.

## Idioma y limitaciones

Interfaz, tooltips, estados, errores y resumenes en **espanol, ingles, frances y aleman**.
El idioma se detecta desde RealTime de Nexus, Unofficial Extras, el idioma del log mas reciente
y, como respaldo, Nexus/ingles. Consultar un log antiguo no cambia el idioma.
Los nombres oficiales y los iconos de profesiones/especializaciones estan incluidos en la DLL.

**Mantener Ctrl derecho** (a la derecha de la barra espaciadora) muestra temporalmente
todo el addon en ingles: ventanas principales/adicionales, menus, opciones, tooltips
y textos copiados mientras esta pulsado. Al soltarlo se recupera el idioma automatico;
si ya era ingles, no cambia. Solo se aplica con el juego en primer plano.
No guarda ajustes, reprocesa logs ni bloquea la tecla para el juego. Ctrl izquierdo no activa esta funcion.
Nombres propios de personajes/cuentas y nombres de archivos no se traducen.

WvW oculta nombre, cuenta y guild de enemigos: se muestra el rango que llega en el log.
El DPS incluye dano directo y condiciones de jugadores y sus mascotas/clones.
Los strips cuentan retiradas completas de boons, incluido el formato actual de arcdps (statechange 72).
El filtro de objetivos de dano/strips no altera las kills, muertes y derribos de jugadores.

**EN:** Independent team overview, player highlights and squad healing windows.
**FR :** Fenetres independantes de resume, joueurs remarquables et soins d'escouade.
**DE:** Unabhangige Fenster fur Teamubersicht, hervorgehobene Spieler und Einsatztruppheilung.

## Requisitos y funcionamiento

- [Nexus](https://raidcore.gg/Nexus) y [arcdps](https://www.deltaconnected.com/arcdps/),
  con grabacion de logs WvW activada. La extension de healing es necesaria para aportar esos datos.
- WvW Fight Analysis es opcional: su evento `EV_LOG_PARSED` acelera la deteccion.
  Como respaldo se vigila cada 3 s la carpeta de logs.
- Parser EVTC y descompresion ZIP propios, sin nuevas dependencias de terceros.
- Carpeta: `boss_encounter_path` de `arcdps.ini`, o
  `Documentos\Guild Wars 2\addons\arcdps\arcdps.cbtlogs`, incluidas subcarpetas.

## Instalacion y actualizacion

Descarga directa:
[fight_analysis_danger.dll v0.2.17](https://github.com/cbart-cmd/gw2addonsreleases/releases/download/fight-analysis-danger-v0.2.17/fight_analysis_danger.dll).
No uses el enlace "latest" del repositorio compartido: tambien distribuye Timers.

Para verificar esa descarga en PowerShell:

```powershell
Get-FileHash .\fight_analysis_danger.dll -Algorithm SHA256
```

SHA-256 de la DLL publicada v0.2.17 (1.142.272 bytes):
`5c566a574b9dff2e17b03813ca3de726560741d657ac03be63db5e2ac9deadde`.
Este hash corresponde al binario publicado, no a cualquier recompilacion local.
MD5 se conserva para compatibilidad con Nexus; no es una firma de autenticidad.

Si compilas desde el codigo, la DLL queda en `target\release\fight_analysis_danger.dll`.
Copiala a `addons` del juego, o ejecuta:

```powershell
.\install.ps1
.\install.ps1 -Build   # tests, compilacion e instalacion
```

Tambien se puede usar `install.bat`. En Nexus (Ctrl+O) cargar **WvW Combat Insights**.
No instalar una segunda DLL con otro nombre encima de la anterior.

Desde v0.2.19, la autoactualizacion `Direct` usa
[`raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/fight_analysis_danger.dll`](https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/fight_analysis_danger.dll)
y su `.md5` (32 caracteres hexadecimales en minusculas, sin BOM ni salto de linea).
Se conserva repositorio, rama, nombre de DLL, firma, identidad y ajustes.
El cliente HTTP de Nexus 2026.2.17.1210 puede fallar con `Failed to read connection`
en la URL anterior de `github.com/.../raw/...`. Las DLL antiguas conservan esa URL:
si no pueden actualizarse, requieren sustituir manualmente una vez la DLL de la
release, con GW2 cerrado. No copiar al juego sin autorizacion ni con GW2 abierto.
La distribucion excluye esta DLL de Git LFS y publica tambien DLL/MD5 como assets de release.
Si una instalacion antigua quedo sustituida por un puntero LFS de unos 130 bytes, cerrar GW2 y
reemplazar manualmente solo la DLL por el binario verificado de la release; no borrar ajustes.

## Desarrollo

Windows x64, Rust estable con rustfmt, Visual Studio Build Tools (Desktop
development with C++) y Windows SDK. No se necesitan repositorios hermanos.
Los generadores de datos son opcionales y solo ellos necesitan consultar la API.

```powershell
cargo fmt --all -- --check
cargo test --locked --all-targets
cargo build --locked --release
python tools\collect_licenses.py --test
python tools\collect_licenses.py --check
cargo run --release --example analyze -- [ruta\log.zevtc] [top] [es|en|fr|de]
python tools\update_game_names.py
python tools\update_weapon_data.py   # catalogo de habilidades e iconos oficiales
```

Modulos: `evtc.rs`/`healing.rs` (parser), `danger.rs` (ranking), `zip.rs`/`inflate.rs` (descompresion),
`logs.rs`, `settings.rs`, `app.rs`, `ui.rs`, `weapon_icons.rs`, `lib.rs`, `i18n.rs` y `game_names.rs`.

CI en Windows ejecuta formato, tests, comprobacion de licencias y build release.
Sus artefactos incluyen DLL, MD5/SHA-256, licencias y un archivo del codigo fuente
del mismo commit. Tiene permisos de solo lectura y no publica actualizaciones.

## Soporte, privacidad y licencia

Para errores, indica versiones, pasos y si ocurre en directo o con un log guardado.
No publiques logs EVTC/ZIP sin anonimizar: pueden incluir cuentas y personajes.
Recorta capturas y elimina rutas personales; comprimir un log no lo anonimiza.
Usa ejemplos sinteticos cuando sea posible. Consulta [CONTRIBUTING.md](CONTRIBUTING.md)
y comunica vulnerabilidades de forma privada siguiendo [SECURITY.md](SECURITY.md).

El codigo propio se distribuye bajo **GNU GPL version 3 o posterior**, sin garantia:
[LICENSE](LICENSE), con [permiso limitado de integracion](ADDITIONAL-PERMISSION.md)
para GW2/Nexus/arcdps. Los recursos de Guild Wars 2 y las dependencias mantienen
sus condiciones originales; no se convierten en GPL por estar incluidos aqui.
Consulta [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) y los textos completos
en [THIRD-PARTY-LICENSES.txt](THIRD-PARTY-LICENSES.txt).
Proyecto comunitario no oficial, sin afiliacion, patrocinio ni aprobacion de ArenaNet o NCSOFT.
