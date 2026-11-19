<p align="center"><img src="../../assets/clickpulse-blue-cursor-v3.png" width="72" alt="ClickPulse"></p>

<h1 align="center">ClickPulse</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="zh-CN.md">简体中文</a> · <a href="zh-TW.md">繁體中文</a> · <a href="ja.md">日本語</a> · <a href="ko.md">한국어</a> · <a href="de.md">Deutsch</a> · <strong>Español</strong> · <a href="fr.md">Français</a></p>

<p align="center"><strong>Tus pulsaciones y clics a lo largo del día, incluso sin conexión.</strong></p>

<p align="center">ClickPulse es una aplicación de escritorio que muestra tu actividad con el teclado y el ratón mediante estadísticas diarias, un historial y mapas de calor. Funciona sin conexión.</p>

<p align="center">
  <a href="#instalación"><img src="../../assets/badges/preview.svg" height="24" alt="Descargar la aplicación"></a>
  <a href="../development.md"><img src="../../assets/badges/rust.svg" height="24" alt="Rust"></a>
  <a href="../../LICENSE"><img src="../../assets/badges/license.svg" height="24" alt="GPL-3.0-or-later"></a>
  <br>
  <a href="../install-macos.md"><img src="../../assets/badges/macos.svg" height="24" alt="macOS 12+ · arm64/x64"></a>
  <a href="../install-windows.md"><img src="../../assets/badges/windows.svg" height="24" alt="Windows 10/11 · x64"></a>
  <a href="../install-linux.md"><img src="../../assets/badges/linux.svg" height="24" alt="Linux · Ubuntu 22.04 / 24.04"></a>
</p>

<p align="center"><a href="https://github.com/endNone/ClickPulse/releases/latest">Descargar la aplicación</a> · <a href="../development.md">Compilar desde el código fuente</a></p>

<p align="center">Las estadísticas de pulsaciones y clics se guardan en tu dispositivo. Sin cuenta ni telemetría.</p>

<p align="center">La aplicación se conecta a GitHub para buscar y descargar actualizaciones. Puedes desactivar la búsqueda automática en los ajustes.</p>

## Tamaño de descarga y uso de memoria

Todos los paquetes de escritorio de v0.3.1 ocupan menos de 7 MiB. Al iniciarse en segundo plano, la aplicación usó **entre un 88,5 % y un 98,9 % menos de memoria** que v0.3.0 en los tres sistemas probados.

**Tamaño de descarga (MiB)**

| Plataforma | x64 | ARM64 |
| :--- | ---: | ---: |
| macOS 12+ | 6,93 DMG · 6,26 ZIP | 6,66 DMG · 6,02 ZIP |
| Windows 10/11 | 4,40 ZIP | — |
| Ubuntu 22.04 / 24.04 | 6,23 ZIP | 6,15 ZIP |

**Memoria medida (MiB)**

| Sistema probado | Inicio en segundo plano | Ventana principal | Modo Mini | Tras 10 min oculta |
| :--- | ---: | ---: | ---: | ---: |
| macOS · Apple Silicon | **15,8** | 223,2 | 158,0 | 29,7 |
| Windows 11 · x64 | **2,3** | 214,5 | 173,0 | 3,6 |
| Ubuntu 24.04 · x64 | **42,5** | 249,7 | 254,2 | 104,2 |

Mediana de cinco mediciones de la aplicación y sus procesos de interfaz, con el registro de entradas desactivado y la ventana principal en inglés. En macOS se midió la huella física de memoria, en Windows el USS y en Ubuntu el PSS con renderizado por software. [Descargas y resultados de v0.3.1](https://github.com/endNone/ClickPulse/releases/tag/v0.3.1).

## Vista general

[![Vista general](../images/overview-en.jpg)](../images/overview-en.jpg)

## Funciones

- Descubre tus patrones de uso con las tendencias de actividad y el calendario.
- Identifica las teclas que más utilizas con el mapa de calor y el diagrama de rectángulos.
- Consulta el historial para ver la actividad de cualquier día.
- Cambia la ubicación de tus datos o combina tu historial con el de otro ordenador.
- Elige entre cuatro temas y ocho idiomas, y selecciona la zona horaria de tus estadísticas.

## Instalación

- **macOS 12+** (Apple Silicon / Intel) · [Descargar DMG](https://github.com/endNone/ClickPulse/releases/latest)
- **Windows 10 / 11** (x64) · [Descargar el ZIP portátil](https://github.com/endNone/ClickPulse/releases/latest)
- **Linux** (Ubuntu 22.04 / 24.04) · [Descargar para Linux](https://github.com/endNone/ClickPulse/releases/latest)

### Instalar con Homebrew o Scoop

**macOS · Homebrew**

```sh
brew tap endnone/clickpulse https://github.com/endNone/ClickPulse
brew install --cask endnone/clickpulse/clickpulse
```

**Windows · Scoop**

```powershell
scoop bucket add clickpulse https://github.com/endNone/ClickPulse
scoop install clickpulse
```

La aplicación para macOS no está notarizada por Apple y la de Windows no está firmada. El sistema puede bloquear el primer inicio.

Guías de instalación · [macOS](../install-macos.md) · [Windows](../install-windows.md) · [Ubuntu](../install-linux.md)

## Capturas de pantalla

Todas las capturas muestran datos de demostración.

### Modo Mini

Consulta los contadores en tiempo real en una pequeña ventana flotante mientras usas otras aplicaciones.

[![Modo Mini](../images/mini-desktop-en.webp)](../images/mini-desktop-en.webp)

### Modo Sport

Pulsa el botón con el rayo en la ventana Mini para activar animaciones que responden a tu ritmo de tecleo y de clics.

[![Modo Sport](../images/sport-desktop-en.gif)](../images/sport-desktop-en.gif)

### Mapa de calor del teclado

[![Mapa de calor del teclado](../images/keyboard-en.jpg)](../images/keyboard-en.jpg)

### Distribución de pulsaciones por tecla

[![Distribución de pulsaciones por tecla](../images/treemap-en.jpg)](../images/treemap-en.jpg)

### Historial

[![Historial](../images/history-en.jpg)](../images/history-en.jpg)

## Licencia

El código se distribuye bajo la licencia [GPL-3.0-or-later](../../LICENSE). A partir de v0.3.2, el nombre ClickPulse y sus iconos quedan fuera de esta licencia y se aplican [condiciones adicionales](../../TRADEMARKS.md) conforme a los apartados 7(c) y 7(e) de la GPLv3. Las versiones anteriores conservan sus condiciones originales.

## Más información

[Documentación](../README.md) · [Historial de cambios](../../CHANGELOG.md)<br>
[Informar de un error](https://github.com/endNone/ClickPulse/issues/new/choose) · [Participar en las conversaciones](https://github.com/endNone/ClickPulse/discussions)

## Evolución de las estrellas en GitHub

Si ClickPulse te resulta útil, puedes darle [una estrella en GitHub](https://github.com/endNone/ClickPulse) 🌟

<p align="center"><a href="https://www.star-history.com/#endNone/ClickPulse&amp;Date"><img src="https://api.star-history.com/image?repos=endNone/ClickPulse&amp;type=Date" width="800" alt="Evolución de las estrellas en GitHub"></a></p>

## Agradecimientos

Gracias a la [comunidad de Appinn (小众软件)](https://meta.appinn.net/t/topic/95871/16) por probar ClickPulse y compartir sus impresiones y sugerencias.
