# ClickPulse 快捷键百科 / Shortcut Atlas

App 自动显示当前操作系统的快捷键统计与用途注释，并提供网页版百科入口。网页版支持交互式键盘预览、三系统对照、用途筛选与搜索。

来源核对日期为 2026 年 11 月 19 日。

## 如何理解统计

- 新版开始累计快捷键，无法从旧按键总数还原快捷键使用记录。
- 只保存系统、日期、修饰键、主键与按日累计次数，不保存时间顺序、文本或应用名。
- 日期按记录时的系统本地日期划分，与每日总数一致；更改显示时区不重分组快捷键。
- 私有预览版的无日期累计只计入全部历史，不计入今日或日期范围。
- 统计页隐藏仅以 Shift 为修饰键的快捷键，使用次数仍会记录。
- 左右修饰键合并；按住产生的重复事件不重复累计。
- 普通打字不计入快捷键；Mac Option 字符输入与 Windows/Linux AltGr 字符输入排除。
- 小键盘 Enter、减号、等号和加号优先匹配原键，若无词条才查对应主键盘注释。统计按实际按键保留。
- macOS 和 Windows 的快捷键按当前布局识别字母、数字和支持的标点；无法识别时保留物理键位。Linux 按美式物理键位识别。
- 与百科词条匹配只说明按下了相同的快捷键。实际操作取决于应用，部分系统保留的快捷键可能无法记录。
- Linux 词条适用于 Ubuntu 的 GNOME 桌面，其他桌面可有不同默认设置。
- 导入旧备份时，同一日期、来源和快捷键取较高累计值，不同来源相加，避免重复导入增加次数。

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

| 快捷键 | 用途 / Use | 场景 | 来源 |
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
| `⌘` + `O` | 打开文件或所选项目 / Open a file or selection | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `N` | 新建文稿或窗口 / New document or window | Apps | [Apple · Intro to Mac keyboard shortcuts](https://support.apple.com/guide/mac-help/intro-to-mac-keyboard-shortcuts-mchld6b9e240/mac) |
| `⌘` + `T` | 新建标签页 / Open a new tab | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `⌘` + `H` | 隐藏其他应用 / Hide other apps | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `⌘` + `W` | 关闭当前应用的所有窗口 / Close all windows of this app | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `⌘` + `M` | 最小化当前应用的所有窗口 / Minimize all windows of this app | Apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `R` | 刷新或重新载入 / Refresh or reload | Supported apps | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `⌘` + `Esc` | 强制退出应用 / Force quit an app | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `Q` | 确认后退出登录 / Log out with confirmation | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `` ` `` | 切换到当前应用的上一个窗口 / Previous window of this app | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `Space` | 切换到上一个输入法 / Previous input source | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `Option` + `Space` | 切换到下一个输入法 / Next input source | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `Up` | 打开调度中心 / Open Mission Control | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `Down` | 显示前台应用的所有窗口 / Show windows of the front app | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `⌘` + `D` | 显示或隐藏程序坞 / Show or hide the Dock | macOS | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `Shift` + `⌘` + `3` | 将全屏截图复制到剪贴板 / Copy full screenshot to clipboard | macOS | [Apple · Screenshots](https://support.apple.com/en-us/102646) |
| `Ctrl` + `Shift` + `⌘` + `4` | 截图 / Screenshot | macOS | [Apple · Screenshots](https://support.apple.com/en-us/102646) |
| `⌘` + `B` | 切换粗体 / Toggle bold | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `I` | 切换斜体 / Toggle italics | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `U` | 切换下划线 / Toggle underline | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `K` | 插入链接 / Insert a link | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `T` | 显示或隐藏字体窗口 / Show or hide fonts | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `S` | 另存为或创建文稿副本 / Save as or duplicate the document | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `⌘` + `F` | 移到搜索栏 / Focus the search field | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `[` | 左对齐 / Align left | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `]` | 右对齐 / Align right | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `/` | 打开帮助菜单 / Open the Help menu | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `⌘` + `C` | 拷贝样式 / Copy formatting | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `⌘` + `V` | 粘贴样式 / Paste formatting | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `Shift` + `⌘` + `V` | 粘贴并匹配周围文本样式 / Paste and match surrounding style | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `Up` | 选至文稿开头 / Select to document start | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `Down` | 选至文稿末尾 / Select to document end | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `Left` | 选至行首 / Select to line start | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `Right` | 选至行尾 / Select to line end | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `Left` | 移到上一词的开头 / Move to the previous word boundary | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `Right` | 移到下一词的末尾 / Move to the next word boundary | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `Shift` + `Left` | 向左按词扩选 / Extend selection left by word | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `Shift` + `Right` | 向右按词扩选 / Extend selection right by word | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `Backspace` | 删除左侧的词 / Delete the word to the left | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `A` | 移到行首或段首 / Move to line or paragraph start | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `E` | 移到行尾或段尾 / Move to line or paragraph end | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `K` | 剪切选区或剪至段尾 / Cut selection or text to paragraph end | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `D` | 删除后一个字符 / Delete the next character | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `H` | 删除前一个字符 / Delete the previous character | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `F` | 向前移动一个字符 / Move forward one character | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `B` | 向后移动一个字符 / Move back one character | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `P` | 上移一行 / Move up one line | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `N` | 下移一行 / Move down one line | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `O` | 在插入点后换行 / Insert a line after the caret | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `T` | 交换插入点两侧的字符 / Transpose adjacent characters | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `⌘` + `D` | 查询所选词的释义 / Look up selected word | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `=` | 放大所选项目 / Increase selection size | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `=` | 放大所选项目 / Increase selection size | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `-` | 缩小所选项目 / Decrease selection size | Text editors | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `-` | 缩小网页 / Zoom out | Safari | [Apple · Browse the web in Safari](https://support.apple.com/guide/mac-help/browse-the-web-mchlgtd_web01/mac) |
| `⌘` + `0` | 恢复网页默认大小 / Reset page zoom | Safari | [Apple · Browse the web in Safari](https://support.apple.com/guide/mac-help/browse-the-web-mchlgtd_web01/mac) |
| `Shift` + `⌘` + `=` | 放大网页 / Zoom in | Safari | [Apple · Browse the web in Safari](https://support.apple.com/guide/mac-help/browse-the-web-mchlgtd_web01/mac) |
| `⌘` + `R` | 显示所选替身的原始文件 / Reveal original of selected alias | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `T` | 单个标签页时切换标签栏 / Toggle tab bar with one tab open | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `E` | 推出所选磁盘 / Eject selected volume | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `J` | 显示查看选项 / Show view options | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `Y` | 快速查看所选文件 / Preview selected files | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `/` | 显示或隐藏状态栏 / Show or hide status bar | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `1` | 图标视图 / Icon view | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `2` | 列表视图 / List view | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `3` | 分栏视图 / Column view | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `4` | 画廊视图 / Gallery view | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `[` | 返回上一个文件夹 / Go to previous folder | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `⌘` + `]` | 前往下一个文件夹 / Go to next folder | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `D` | 打开桌面文件夹 / Open Desktop folder | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `O` | 打开文稿文件夹 / Open Documents folder | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `F` | 打开最近使用 / Open Recents | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `R` | 打开隔空投送 / Open AirDrop | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `U` | 打开实用工具 / Open Utilities | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `I` | 打开 iCloud 云盘 / Open iCloud Drive | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `K` | 打开网络窗口 / Open Network | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `⌘` + `L` | 打开下载文件夹 / Open Downloads | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Shift` + `⌘` + `Backspace` | 清倒废纸篓 / Empty Trash | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `⌘` + `S` | 显示或隐藏边栏 / Show or hide sidebar | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Option` + `⌘` + `P` | 显示或隐藏路径栏 / Show or hide path bar | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |
| `Ctrl` + `⌘` + `A` | 制作替身 / Create an alias | Finder | [Apple · Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |

### Windows

| 快捷键 | 用途 / Use | 场景 | 来源 |
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
| `Win` + `M` | 最小化所有窗口 / Minimize all windows | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `Home` | 最小化或还原其他窗口 / Toggle other windows minimized | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Shift` + `Win` + `Left` | 将窗口移到左侧显示器 / Move window to left monitor | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Shift` + `Win` + `Right` | 将窗口移到右侧显示器 / Move window to right monitor | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Win` + `D` | 新建虚拟桌面 / Create a virtual desktop | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Win` + `F4` | 关闭当前虚拟桌面 / Close current virtual desktop | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Win` + `Left` | 切到左侧虚拟桌面 / Switch to desktop on the left | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Win` + `Right` | 切到右侧虚拟桌面 / Switch to desktop on the right | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `A` | 打开操作中心或快速设置 / Open action center or quick settings | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `N` | 打开通知中心和日历 / Open notifications and calendar | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `H` | 启动语音输入 / Start voice typing | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `X` | 打开快速链接菜单 / Open Quick Link menu | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `P` | 选择投影模式 / Choose display projection mode | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `K` | 连接无线显示器 / Connect to a wireless display | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `G` | 打开游戏栏 / Open Game Bar | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Shift` + `Win` + `=` | 用放大镜放大 / Zoom in with Magnifier | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `-` | 用放大镜缩小 / Zoom out with Magnifier | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `Esc` | 关闭放大镜 / Close Magnifier | Windows | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Alt` + `Win` + `R` | 开始或停止录制 / Start or stop recording | Game Bar | [Microsoft · Record with Xbox Game Bar](https://support.microsoft.com/en-us/accessibility/windows/use-a-screen-reader-to-record-your-screen-with-xbox-game-bar) |
| `Ctrl` + `Shift` + `N` | 新建文件夹 / Create a folder | File Explorer | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Alt` + `Enter` | 显示所选项目的属性 / Show item properties | File Explorer | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Alt` + `Left` | 返回上一个文件夹 / Go to previous folder | File Explorer | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Alt` + `Right` | 前往下一个文件夹 / Go to next folder | File Explorer | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Alt` + `Up` | 打开上级文件夹 / Open parent folder | File Explorer | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Tab` | 下一个标签页 / Next tab | File Explorer | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Shift` + `Tab` | 上一个标签页 / Previous tab | File Explorer | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Tab` | 切到对话框的下一个选项卡 / Next tab in dialog | Dialogs | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Shift` + `Tab` | 切到对话框的上一个选项卡 / Previous tab in dialog | Dialogs | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Shift` + `Left` | 向左按词扩选 / Extend selection one word left | Text editors | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `Shift` + `Right` | 向右按词扩选 / Extend selection one word right | Text editors | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Ctrl` + `S` | 保存当前网页 / Save this page | Microsoft Edge | [Microsoft · Edge keyboard shortcuts](https://support.microsoft.com/en-us/edge/keyboard-shortcuts-in-microsoft-edge) |
| `Ctrl` + `N` | 新建浏览器窗口 / New browser window | Microsoft Edge | [Microsoft · Edge keyboard shortcuts](https://support.microsoft.com/en-us/edge/keyboard-shortcuts-in-microsoft-edge) |
| `Ctrl` + `O` | 打开本地文件 / Open a local file | Microsoft Edge | [Microsoft · Edge keyboard shortcuts](https://support.microsoft.com/en-us/edge/keyboard-shortcuts-in-microsoft-edge) |
| `Ctrl` + `P` | 打印当前网页 / Print this page | Microsoft Edge | [Microsoft · Edge keyboard shortcuts](https://support.microsoft.com/en-us/edge/keyboard-shortcuts-in-microsoft-edge) |
| `Ctrl` + `W` | 关闭当前标签页 / Close this tab | Microsoft Edge | [Microsoft · Edge keyboard shortcuts](https://support.microsoft.com/en-us/edge/keyboard-shortcuts-in-microsoft-edge) |
| `Ctrl` + `T` | 打开新标签页 / Open a new tab | Microsoft Edge | [Microsoft · Edge keyboard shortcuts](https://support.microsoft.com/en-us/edge/keyboard-shortcuts-in-microsoft-edge) |
| `Ctrl` + `Shift` + `=` | 放大网页 / Zoom in | Microsoft Edge | [Microsoft · Edge keyboard shortcuts](https://support.microsoft.com/en-us/edge/keyboard-shortcuts-in-microsoft-edge) |
| `Ctrl` + `-` | 缩小网页 / Zoom out | Microsoft Edge | [Microsoft · Edge keyboard shortcuts](https://support.microsoft.com/en-us/edge/keyboard-shortcuts-in-microsoft-edge) |
| `Ctrl` + `0` | 重置网页缩放 / Reset page zoom | Microsoft Edge | [Microsoft · Edge keyboard shortcuts](https://support.microsoft.com/en-us/edge/keyboard-shortcuts-in-microsoft-edge) |
| `Win` + `0` | 打开或切到任务栏中数字 0 对应的应用 / Open or switch to the taskbar app assigned to 0 | Taskbar | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `1` | 打开或切到任务栏第 1 个固定应用 / Open or switch to pinned app 1 | Taskbar | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `2` | 打开或切到任务栏第 2 个固定应用 / Open or switch to pinned app 2 | Taskbar | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `3` | 打开或切到任务栏第 3 个固定应用 / Open or switch to pinned app 3 | Taskbar | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `4` | 打开或切到任务栏第 4 个固定应用 / Open or switch to pinned app 4 | Taskbar | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `5` | 打开或切到任务栏第 5 个固定应用 / Open or switch to pinned app 5 | Taskbar | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `6` | 打开或切到任务栏第 6 个固定应用 / Open or switch to pinned app 6 | Taskbar | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `7` | 打开或切到任务栏第 7 个固定应用 / Open or switch to pinned app 7 | Taskbar | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `8` | 打开或切到任务栏第 8 个固定应用 / Open or switch to pinned app 8 | Taskbar | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |
| `Win` + `9` | 打开或切到任务栏第 9 个固定应用 / Open or switch to pinned app 9 | Taskbar | [Microsoft · Windows keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows) |

### Ubuntu / GNOME

| 快捷键 | 用途 / Use | 场景 | 来源 |
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
| `Super` + `H` | 隐藏窗口 / Hide window | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Super` + `Up` | 最大化窗口 / Maximize window | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Super` + `Down` | 还原窗口 / Restore window | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Super` + `Left` | 将窗口平铺到左侧 / Tile window to the left | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Super` + `Right` | 将窗口平铺到右侧 / Tile window to the right | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Super` + `Space` | 下一个输入法 / Next input source | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Shift` + `Super` + `Space` | 上一个输入法 / Previous input source | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Super` + `N` | 聚焦当前通知 / Focus active notification | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Super` + `S` | 打开系统菜单 / Open system menu | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Super` + `Home` | 切换到第一个工作区 / Go to first workspace | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Super` + `End` | 切换到最后一个工作区 / Go to last workspace | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Shift` + `Super` + `Up` | 将窗口移到上方显示器 / Move window to monitor above | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Shift` + `Super` + `Down` | 将窗口移到下方显示器 / Move window to monitor below | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Shift` + `PrtSc` | 保存全屏截图 / Save full-screen screenshot | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
| `Alt` + `PrtSc` | 保存窗口截图 / Save window screenshot | GNOME | [GNOME · Keyboard shortcut settings](https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html) |
