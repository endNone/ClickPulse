<p align="center"><img src="../../assets/clickpulse-blue-cursor-v3.png" width="72" alt="ClickPulse"></p>

<h1 align="center">ClickPulse</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="zh-CN.md">简体中文</a> · <a href="zh-TW.md">繁體中文</a> · <strong>日本語</strong> · <a href="ko.md">한국어</a> · <a href="de.md">Deutsch</a> · <a href="es.md">Español</a> · <a href="fr.md">Français</a></p>

<p align="center"><strong>日々のキー入力とクリックを、手元で振り返る。</strong></p>

<p align="center">ClickPulse は、キーボードとマウスの使用状況を日ごとの統計、履歴、ヒートマップで表示するオフラインのデスクトップアプリです。</p>

<p align="center">
  <a href="#インストール"><img src="../../assets/badges/preview.svg" height="24" alt="パッケージをダウンロード"></a>
  <a href="../development.md"><img src="../../assets/badges/rust.svg" height="24" alt="Rust"></a>
  <a href="../../LICENSE"><img src="../../assets/badges/license.svg" height="24" alt="GPL-3.0-or-later"></a>
  <br>
  <a href="../install-macos.md"><img src="../../assets/badges/macos.svg" height="24" alt="macOS 12+ · arm64/x64"></a>
  <a href="../install-windows.md"><img src="../../assets/badges/windows.svg" height="24" alt="Windows 10/11 · x64"></a>
  <a href="../install-linux.md"><img src="../../assets/badges/linux.svg" height="24" alt="Linux · Ubuntu 22.04 / 24.04"></a>
</p>

<p align="center"><a href="https://github.com/endNone/ClickPulse/releases/latest">パッケージをダウンロード</a> · <a href="../development.md">ソースからビルド</a></p>

<p align="center">キー入力とクリックの統計は端末内に保存されます。アカウント登録は不要で、テレメトリも収集しません。</p>

<p align="center">更新の確認とダウンロードの際は GitHub に接続します。自動確認は設定でオフにできます。</p>

## ダウンロードサイズとメモリ使用量

v0.3.1 のデスクトップ版はどのパッケージも 7 MiB 未満です。検証した 3 つの環境では、バックグラウンド起動時のメモリ使用量が v0.3.0 より **88.5～98.9% 減少**しました。

**ダウンロードサイズ（MiB）**

| プラットフォーム | x64 | ARM64 |
| :--- | ---: | ---: |
| macOS 12+ | 6.93 DMG · 6.26 ZIP | 6.66 DMG · 6.02 ZIP |
| Windows 10/11 | 4.40 ZIP | — |
| Ubuntu 22.04 / 24.04 | 6.23 ZIP | 6.15 ZIP |

**メモリ使用量の実測値（MiB）**

| 検証環境 | バックグラウンド起動時 | 通常ウィンドウ | Mini モード | 非表示から 10 分後 |
| :--- | ---: | ---: | ---: | ---: |
| macOS · Apple Silicon | **15.8** | 223.2 | 158.0 | 29.7 |
| Windows 11 · x64 | **2.3** | 214.5 | 173.0 | 3.6 |
| Ubuntu 24.04 · x64 | **42.5** | 249.7 | 254.2 | 104.2 |

入力の記録をオフにし、アプリ本体と画面表示に使うプロセスの合計を 5 回測定した中央値です。通常ウィンドウの表示言語は英語です。macOS は physical footprint、Windows は USS、Ubuntu はソフトウェアレンダリング時の PSS を使用しています。 [v0.3.1 のダウンロードと測定結果](https://github.com/endNone/ClickPulse/releases/tag/v0.3.1).

## 概要

[![概要](../images/overview-en.jpg)](../images/overview-en.jpg)

## 機能

- 推移グラフとカレンダーで、日々の活動パターンを確認できます。
- キーボードのヒートマップとツリーマップで、よく使うキーが分かります。
- 履歴から任意の日を選んで、その日の活動を振り返れます。
- 保存先の変更や、別のパソコンの履歴との統合ができます。
- 4 種類のテーマ、8 種類の表示言語、集計に使うタイムゾーンを選べます。

## インストール

- **macOS 12+** (Apple Silicon / Intel) · [DMG をダウンロード](https://github.com/endNone/ClickPulse/releases/latest)
- **Windows 10 / 11** (x64) · [ポータブル版 ZIP をダウンロード](https://github.com/endNone/ClickPulse/releases/latest)
- **Linux** (Ubuntu 22.04 / 24.04) · [Linux 版をダウンロード](https://github.com/endNone/ClickPulse/releases/latest)

### Homebrew または Scoop でインストール

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

macOS 版は Apple の公証を受けておらず、Windows 版は未署名です。初回起動時に OS が実行をブロックすることがあります。

OS 別インストールガイド · [macOS](../install-macos.md) · [Windows](../install-windows.md) · [Ubuntu](../install-linux.md)

## スクリーンショット

画像にはすべてデモデータを使用しています。

### Mini モード

小さなフローティングウィンドウで、他のアプリを使いながら入力回数を確認できます。

[![Mini モード](../images/mini-desktop-en.webp)](../images/mini-desktop-en.webp)

### Sport モード

Mini ウィンドウの稲妻ボタンを押すと、入力やクリックの速さに反応するアニメーションが有効になります。

[![Sport モード](../images/sport-desktop-en.gif)](../images/sport-desktop-en.gif)

### キーボードヒートマップ

[![キーボードヒートマップ](../images/keyboard-en.jpg)](../images/keyboard-en.jpg)

### キー使用頻度のツリーマップ

[![キー使用頻度のツリーマップ](../images/treemap-en.jpg)](../images/treemap-en.jpg)

### 履歴

[![履歴](../images/history-en.jpg)](../images/history-en.jpg)

## ライセンス

コードのライセンスは [GPL-3.0-or-later](../../LICENSE) です。v0.3.2 以降、ClickPulse の名称とアイコンはこの許諾の対象外で、GPLv3 の第 7(c) 項および第 7(e) 項に基づく[追加条項](../../TRADEMARKS.md)が適用されます。それ以前のリリースには公開時の条項が適用されます。

## 関連リンク

[ドキュメント](../README.md) · [変更履歴](../../CHANGELOG.md)<br>
[不具合を報告](https://github.com/endNone/ClickPulse/issues/new/choose) · [ディスカッションに参加](https://github.com/endNone/ClickPulse/discussions)

## GitHub スター数の推移

ClickPulse が役に立ったら、[GitHub でスター](https://github.com/endNone/ClickPulse)を付けていただけるとうれしいです 🌟

<p align="center"><a href="https://www.star-history.com/#endNone/ClickPulse&amp;Date"><img src="https://api.star-history.com/image?repos=endNone/ClickPulse&amp;type=Date" width="800" alt="GitHub スター数の推移"></a></p>

## 謝辞

ClickPulse を試し、感想や改善案を寄せてくださった [Appinn（小众软件）コミュニティ](https://meta.appinn.net/t/topic/95871/16)の皆さんに感謝します。
