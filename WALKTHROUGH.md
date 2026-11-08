# Recorrido funcional del código fuente

Esta guía explica, con palabras sencillas, cómo está organizado WvW Combat
Insights, qué ocurre al cargarlo en Nexus y qué aporta cada archivo de código.
No intenta describir cada detalle del formato EVTC ni sustituye a
[calculos.md](calculos.md), que explica las definiciones de las métricas.

## Mapa de carpetas

```text
addon.toml                 Identidad del addon y URL de actualización
Cargo.toml / Cargo.lock    Versión, dependencias y configuración de Rust
settings.default.json      Valores iniciales de las preferencias
src/                       Análisis, datos de combate, ajustes y ventanas
nexusbase/                 Conexión reutilizable con Nexus, ArcDPS y Windows
  gw2/                     Tablas de nombres, habilidades e iconos de GW2
  assets/                  Imágenes incluidas en la DLL
examples/                  Programa de consola para analizar un log
tools/                     Generadores de datos y scripts de publicacion
elitei/                    Copias de referencia del codigo de Elite Insights
target/                    Resultados temporales de compilación (no es código fuente)
install.ps1 / install.bat  Ayudantes para compilar e instalar la DLL
README.md / readme.html     Guías de uso e instalación
calculos.md                Definiciones y límites de las métricas
PROGRESO.md                Historial de cambios y verificaciones
```

`src` contiene la lógica propia del addon. `nexusbase` agrupa las piezas que
sirven para comunicarse con Nexus, ArcDPS y Windows, además de algunos datos
oficiales de Guild Wars 2. No es otro programa que haya que instalar.

## Qué ocurre al cargar el addon

1. **Cargo prepara la DLL.** Al compilar, Cargo ejecuta
   [nexusbase/build.rs](nexusbase/build.rs). Este lee
   [addon.toml](addon.toml), comprueba sus valores y genera dos archivos
   temporales: las constantes de identidad y la exportación que Nexus espera.
   Esos archivos se escriben bajo `target`; no se editan a mano.
2. **Nexus reconoce la DLL.** [src/lib.rs](src/lib.rs) declara los módulos del
   addon e incorpora la exportación generada. Esa exportación proporciona a
   Nexus `GetAddonDef`, con nombre, firma, versión, proveedor y URL de
   actualización.
3. **Nexus llama a `nexusbase/lifecycle.rs`.** La función `load` inicia el
   idioma, arranca el estado y el hilo de análisis de
   [src/app.rs](src/app.rs), registra los eventos de combate de
   [src/live.rs](src/live.rs), el atajo de teclado y los callbacks de dibujo
   de ventanas y opciones de [src/ui.rs](src/ui.rs).
4. **Se cargan las preferencias.** `app::start` usa los ajustes de
   [src/settings.rs](src/settings.rs). Al iniciar, activa y guarda
   `follow_latest` en las tres ventanas, incluso si antes estaba apagado.
   Durante esa sesión se puede desactivar desde el menú; el siguiente inicio
   lo activa otra vez.
5. **El addon escucha dos caminos de datos.**
   - Los eventos de ArcDPS alimentan el combate provisional en vivo. El bridge
     de ArcDPS Integration de Nexus debe estar entregando eventos para que esta
     parte reciba actividad.
   - Los logs `.zevtc` de arcdps se buscan en segundo plano. El evento de WvW
     Fight Analysis puede avisar que terminó un log; si no está disponible, el
     addon comprueba periódicamente la carpeta.
6. **Los datos llegan a las ventanas.** `app` conserva los informes y su
   historial en memoria. `live` actualiza el informe provisional. En cada
   render, `ui` elige el informe que debe mostrar y pinta Resumen, Destacados
   y Healing. Los informes en memoria no se guardan como logs propios.
7. **Al descargar el addon**, `nexusbase/lifecycle.rs` llama a `app::stop`
   para cerrar el hilo de análisis; las suscripciones registradas se revierten
   al descargar.

## Camino de un log y de sus boons

Cuando hay un `.zevtc`, el hilo de `app` lo abre y llama a
`zip::first_entry` para extraer el EVTC contenido dentro. Si hace falta,
`inflate::inflate` descomprime el flujo ZIP. Después, `evtc::parse` prepara la
lista de jugadores, equipos y eventos y calcula las métricas.

Para los **boons**, el EVTC no ofrece una cifra ya lista de “corrups”. El parser
recorre los eventos de retirada y reconoce una lista concreta de boons en
`evtc::is_tracked_boon`. Solo cuenta retiradas completas que cumplen las
condiciones del formato y que puede atribuir a un jugador aliado. Si está
activada la preferencia de contar solo contra jugadores, también exige que el
objetivo sea un jugador del equipo opuesto. Cada retirada válida incrementa
`strips` del jugador que la hizo; después la UI presenta esos strips como
corrups. La preferencia de ese filtro se conserva sin cambios por la corrección
de `follow_latest`.

Las **limpias** son otra métrica: `healing.rs` las obtiene de eventos de
condiciones eliminadas en miembros del grupo/escuadrón. No son boon strips.
El daño, las kills, las muertes, los derribos, la atribución de mascotas y el
down contribution se calculan en el parser y en
[src/contribution.rs](src/contribution.rs), con las definiciones completas en
[calculos.md](calculos.md).

## Archivos de código de `src/`

| Archivo | Qué hace |
|---|---|
| [src/lib.rs](src/lib.rs) | Es el punto donde Rust declara los módulos del addon e incorpora la información generada para que Nexus pueda cargar la DLL. No es una aplicación de consola ni contiene `main`. |
| [src/app.rs](src/app.rs) | Mantiene el estado compartido: ajustes, informes, historial y estado de carga. Inicia el hilo que vigila los logs, responde a peticiones de analizar/reprocesar y convierte cada log en un informe. Al arrancar fuerza `follow_latest`; al parar, cierra el hilo. |
| [src/live.rs](src/live.rs) | Se suscribe a los eventos de ArcDPS. Mantiene el roster recibido, detecta el inicio y fin del combate PvP pertinente, acumula eventos y produce informes provisionales en vivo. Conserva combates finalizados en el historial de la sesión hasta que llega el log real. |
| [src/logs.rs](src/logs.rs) | Encuentra la carpeta de logs usando el ajuste del usuario, `arcdps.ini` o la ruta habitual. Recorre subcarpetas y localiza logs recientes o un log cuyo nombre le dieron. |
| [src/zip.rs](src/zip.rs) | Abre el `.zevtc`, que normalmente es un ZIP con un EVTC dentro, y entrega el contenido descomprimido al parser. También acepta un EVTC sin ZIP. |
| [src/inflate.rs](src/inflate.rs) | Implementa la descompresión DEFLATE que usa `zip.rs`, con un límite de tamaño para evitar expansiones desmedidas. |
| [src/evtc.rs](src/evtc.rs) | Es el parser y analizador principal. Interpreta los registros del EVTC, identifica jugadores y equipos, suma daño, strips, kills, muertes y derribos, y reúne la cobertura de healing. El daño absorbido por barrera no cuenta como daño ni como contribución al derribo. También prepara informes a partir de los eventos en vivo. |
| [src/healing.rs](src/healing.rs) | Lee los eventos de la extensión Healing Stats que vienen en el log. Calcula healing, barrera y limpias. Como Elite Insights, suma toda cura a un objetivo aliado (byte IFF amigo), sin exigir escuadrón, y marca la cobertura parcial para no presentar datos ausentes como ceros. |
| [src/contribution.rs](src/contribution.rs) | Calcula cuánto daño contribuye cada jugador a derribar a un enemigo, según la metodología descrita en `calculos.md`. Resuelve mascotas y cambios de identidad usando el momento del evento. |
| [src/danger.rs](src/danger.rs) | Ordena y selecciona jugadores destacados por equipo. Prepara resúmenes de texto y líneas compactas que se pueden copiar o pegar en el chat. |
| [src/ui.rs](src/ui.rs) | Dibuja las tres ventanas, sus filas, pestañas, menús contextuales, opciones y ventanas adicionales. Decide qué combate y equipo mostrar y envía las acciones de navegación a `app` o al espacio de trabajo de la UI. |
| [src/settings.rs](src/settings.rs) | Define los ajustes, los valores iniciales, el formato `clave=valor`, las migraciones y la lectura/escritura del archivo personal de configuración. |
| [src/i18n.rs](src/i18n.rs) | Une la selección de idioma con los textos de la interfaz y traduce nombres, métricas, estados, duraciones y fechas de informes. |

## Archivos de `nexusbase/`

| Archivo | Qué hace |
|---|---|
| [nexusbase/mod.rs](nexusbase/mod.rs) | Declara las piezas de infraestructura disponibles y expone la identidad generada durante la compilación. |
| [nexusbase/lifecycle.rs](nexusbase/lifecycle.rs) | Conecta Nexus con el addon: carga y descarga, inicializa el idioma, inicia y detiene `app`, conecta los eventos de ArcDPS, y registra dibujo, opciones y atajo. |
| [nexusbase/build.rs](nexusbase/build.rs) | Se ejecuta al compilar, valida `addon.toml` y crea los archivos temporales de identidad y exportación de Nexus. |
| [nexusbase/project_config.rs](nexusbase/project_config.rs) | Define qué campos debe tener `addon.toml`, valida la identidad del proyecto y contiene pruebas para ese contrato. |
| [nexusbase/arc.rs](nexusbase/arc.rs) | Proporciona suscripciones tipadas a eventos de ArcDPS y permite pedir la repetición del roster cuando el bridge lo ofrece. Transporta eventos; su significado de combate se interpreta en `src/live.rs`. |
| [nexusbase/language.rs](nexusbase/language.rs) | Detecta el idioma del juego y coordina las fuentes de idioma de Nexus, Unofficial Extras y el log, con sus alternativas cuando falta alguna fuente. |
| [nexusbase/paths.rs](nexusbase/paths.rs) | Encuentra Documentos de Windows y construye las rutas persistentes del addon y de sus ajustes, respetando la carpeta Documentos redirigida. |
| [nexusbase/gw2/mod.rs](nexusbase/gw2/mod.rs) | Agrupa los catálogos oficiales de nombres, habilidades e iconos de GW2. |
| [nexusbase/gw2/game_names.rs](nexusbase/gw2/game_names.rs) | Tabla generada de nombres de profesiones, especializaciones y rangos WvW en español, inglés, francés y alemán. |
| [nexusbase/gw2/weapon_skills.rs](nexusbase/gw2/weapon_skills.rs) | Catálogo generado que permite relacionar una habilidad usada en el combate con un arma posible. `evtc.rs` lo consulta al inferir armas; no afirma que sean todo el equipo del jugador. |
| [nexusbase/gw2/weapon_icons.rs](nexusbase/gw2/weapon_icons.rs) | Relaciona tipos de arma con sus imágenes incluidas y con la información de uso acuático. |
| [nexusbase/gw2/specialization_icons.rs](nexusbase/gw2/specialization_icons.rs) | Relaciona la profesión y especialización de un jugador con el icono que se pinta en la UI. |

### Recursos de `nexusbase/assets/`

No son archivos de lógica, sino imágenes empaquetadas en la DLL:

- `professions/`: iconos de las nueve profesiones.
- `specializations/`: iconos de especializaciones de élite.
- `weapons/`: imágenes de los tipos de arma.
- `weapons/sources.json`: procedencia y datos de atribución de las imágenes de armas.

Los archivos que enlazan esos recursos con el código son los módulos de
`nexusbase/gw2/`. El addon no tiene que descargarlos mientras se juega.

## Herramientas, instalación y pruebas

| Archivo | Qué hace |
|---|---|
| [examples/analyze.rs](examples/analyze.rs) | Programa de consola para analizar un log sin abrir GW2. Usa las mismas piezas públicas de logs, ZIP, EVTC y ranking que el addon. Sirve para revisar un `.zevtc` y mostrar un resumen. |
| [tools/update_game_names.py](tools/update_game_names.py) | Consulta la API pública de GW2 y regenera `nexusbase/gw2/game_names.rs` con nombres localizados. Es una herramienta de mantenimiento, no se ejecuta dentro del juego. |
| [tools/update_weapon_data.py](tools/update_weapon_data.py) | Regenera el catálogo de habilidades relacionadas con armas y descarga/actualiza los iconos oficiales. Es una tarea de desarrollo; el resultado queda en `nexusbase/gw2/` y `nexusbase/assets/`. |
| [tools/publish.ps1](tools/publish.ps1) | Publica una versión con todos los pasos: formato, tests y build release, `GetAddonDef`, copia de la DLL y su MD5 al repo de distribución, comprobación de `.gitattributes` y del blob preparado, commits y push en ambos repos, release con `--latest=false`, dejar solo las dos releases de este addon y verificar las cuatro URL de descarga. |
| [tools/verify-release.ps1](tools/verify-release.ps1) | Comprobación de solo lectura: `GetAddonDef` de la DLL local y descarga de la URL exacta, la antigua, el commit de distribución y el asset de la release para comparar tamaño, MD5 y `MZ`, junto con los dos sidecars. |
| [install.ps1](install.ps1) | Opcionalmente ejecuta tests y build release y copia la DLL a la carpeta `addons` indicada. Se detiene si el juego mantiene bloqueada la DLL. |
| [install.bat](install.bat) | Abre `install.ps1` con PowerShell, útil para iniciar el instalador con doble clic. |
| [Cargo.toml](Cargo.toml) | Declara la versión, dependencias, configuración de compilación y que el proyecto produce una DLL (`cdylib`) y una biblioteca reutilizable (`rlib`). |
| [Cargo.lock](Cargo.lock) | Fija las versiones exactas de dependencias para repetir la compilación. |
| [settings.default.json](settings.default.json) | Fuente de los valores iniciales de ventanas y preferencias; se incorpora al compilar y no es el archivo personal que se edita mientras se juega. |

## Referencias de Elite Insights

[elitei/](elitei/README.md) guarda copias del código de Elite Insights usadas para fijar
reglas de cálculo (`IFF.Friend` para curas, `HealthDamage` sin barrera, retiradas completas
de boons y condiciones, contribución al derribo). No se compila ni forma parte del addon.
[elitei/fetch-ei.ps1](elitei/fetch-ei.ps1) vuelve a descargar esos archivos de `master`.

## Publicacion de una version

`./tools/publish.ps1 -Bump patch` hace la secuencia completa; `./tools/verify-release.ps1`
comprueba después lo publicado sin tocar nada. No copian la DLL al juego: eso es manual y
con GW2 cerrado.

Los tests están junto al código dentro de bloques `#[cfg(test)]`. Comprueban
partes concretas como el parser, los ajustes, el ranking, la UI y el contrato
de proyecto. `target/` guarda el resultado generado al compilar y probar; se
puede regenerar con Cargo y no es el lugar donde se edita la lógica.
