# 使用 Homebrew 或 Scoop 安装

[English](../package-managers.md) · **简体中文**

以下软件源由作者 Wuhan Zhang 在 [endNone/ClickPulse](https://github.com/endNone/ClickPulse) 仓库内直接维护（非 Homebrew 官方核心 cask、Scoop Extras 或 WinGet 仓库）。两者均下载 GitHub Releases 中的同源版本包并校验 SHA256。

## macOS：Homebrew Cask

已安装 [Homebrew](https://brew.sh) 时运行：

```sh
brew tap endnone/clickpulse https://github.com/endNone/ClickPulse && brew install --cask endnone/clickpulse/clickpulse
```

Cask 会自动匹配 Apple Silicon 或 Intel 架构并将 `ClickPulse.app` 安装到 `/Applications`（仅当前用户安装可附加 `--appdir="$HOME/Applications"`）。若此前手动安装过，建议先退出并移入废纸篓，避免系统里并存两份应用；独立的历史数据目录会完整保留。

从 **0.3.2** 起公开发布包使用固定自签证书，但仍**未经 Apple 公证**。首次打开与输入监控权限请按 [macOS 安装指南](install-macos.md#2-allow-the-first-launch) 操作。

更新或卸载前请先退出 ClickPulse：

```sh
brew update
brew upgrade --cask --greedy endnone/clickpulse/clickpulse
# 卸载程序（保留历史数据）：
brew uninstall --cask endnone/clickpulse/clickpulse
# 连同本地历史与设置一并清除：
brew uninstall --zap --cask endnone/clickpulse/clickpulse
```

## Windows：Scoop

已安装 [Scoop](https://scoop.sh) 时在 PowerShell 运行：

```powershell
scoop bucket add clickpulse https://github.com/endNone/ClickPulse; if ($?) { scoop install clickpulse/clickpulse }
```

Scoop 会下载完整 x64 便携包并创建开始菜单快捷方式；数据仍保存在 `%LOCALAPPDATA%\ClickPulse` 或自定义目录。若缺少 WebView2 Runtime 请按提示安装。由于程序未进行商业签名，如遇 Smart App Control 拦截请见 [Windows 安装指南](install-windows.md#windows-security-prompts)。

更新或卸载前请先退出 ClickPulse：

```powershell
scoop update
scoop update clickpulse
# 卸载程序（保留历史数据）：
scoop uninstall clickpulse
```

使用 Scoop 或 WinGet 安装的版本在应用内检查更新时，会直接提示对应的命令行更新指令，不会自行覆盖包管理器托管的文件。如需在卸载时选择是否删除本地历史，请先在软件内使用 **设置 → 应用 → 卸载与数据**。
