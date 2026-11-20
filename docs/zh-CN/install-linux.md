# Ubuntu 安装指南

[English](../install-linux.md) · **简体中文**

ClickPulse Linux 版面向 **Ubuntu 22.04** 和 **24.04**，提供 **x64**（Intel/AMD 电脑）和 **ARM64**（如 Apple Silicon 虚拟机）便携包。

## 1. 安装依赖与解压

先安装所需系统库：

```sh
sudo apt update
sudo apt install libgtk-3-0 libwebkit2gtk-4.1-0 libayatana-appindicator3-1 libxdo3 fonts-noto-cjk
```

中日韩界面使用系统字体，安装 `fonts-noto-cjk` 可避免最小化安装的 Ubuntu 出现方块乱码。

将完整的便携版 ZIP 解压到固定目录（例如 `~/.local/opt/ClickPulse`），保留同目录下的清单与许可文件，然后运行 `./clickpulse`。如需使用附带的 `.desktop` 文件，请先将其中 `Exec` 和 `Icon` 改为绝对路径后再复制到 `~/.local/share/applications/`。

## 2. 授予输入读取权限

以普通桌面用户运行以下命令，随后**注销并重新登录**生效：

```sh
sudo usermod -aG input "$USER"
```

加入 `input` 用户组允许程序以只读方式监听物理键鼠事件（不记录输入文本、鼠标轨迹或窗口标题）。请勿使用 `sudo` 运行桌面端。采集器独立于 X11/Wayland，自动过滤长按连发并支持设备热插拔。

## 3. 托盘、更新与数据目录

- 关闭窗口时可选择隐藏到托盘或保存退出；显示托盘图标需启用 Ubuntu 的 AppIndicator 扩展。Wayland 下置顶与窗口定位受桌面合成器限制。
- 在 **设置** 中支持开机自启、自动/手动检查更新；新版本会在后台下载，确认后原地更新。
- 历史数据与设置位于 `$XDG_DATA_HOME/ClickPulse`（通常为 `~/.local/share/ClickPulse`），更新或删除程序目录不会清除历史数据。
