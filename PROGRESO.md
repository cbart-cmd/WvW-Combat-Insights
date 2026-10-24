# Estado del snapshot publico

## v0.2.19

- Endpoint Direct incorporado cambiado a raw.githubusercontent.com; firma, identidad y ajustes conservados.
- Regresion fija endpoint/proveedor. Reglas persistentes de GetAddonDef, MD5 exacto, LFS, cliente Nexus y verificacion de caches.
- Versiones instaladas con URL anterior pueden necesitar una sustitucion manual autorizada con GW2 cerrado; no se instala en el juego.
- Formato, 94 tests unitarios y 2 de configuracion, self-check de licencias e inventario correctos; compilacion release independiente.
- GetAddonDef real v0.2.19 verificado: firma -1178682436, proveedor Direct (3), nombre y URL directa correctos.
- Cliente original de Nexus reutilizable preparado fuera del juego; la verificacion remota se realiza tras publicar DLL/MD5, fuente y licencias.

## v0.2.18

- Directo por PvP propio/equipo mientras el POV esta en combate, tambien para quien solo cura.
- Autoapertura en el primer refresco, sin el umbral de duracion de logs.
- Combates finalizados en historial de sesion, incluidos cambios de mapa y combates entre refrescos; su log real los sustituye. Sin guardar provisionales en disco.
- Equipo propio inicial al cambiar de informe, preservando colores manuales durante refrescos; migracion unica del antiguo valor enemigo.
- Validacion local: 94 tests unitarios y 2 de configuracion correctos, formato y DLL release; pendiente comprobacion dentro de GW2.

## Snapshot inicial v0.2.17

- Version funcional inicial: 0.2.17.
- Snapshot independiente, sin historial anterior ni workspace de otros proyectos.
- GPL-3.0-or-later con permiso limitado de integracion; atribuciones y textos de terceros incluidos.
- README Markdown/HTML, soporte privado de seguridad y plantilla de errores preparados.
- CI Windows de solo lectura: formato, tests, licencias, release y artefactos con fuente/checksums.
- Validacion local: formato correcto, 92 tests Rust y self-check del recopilador correctos; DLL release compilada.
- Distribucion existente verificada: asset release y ambos endpoints de actualizacion coinciden en MZ, tamano y SHA-256.
- Gitleaks local no detecto secretos en el snapshot; no sustituye una auditoria de seguridad.
- El resultado remoto de CI se consulta en Actions. No se ha realizado una prueba manual dentro de GW2.
