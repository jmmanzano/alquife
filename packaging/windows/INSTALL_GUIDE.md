# Instalador de Alquife para Windows

## Instalación

### Pasos rápidos:
1. Descarga `alquife.exe` junto con `install.bat` en la misma carpeta
2. Ejecuta `install.bat` haciendo doble clic
3. El instalador:
   - Copia `alquife.exe` a `%LOCALAPPDATA%\Alquife\`
   - Crea accesos directos en el menú Inicio y Escritorio
   - Descarga e instala `ffmpeg` si no está disponible
   - Añade la ruta a la variable PATH del usuario

### Ubicación de archivos:

| Tipo | Ubicación |
|------|-----------|
| **Ejecutable** | `%LOCALAPPDATA%\Alquife\alquife.exe` |
| **Configuración** | `%LOCALAPPDATA%\Alquife\config.toml` |
| **Temas** | `%LOCALAPPDATA%\Alquife\themes\` |
| **Equalizadores** | `%LOCALAPPDATA%\Alquife\equalizer\` |
| **Cola de reproducción** | `%LOCALAPPDATA%\Alquife\queue.json` |
| **Estado UI** | `%LOCALAPPDATA%\Alquife\ui_state.json` |
| **FFmpeg** | `%LOCALAPPDATA%\Alquife\ffmpeg\bin\` |
| **Accesos directos** | Menú Inicio y Escritorio |

### Instalación personalizada:
```batch
install.bat "C:\ruta\personalizada\Alquife"
```

## Desinstalación

1. Ejecuta `uninstall.bat`
2. Confirma la desinstalación cuando se te solicite
3. Elige si quieres mantener o eliminar los archivos de configuración

El desinstalador:
- Elimina el directorio de instalación
- Elimina los accesos directos
- Limpia la variable PATH del usuario
- Pregunta si deseas eliminar los datos de configuración

## Acceso a los archivos de configuración

- **Explorador**: `%LOCALAPPDATA%\Alquife\`
- **Terminal**: `cd %LOCALAPPDATA%\Alquife`

