<p align="center"><img src="../../assets/clickpulse-blue-cursor-v3.png" width="72" alt="ClickPulse"></p>

<h1 align="center">ClickPulse</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="zh-CN.md">简体中文</a> · <strong>繁體中文</strong> · <a href="ja.md">日本語</a> · <a href="ko.md">한국어</a> · <a href="de.md">Deutsch</a> · <a href="es.md">Español</a> · <a href="fr.md">Français</a></p>

<p align="center"><strong>每一次敲擊與點按，都有跡可循。</strong></p>

<p align="center">ClickPulse 是一款離線桌面應用程式，透過每日統計、歷史紀錄與熱度圖，呈現你的鍵盤與滑鼠使用情形。</p>

<p align="center">
  <a href="#安裝"><img src="../../assets/badges/preview.svg" height="24" alt="下載安裝套件"></a>
  <a href="../development.md"><img src="../../assets/badges/rust.svg" height="24" alt="Rust"></a>
  <a href="../../LICENSE"><img src="../../assets/badges/license.svg" height="24" alt="GPL-3.0-or-later"></a>
  <br>
  <a href="../install-macos.md"><img src="../../assets/badges/macos.svg" height="24" alt="macOS 12+ · arm64/x64"></a>
  <a href="../install-windows.md"><img src="../../assets/badges/windows.svg" height="24" alt="Windows 10/11 · x64"></a>
  <a href="../install-linux.md"><img src="../../assets/badges/linux.svg" height="24" alt="Linux · Ubuntu 22.04 / 24.04"></a>
</p>

<p align="center"><a href="https://github.com/endNone/ClickPulse/releases/latest">下載安裝套件</a> · <a href="../development.md">從原始碼建置</a></p>

<p align="center">按鍵與點擊統計只儲存在你的裝置上，無須帳號，也不收集遙測資料。</p>

<p align="center">檢查與下載更新時會連線至 GitHub。你可以在設定中關閉自動檢查。</p>

## 總覽

[![總覽](../images/overview-en.jpg)](../images/overview-en.jpg)

## 功能

- 透過活動趨勢和日曆，找出每天最活躍的時段。
- 在鍵盤熱度圖與矩形式樹狀圖中查看最常用的按鍵。
- 在歷史紀錄中回顧任意一天的活動。
- 移動資料夾，或合併另一部電腦上的歷史紀錄。
- 從四款佈景主題與八種介面語言中挑選，並指定統計使用的時區。

## 安裝

- **macOS 12+** (Apple Silicon / Intel) · [下載 DMG](https://github.com/endNone/ClickPulse/releases/latest)
- **Windows 10 / 11** (x64) · [下載可攜式 ZIP](https://github.com/endNone/ClickPulse/releases/latest)
- **Linux** (Ubuntu 22.04 / 24.04) · [下載 Linux 套件](https://github.com/endNone/ClickPulse/releases/latest)

### 使用 Homebrew 或 Scoop 安裝

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

macOS 版未經 Apple 公證，Windows 版未簽署。系統可能會阻擋首次啟動。

各平台安裝指南 · [macOS](../install-macos.md) · [Windows](../install-windows.md) · [Ubuntu](../install-linux.md)

## 畫面截圖

以下截圖均使用示範資料。

### Mini 模式

使用其他應用程式時，也能透過小型浮動視窗查看即時計數。

[![Mini 模式](../images/mini-desktop-en.webp)](../images/mini-desktop-en.webp)

### Sport 模式

點選 Mini 視窗中的閃電按鈕，即可開啟隨打字與點擊速度變化的動畫效果。

[![Sport 模式](../images/sport-desktop-en.gif)](../images/sport-desktop-en.gif)

### 鍵盤熱度圖

[![鍵盤熱度圖](../images/keyboard-en.jpg)](../images/keyboard-en.jpg)

### 按鍵使用分布（矩形式樹狀圖）

[![按鍵使用分布（矩形式樹狀圖）](../images/treemap-en.jpg)](../images/treemap-en.jpg)

### 歷史紀錄

[![歷史紀錄](../images/history-en.jpg)](../images/history-en.jpg)

## 授權條款

程式碼採用 [GPL-3.0-or-later](../../LICENSE) 授權。自 v0.3.2 起，ClickPulse 名稱與圖示不在此授權範圍內，適用依 GPLv3 第 7(c) 與 7(e) 款訂定的[附加條款](../../TRADEMARKS.md)。先前發布的版本沿用發布當時的條款。

## 更多資訊

[專案文件](../README.md) · [更新紀錄](../../CHANGELOG.md)<br>
[回報問題](https://github.com/endNone/ClickPulse/issues/new/choose) · [參與討論](https://github.com/endNone/ClickPulse/discussions)

## GitHub 星號數趨勢

如果 ClickPulse 對你有幫助，歡迎在 [GitHub 上給它一顆星](https://github.com/endNone/ClickPulse) 🌟

<p align="center"><a href="https://www.star-history.com/#endNone/ClickPulse&amp;Date"><img src="https://api.star-history.com/image?repos=endNone/ClickPulse&amp;type=Date" width="800" alt="GitHub 星號數趨勢"></a></p>

## 致謝

感謝[小众软件社群](https://meta.appinn.net/t/topic/95871/16)的朋友們試用 ClickPulse，分享使用心得與改進建議。
