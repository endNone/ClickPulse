# ClickPulse 快捷键百科 / Shortcut Atlas

App 自动显示当前操作系统的快捷键统计和百科词条。网页版可按系统查阅组合键、常见用途及官方来源。

## 如何理解统计

- 新版开始累计快捷键，旧按键总数不能还原历史组合。
- 只保存系统、日期、修饰键、主键与按日累计次数，不保存时间顺序、文本或应用名。
- 日期按记录时的系统本地日期划分，与每日总数一致；更改显示时区不重分组快捷键。
- 私有预览版的无日期累计只计入全部历史，不计入今日或日期范围。
- 默认隐藏只含 Shift 的组合；可在统计页打开显示，不影响记录或百科。
- 左右修饰键合并；按住产生的重复事件不重复累计。
- 普通打字不计入快捷键；Mac Option 字符输入与 Windows/Linux AltGr 字符输入排除。
- 按键标识按物理键位统一；非美式布局和自定义改键可能与标注名称不同。
- 命中词条仅代表组合相同，不代表用途已执行；某些系统保留组合可能无法捕获。
- Linux 词条适用于 Ubuntu 的 GNOME 桌面，其他桌面可有不同默认设置。
- 导入旧备份时，同一日期、来源和组合取较高累计值，不同来源相加，避免重复导入增加次数。

## 阅读与网页

本文件可在 GitHub 直接阅读。网页默认中文，支持切换英文、用途筛选和搜索。首页及 Mac、Windows、Ubuntu 独立页面均预先生成完整表格，不依赖 JavaScript 阅读。
搜索支持 Ctrl+C、Command+Shift+3、⌘⇧3 等常见写法，也可以搜索用途。

本地生成：`node scripts/build-shortcut-guide.cjs`，随后用任意静态服务器打开 `dist/shortcut-guide/`。

生成结果包含 canonical 和 sitemap.xml；可通过 SHORTCUT_GUIDE_BASE_URL 指定正式站点地址。

GitHub Pages 发布：在仓库 Settings → Pages 中选择 GitHub Actions，再手动运行 Shortcut guide 工作流。
该工作流仅手动触发，不随普通代码推送自动发布。部署后的地址由工作流的 environment 输出给出。

## 维护词条

词条位于 `ui/shortcuts-catalog.js`：平台、修饰键、键位、场景、用途、来源。
新增词条应核对官方说明；用途保持简短，避免将应用专属操作当作系统通用行为。
运行 `node --test tests/insights.test.cjs` 与本生成脚本可检查词条并同步文档。

## 系统索引

### macOS

| 组合键 | 用途 / Use | 场景 | 来源 |
| --- | --- | --- | --- |
| `⌘` + `C` | 复制 / Copy | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `V` | 粘贴 / Paste | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `X` | 剪切 / Cut | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `Z` | 撤销 / Undo | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `A` | 全选 / Select all | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `S` | 保存 / Save | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `F` | 查找 / Find | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `G` | 下一个匹配 / Next match | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `P` | 打印 / Print | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `Z` | 重做 / Redo | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `G` | 上一个匹配 / Previous match | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `Q` | 退出应用 / Quit app | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `W` | 关闭窗口 / Close window | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `H` | 隐藏应用 / Hide app | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `M` | 最小化 / Minimize | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `Tab` | 切换应用 / Switch app | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `Space` | 聚焦搜索 / Spotlight | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `` ` `` | 切换应用内窗口 / Switch app window | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `,` | 应用设置 / App settings | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `3` | 全屏截图 / Screen capture | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `4` | 区域截图 / Area capture | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `5` | 截屏工具 / Capture controls | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `⌘` + `F` | 切换全屏 / Toggle fullscreen | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `⌘` + `Space` | 字符检视器 / Character viewer | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `⌘` + `Q` | 锁定屏幕 / Lock screen | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `N` | 新建窗口 / New window | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `O` | 打开所选项目 / Open selection | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `I` | 项目信息 / Item information | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `Backspace` | 移到废纸篓 / Move to Trash | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `D` | 复制项目 / Duplicate item | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `N` | 新建文件夹 / New folder | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `G` | 前往文件夹 / Go to folder | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `A` | 应用程序文件夹 / Applications folder | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `H` | 个人文件夹 / Home folder | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `Left` | 行首 / Line start | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `Right` | 行尾 / Line end | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `Up` | 文稿开头 / Document start | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `Down` | 文稿末尾 / Document end | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |

### Windows

| 组合键 | 用途 / Use | 场景 | 来源 |
| --- | --- | --- | --- |
| `Ctrl` + `C` | 复制 / Copy | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `V` | 粘贴 / Paste | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `X` | 剪切 / Cut | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Z` | 撤销 / Undo | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Y` | 重做 / Redo | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `A` | 全选 / Select all | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `F` | 查找 / Find | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `H` | 替换 / Replace | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `B` | 加粗 / Bold | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `I` | 斜体 / Italic | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `U` | 下划线 / Underline | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Backspace` | 删除前一个词 / Delete preceding word | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Delete` | 删除后一个词 / Delete following word | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Left` | 上一个词 / Previous word | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Right` | 下一个词 / Next word | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Home` | 文档开头 / Document start | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `End` | 文档末尾 / Document end | Apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Shift` + `V` | 粘贴纯文本 / Plain-text paste | Supported apps | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Alt` + `Tab` | 切换窗口 / Switch window | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Alt` + `F4` | 关闭窗口 / Close window | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Alt` + `Space` | 窗口菜单 / Window menu | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Shift` + `Esc` | 任务管理器 / Task Manager | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `E` | 文件资源管理器 / File Explorer | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `D` | 显示桌面 / Show desktop | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `L` | 锁定电脑 / Lock computer | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `I` | 设置 / Settings | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `R` | 运行窗口 / Run dialog | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `S` | 搜索 / Search | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `Tab` | 任务视图 / Task View | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `V` | 剪贴板历史 / Clipboard history | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `Left` | 窗口贴靠左侧 / Snap left | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `Right` | 窗口贴靠右侧 / Snap right | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `Up` | 最大化 / Maximize | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `Down` | 最小化或还原 / Minimize / restore | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `Space` | 切换输入语言 / Next input language | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `.` | 表情面板 / Emoji panel | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Shift` + `Win` + `S` | 区域截图 / Area capture | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Alt` + `PrtSc` | 截取当前窗口 / Capture active window | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `PrtSc` | 保存全屏截图 / Save screen capture | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `Pause` | 系统信息 / System information | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |

### Ubuntu / GNOME

| 组合键 | 用途 / Use | 场景 | 来源 |
| --- | --- | --- | --- |
| `Ctrl` + `A` | 全选 / Select all | Apps | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Ctrl` + `C` | 复制 / Copy | Apps | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Ctrl` + `V` | 粘贴 / Paste | Apps | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Ctrl` + `X` | 剪切 / Cut | Apps | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Ctrl` + `Z` | 撤销 / Undo | Apps | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Super` + `Tab` | 切换应用 / Switch app | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Super` + `` ` `` | 切换应用内窗口 / Switch app window | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Super` + `A` | 应用列表 / App grid | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Super` + `PageUp` | 上一个工作区 / Previous workspace | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Super` + `PageDown` | 下一个工作区 / Next workspace | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Super` + `L` | 锁定屏幕 / Lock screen | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Super` + `V` | 通知列表 / Notifications | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Shift` + `Super` + `Tab` | 上一个应用 / Previous app | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Shift` + `Super` + `PageUp` | 移到上一个工作区 / Move to previous workspace | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Shift` + `Super` + `PageDown` | 移到下一个工作区 / Move to next workspace | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Shift` + `Super` + `Left` | 移到左侧显示器 / Move to left display | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Shift` + `Super` + `Right` | 移到右侧显示器 / Move to right display | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Alt` + `F2` | 运行命令 / Run command | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Alt` + `Esc` | 切换工作区内窗口 / Cycle workspace windows | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Ctrl` + `Alt` + `Tab` | 聚焦顶栏 / Focus top bar | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Ctrl` + `Alt` + `Shift` + `R` | 屏幕录制 / Screen recording | GNOME | [GNOME · Desktop shortcuts](https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html) |
| `Ctrl` + `Shift` + `C` | 复制 / Copy | Terminal | [GNOME · Terminal shortcuts](https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html) |
| `Ctrl` + `Shift` + `V` | 粘贴 / Paste | Terminal | [GNOME · Terminal shortcuts](https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html) |
| `Ctrl` + `Shift` + `T` | 新建标签页 / New tab | Terminal | [GNOME · Terminal shortcuts](https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html) |
| `Ctrl` + `Shift` + `N` | 新建窗口 / New window | Terminal | [GNOME · Terminal shortcuts](https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html) |
| `Ctrl` + `Shift` + `W` | 关闭标签页 / Close tab | Terminal | [GNOME · Terminal shortcuts](https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html) |
| `Ctrl` + `Shift` + `Q` | 关闭窗口 / Close window | Terminal | [GNOME · Terminal shortcuts](https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html) |
| `Ctrl` + `Shift` + `F` | 搜索 / Search | Terminal | [GNOME · Terminal shortcuts](https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html) |
| `Ctrl` + `Shift` + `G` | 下一个匹配 / Next match | Terminal | [GNOME · Terminal shortcuts](https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html) |
| `Ctrl` + `Shift` + `H` | 上一个匹配 / Previous match | Terminal | [GNOME · Terminal shortcuts](https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html) |
| `Ctrl` + `PageUp` | 上一个标签页 / Previous tab | Terminal | [GNOME · Terminal shortcuts](https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html) |
| `Ctrl` + `PageDown` | 下一个标签页 / Next tab | Terminal | [GNOME · Terminal shortcuts](https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html) |
| `Ctrl` + `0` | 重置缩放 / Reset zoom | Terminal | [GNOME · Terminal shortcuts](https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html) |
