Tengo una duda, en el punto ## Fase 6 — Sync filesystem local (primer provider), no entiendo la de sincronización local ¿ se refiere a la sincronización de notas entre dos instancias de la app ?

Sí, exactamente. La idea es que la app guarda el vault (notas cifradas) en `$APP_DATA/vault/`, que es una carpeta local del sistema. La sincronización consiste en copiar ese vault a **otra carpeta** que tú eliges — por ejemplo una carpeta de Dropbox, Google Drive, o un NAS montado como unidad local.

El flujo sería:

1. Tú configuras un "sync target": `/Users/guille/Dropbox/vsyncnotes-vault/`
2. La app copia los archivos cifrados allí (push)
3. En otro Mac, instalas vsyncnotes y apuntas al mismo directorio de Dropbox
4. Esa segunda instancia detecta los archivos nuevos y los descarga (pull)

**No es sync en tiempo real** — es más bien como Joplin lo hace: un botón "Sync" o un intervalo periódico que compara timestamps y decide qué subir/bajar.

La ventaja de este enfoque es que **la app no necesita integración directa con ningún proveedor cloud** — solo usa el filesystem. Si tienes Dropbox instalado, su cliente ya se encarga de sincronizar esa carpeta entre máquinas. vsyncnotes solo ve archivos locales.

En resumen: **sí, es sincronización entre dos instancias**, pero indirecta, a través de una carpeta compartida (Dropbox, iCloud Drive, Google Drive, NAS, etc.).

ya, eso lo entiendo, pero vamos a ir por pasos y la primera fase es "sync local", ¿ se refiere al sync entre 2 instancias de la app en el mismo equipo ?

No, "sync local" no es entre dos instancias en el mismo equipo — eso no tendría sentido práctico.

"Sync filesystem local" significa que el **sync target es una carpeta local** (en contraposición a un API remota como S3 o WebDAV). Es decir:

- La app lee/escribe el vault en `$APP_DATA/vault/`
- El sync copia esos archivos a **otra ruta del mismo sistema de archivos** que tú configuras, por ejemplo `/Users/guille/Dropbox/vsyncnotes/`

Esa carpeta destino puede ser:
- Una carpeta de Dropbox/iCloud que el cliente cloud sincroniza automáticamente
- Un NAS montado como volumen
- Un disco externo
- Cualquier ruta accesible localmente

Es decir, la Fase 6 implementa el **motor de sync** (comparar timestamps, push/pull, resolver conflictos) asumiendo que el destino es simplemente una ruta en el sistema de archivos. Las fases posteriores (8 en el roadmap) añadirían providers que hablan con APIs remotas directamente (S3, WebDAV, Google Drive API...).
