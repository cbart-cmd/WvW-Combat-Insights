# Estado del snapshot publico

- Version funcional: 0.2.17, sin cambios de comportamiento respecto al addon distribuido.
- Snapshot independiente, sin historial anterior ni workspace de otros proyectos.
- GPL-3.0-or-later con permiso limitado de integracion; atribuciones y textos de terceros incluidos.
- README Markdown/HTML, soporte privado de seguridad y plantilla de errores preparados.
- CI Windows de solo lectura: formato, tests, licencias, release y artefactos con fuente/checksums.
- Validacion local: formato correcto, 92 tests Rust y self-check del recopilador correctos; DLL release compilada.
- Distribucion existente verificada: asset release y ambos endpoints de actualizacion coinciden en MZ, tamano y SHA-256.
- Gitleaks local no detecto secretos en el snapshot; no sustituye una auditoria de seguridad.
- El resultado remoto de CI se consulta en Actions. No se ha realizado una prueba manual dentro de GW2.
