---
name: build-dev
description: Esta habilidar permite generar un deb instalable de la aplicación.
---

# Build Dev Skill

Este skill automatiza la construcción de un paquete `.deb` instalable para Alquife.

## Uso

Para construir el paquete Debian, ejecuta el siguiente comando:

```bash
bash .github/scripts/build_deb.sh
```

## Funcionalidad

El script `build_deb.sh` realiza las siguientes tareas:

1. **Validación de dependencias**: Verifica que `dpkg-deb` y `dpkg` estén instalados
2. **Lectura de versión**: Extrae la versión del archivo `Cargo.toml`
3. **Compilación**: Construye el binario en modo release
   - Intenta primero con el toolchain estable
   - Si falla, reinenta con `cargo +nightly`
4. **Preparación del árbol de paquetes**: 
   - Crea la estructura de directorios Debian
   - Instala el binario en `/usr/bin/alquife`
   - Instala el archivo desktop en `/usr/share/applications/`
   - Instala el icono en `/usr/share/icons/hicolor/256x256/apps/`
5. **Creación del archivo control**: Genera el metadatos del paquete con dependencias
6. **Empaquetamiento**: Construye el archivo `.deb`

## Requisitos

- `dpkg` y `dpkg-deb` instalados (en sistemas Debian/Ubuntu)
- Rust y Cargo
- Las fuentes del proyecto con:
  - `Cargo.toml` (con versión)
  - `packaging/debian/alquife.desktop`
  - Ícono en `icons/alquife.png` o `screenshots/alquife.png`

## Instalación del paquete

Una vez construido, instala el paquete `.deb` con:

```bash
sudo apt install ./target/deb/alquife_*.deb
```

## Dependencias del paquete

El `.deb` generado depende de:
- `mpv` - Reproducción de audio
- `pipewire` - Server de audio
- `wireplumber` - Gestor de audio
- `dbus` - Sistema de mensajería