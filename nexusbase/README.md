# Nexus addon base (v1)

Contrato de estructura para estos addons y para futuros agentes de IA. Cada
repositorio contiene su propia copia versionada de `nexusbase`: debe compilar
sin acceder a un checkout hermano. No es un framework ni una dependencia
remota adicional.

## Estructura

```text
addon.toml                 Identidad y configuracion de compilacion del addon
Cargo.toml                 Version, dependencias, capacidades y tipo de DLL
nexusbase/
  build.rs                 Valida addon.toml y genera constantes y Nexus export
  project_config.rs        Esquema y pruebas de configuracion de proyecto
  mod.rs                   Solo los modulos que este addon utiliza
  lifecycle.rs             Adaptador load/unload: conecta Nexus con src
  arc.rs                   Suscripciones tipadas y replay del roster
  language.rs              Idioma GW2/RTAPI, Extras, fallback y plantillas
  paths.rs                 Documentos de Windows y almacenamiento del addon
  platform.rs              Opcional: renderer, texturas, Mumble y swap chain
  download.rs              Opcional: transporte HTTPS de Windows
  gw2/                     Datos, nombres y servicios oficiales reutilizables
  assets/                  Recursos oficiales incorporados, si se usan
src/
  lib.rs                   Declara modulos e incluye el export generado
  i18n.rs                  Textos del addon y presentacion especifica
  ...                      Funciones y estado exclusivos de este addon
tools/                     Generadores offline; escriben en nexusbase/gw2 y assets
```

Un modulo opcional solo se copia si existe un consumidor real. Timers mantiene
en `src/boons` su reconocimiento de pantalla, plantillas OCR y mascaras; no son
infraestructura GW2. Combat Insights mantiene en `src` el parser EVTC, el
analisis y las sesiones live. Sus tablas oficiales de habilidades, profesiones
e iconos estan en `nexusbase/gw2` y `nexusbase/assets`. Los nombres oficiales de
Timers estan en `nexusbase/gw2/game_names.json`; sus textos propios siguen en
`src/i18n/translations.rs`. No convertir datos oficiales en traducciones
inventadas ni eliminar su procedencia/licencia.

## Tres clases distintas de configuracion

1. **Proyecto: `addon.toml`.** `name`, `signature`, `update_link`,
   `language_key`, `data_directory` y `settings_file`. Se validan al compilar
   y se incorporan en la DLL; no hay que distribuir un TOML junto a ella.
   La version solo se cambia en `Cargo.toml`. No editar archivos en `OUT_DIR`.
   El unico parser TOML es una dependencia de compilacion/pruebas, ya utilizada
   por la cadena de herramientas; no se anade un parser en tiempo de ejecucion.
2. **Valores iniciales de funciones.** Conservar los JSON existentes de cada
   addon (`settings.default.json` o `defaults.json` y `digits.json`).
   Parametros de OCR, audio, calculos y limites de seguridad no se convierten
   en configuracion de Nexus por estar escritos como constantes.
3. **Preferencias del usuario.** Conservar rutas, formatos, migraciones,
   identificadores de ventanas y claves guardadas. Cambiar el directorio del
   codigo NO autoriza mover, sobrescribir ni reiniciar ajustes instalados.

En un addon existente no cambiar firma, URL de actualizacion ni directorio de
datos como parte de una refactorizacion. En uno nuevo elegir valores unicos y
actualizar la expectativa de identidad en `project_config.rs`.

## Reglas de dependencia

- `src` consume `nexusbase`, no al reves. La unica excepcion es
  `lifecycle.rs`: es el adaptador explicito de registro y parada del addon.
- Utilizar `arc::subscribe` para registrar callbacks con
  `revert_on_unload`. El significado de los eventos pertenece al addon.
  Mantener los contratos FFI, los chequeos de punteros y el orden de descarga.
- `language::initialize` registra la deteccion y el evento de Extras una sola
  vez por carga. Prioridad: idioma del juego, Extras, log si existe, Nexus,
  ingles. Las traducciones y excepciones de UX propias permanecen en `src`.
- Conservar las API publicas existentes mediante reexports cuando se extraen
  helpers. No crear wrappers o una segunda API para llamadas que ya ofrece
  `nexus-rs`.
- Cada modulo puede usar las dependencias ya presentes que necesita; no copiar
  todos los features de Windows ni todos los recursos a cada addon.

## Cambios y nuevos proyectos

1. Copiar el nucleo `build.rs`, `project_config.rs`, `mod.rs`, `arc.rs`,
   `language.rs` y `paths.rs`, mas este contrato, desde un addon validado.
2. Registrar `build = "nexusbase/build.rs"` en el paquete y `toml_edit` con
   solo `parse` como dependencia de build y dev. Registrar el target de pruebas
   `nexusbase_project` con `path = "nexusbase/project_config.rs"`.
3. Crear `addon.toml`, un adaptador `lifecycle.rs` y un `src/lib.rs` que use
   `#[path = "..\\nexusbase\\mod.rs"] mod nexusbase;` e
   `include!(concat!(env!("OUT_DIR"), "/export.rs"));`.
4. Copiar solo las capacidades con consumidores y conservar la procedencia de
   datos. Los generadores deben actualizar las rutas nuevas, no recrear
   archivos obsoletos en `src`.
5. Al corregir infraestructura comun, aplicar el mismo arreglo a los addons
   que tengan ese modulo y ejecutar sus pruebas. Las copias NO se sincronizan
   automaticamente; no sobrescribir adaptadores ni capacidades especificas.
6. Ejecutar `cargo fmt --all`, `cargo test --all-targets` y
   `cargo build --release` en cada repositorio afectado. Revisar la ausencia
   de cambios accidentales y probar dentro de GW2 cuando sea posible.

## Publicacion

Actualizar README.md, readme.html y progreso antes del commit. Incrementar la
version, probar y compilar cada DLL. Publicar DLL y MD5 en el repositorio de
distribucion y en GitHub Releases siguiendo las instrucciones del repositorio.
Las excepciones de Git LFS deben producir un blob binario real del mismo tamano
que la DLL local. Verificar los bytes `MZ`, tamano y MD5 del asset y de la URL
exacta de `update_link` en `addon.toml`, ademas de las URL antiguas soportadas.
Un HTTP 200 o una URL con cache-buster no demuestran una actualizacion correcta.
No cambiar endpoints para ocultar un problema de cache.
