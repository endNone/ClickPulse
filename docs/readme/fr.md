<p align="center"><img src="../../assets/clickpulse-blue-cursor-v3.png" width="72" alt="ClickPulse"></p>

<h1 align="center">ClickPulse</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="zh-CN.md">简体中文</a> · <a href="zh-TW.md">繁體中文</a> · <a href="ja.md">日本語</a> · <a href="ko.md">한국어</a> · <a href="de.md">Deutsch</a> · <a href="es.md">Español</a> · <strong>Français</strong></p>

<p align="center"><strong>Vos frappes et vos clics au fil des jours, même hors ligne.</strong></p>

<p align="center">ClickPulse est une application de bureau qui présente votre activité au clavier et à la souris sous forme de statistiques quotidiennes, d’historique et de cartes de chaleur. Elle fonctionne hors ligne.</p>

<p align="center">
  <a href="#installation"><img src="../../assets/badges/preview.svg" height="24" alt="Télécharger l’application"></a>
  <a href="../development.md"><img src="../../assets/badges/rust.svg" height="24" alt="Rust"></a>
  <a href="../../LICENSE"><img src="../../assets/badges/license.svg" height="24" alt="GPL-3.0-or-later"></a>
  <br>
  <a href="../install-macos.md"><img src="../../assets/badges/macos.svg" height="24" alt="macOS 12+ · arm64/x64"></a>
  <a href="../install-windows.md"><img src="../../assets/badges/windows.svg" height="24" alt="Windows 10/11 · x64"></a>
  <a href="../install-linux.md"><img src="../../assets/badges/linux.svg" height="24" alt="Linux · Ubuntu 22.04 / 24.04"></a>
</p>

<p align="center"><a href="https://github.com/endNone/ClickPulse/releases/latest">Télécharger l’application</a> · <a href="../development.md">Compiler depuis les sources</a></p>

<p align="center">Vos statistiques de frappe et de clic restent sur votre appareil. Aucun compte, aucune télémétrie.</p>

<p align="center">La recherche et le téléchargement des mises à jour passent par GitHub. Vous pouvez désactiver la recherche automatique dans les paramètres.</p>

## Taille du téléchargement et mémoire utilisée

Tous les paquets de bureau de la v0.3.1 pèsent moins de 7 MiB. Au démarrage en arrière-plan, la consommation de mémoire a **diminué de 88,5 à 98,9 %** par rapport à la v0.3.0 sur les trois systèmes testés.

**Taille du téléchargement (MiB)**

| Plateforme | x64 | ARM64 |
| :--- | ---: | ---: |
| macOS 12+ | 6,93 DMG · 6,26 ZIP | 6,66 DMG · 6,02 ZIP |
| Windows 10/11 | 4,40 ZIP | — |
| Ubuntu 22.04 / 24.04 | 6,23 ZIP | 6,15 ZIP |

**Mémoire mesurée (MiB)**

| Système testé | Démarrage en arrière-plan | Fenêtre principale | Mode Mini | Après 10 min masquée |
| :--- | ---: | ---: | ---: | ---: |
| macOS · Apple Silicon | **15,8** | 223,2 | 158,0 | 29,7 |
| Windows 11 · x64 | **2,3** | 214,5 | 173,0 | 3,6 |
| Ubuntu 24.04 · x64 | **42,5** | 249,7 | 254,2 | 104,2 |

Médiane de cinq mesures pour l’application et ses processus d’affichage, avec l’enregistrement des entrées désactivé et la fenêtre principale en anglais. Les mesures utilisent l’empreinte physique sous macOS, l’USS sous Windows et le PSS sous Ubuntu avec rendu logiciel. [Téléchargements et mesures de la v0.3.1](https://github.com/endNone/ClickPulse/releases/tag/v0.3.1).

## Vue d’ensemble

[![Vue d’ensemble](../images/overview-en.jpg)](../images/overview-en.jpg)

## Fonctionnalités

- Repérez vos périodes d’activité grâce aux tendances et au calendrier.
- Retrouvez les touches les plus utilisées sur la carte de chaleur et le diagramme en rectangles.
- Consultez l’historique pour revoir l’activité d’une journée précise.
- Déplacez vos données ou fusionnez-les avec l’historique d’un autre ordinateur.
- Choisissez parmi quatre thèmes et huit langues, et définissez le fuseau horaire de vos statistiques.

## Installation

- **macOS 12+** (Apple Silicon / Intel) · [Télécharger le DMG](https://github.com/endNone/ClickPulse/releases/latest)
- **Windows 10 / 11** (x64) · [Télécharger le ZIP portable](https://github.com/endNone/ClickPulse/releases/latest)
- **Linux** (Ubuntu 22.04 / 24.04) · [Télécharger la version Linux](https://github.com/endNone/ClickPulse/releases/latest)

### Installer avec Homebrew ou Scoop

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

La version macOS n’est pas notariée par Apple et la version Windows n’est pas signée. Le système peut bloquer le premier lancement.

Guides d’installation · [macOS](../install-macos.md) · [Windows](../install-windows.md) · [Ubuntu](../install-linux.md)

## Captures d’écran

Toutes les captures présentent des données de démonstration.

### Mode Mini

Gardez vos compteurs à l’écran dans une petite fenêtre flottante pendant que vous utilisez d’autres applications.

[![Mode Mini](../images/mini-desktop-en.webp)](../images/mini-desktop-en.webp)

### Mode Sport

Le bouton en forme d’éclair de la fenêtre Mini active des animations qui réagissent à votre rythme de frappe et de clic.

[![Mode Sport](../images/sport-desktop-en.gif)](../images/sport-desktop-en.gif)

### Carte de chaleur du clavier

[![Carte de chaleur du clavier](../images/keyboard-en.jpg)](../images/keyboard-en.jpg)

### Répartition des frappes par touche

[![Répartition des frappes par touche](../images/treemap-en.jpg)](../images/treemap-en.jpg)

### Historique

[![Historique](../images/history-en.jpg)](../images/history-en.jpg)

## Licence

Le code est distribué sous licence [GPL-3.0-or-later](../../LICENSE). À partir de la version v0.3.2, le nom ClickPulse et ses icônes sont exclus de cette autorisation. Des [conditions supplémentaires](../../TRADEMARKS.md) s’appliquent conformément aux articles 7(c) et 7(e) de la GPLv3. Les versions antérieures conservent leurs conditions d’origine.

## Pour aller plus loin

[Documentation](../README.md) · [Journal des modifications](../../CHANGELOG.md)<br>
[Signaler un problème](https://github.com/endNone/ClickPulse/issues/new/choose) · [Participer aux discussions](https://github.com/endNone/ClickPulse/discussions)

## Évolution des étoiles sur GitHub

Si ClickPulse vous est utile, pensez à lui laisser [une étoile sur GitHub](https://github.com/endNone/ClickPulse) 🌟

<p align="center"><a href="https://www.star-history.com/#endNone/ClickPulse&amp;Date"><img src="https://api.star-history.com/image?repos=endNone/ClickPulse&amp;type=Date" width="800" alt="Évolution des étoiles sur GitHub"></a></p>

## Remerciements

Merci à la [communauté Appinn (小众软件)](https://meta.appinn.net/t/topic/95871/16) d’avoir essayé ClickPulse et partagé ses impressions et ses suggestions.
