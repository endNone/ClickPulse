'use strict';
// Curated reference data, shared by the offline App and static handbook.
const ClickPulseShortcutCatalog=(()=>{
const sources={
  "apple": {
    "name": "Apple · Mac keyboard shortcuts",
    "url": "https://support.apple.com/en-us/102650"
  },
  "apple-screenshots": {
    "name": "Apple · Screenshots",
    "url": "https://support.apple.com/en-us/102646"
  },
  "windows": {
    "name": "Microsoft · Windows keyboard shortcuts",
    "url": "https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows"
  },
  "gnome": {
    "name": "GNOME · Desktop shortcuts",
    "url": "https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html"
  },
  "gnome-terminal": {
    "name": "GNOME · Terminal shortcuts",
    "url": "https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html"
  },
  "mac-guide": {
    "name": "Apple · Intro to Mac keyboard shortcuts",
    "url": "https://support.apple.com/guide/mac-help/intro-to-mac-keyboard-shortcuts-mchld6b9e240/mac"
  },
  "apple-safari": {
    "name": "Apple · Browse the web in Safari",
    "url": "https://support.apple.com/guide/mac-help/browse-the-web-mchlgtd_web01/mac"
  },
  "ms-edge": {
    "name": "Microsoft · Edge keyboard shortcuts",
    "url": "https://support.microsoft.com/en-us/edge/keyboard-shortcuts-in-microsoft-edge"
  },
  "ms-game": {
    "name": "Microsoft · Record with Xbox Game Bar",
    "url": "https://support.microsoft.com/en-us/accessibility/windows/use-a-screen-reader-to-record-your-screen-with-xbox-game-bar"
  },
  "gnome-settings": {
    "name": "GNOME · Keyboard shortcut settings",
    "url": "https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html"
  }
};
const keyLabels={"0": "A", "1": "S", "2": "D", "3": "F", "4": "H", "5": "G", "6": "Z", "7": "X", "8": "C", "9": "V", "11": "B", "12": "Q", "13": "W", "14": "E", "15": "R", "16": "Y", "17": "T", "18": "1", "19": "2", "20": "3", "21": "4", "22": "6", "23": "5", "24": "=", "25": "9", "26": "7", "27": "-", "28": "8", "29": "0", "30": "]", "31": "O", "32": "U", "33": "[", "34": "I", "35": "P", "36": "Enter", "37": "L", "38": "J", "39": "'", "40": "K", "41": ";", "42": "\\", "43": ",", "44": "/", "45": "N", "46": "M", "47": ".", "48": "Tab", "49": "Space", "50": "`", "51": "Backspace", "53": "Esc", "122": "F1", "120": "F2", "99": "F3", "118": "F4", "96": "F5", "97": "F6", "98": "F7", "100": "F8", "101": "F9", "109": "F10", "103": "F11", "111": "F12", "115": "Home", "116": "PageUp", "117": "Delete", "119": "End", "121": "PageDown", "123": "Left", "124": "Right", "125": "Down", "126": "Up"};
Object.assign(keyLabels, {10:'ISO §',64:'F17',65:'Num .',67:'Num ×',69:'Num +',71:'Num Clear',72:'Volume Up',73:'Volume Down',74:'Mute',75:'Num ÷',76:'Num Enter',78:'Num −',79:'F18',80:'F19',81:'Num =',82:'Num 0',83:'Num 1',84:'Num 2',85:'Num 3',86:'Num 4',87:'Num 5',88:'Num 6',89:'Num 7',90:'F20',91:'Num 8',92:'Num 9',93:'JIS ¥',94:'JIS _',95:'Num ,',102:'Eisu',104:'Kana',105:'F13',106:'F16',107:'F14',113:'F15',114:'Insert'});
const entries=[
  {
    "id": "macos-8-8-apps",
    "platform": "macos",
    "modifiers": 8,
    "key": 8,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Copy",
      "zh": "复制",
      "zh-TW": "複製"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-9-apps",
    "platform": "macos",
    "modifiers": 8,
    "key": 9,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Paste",
      "zh": "粘贴",
      "zh-TW": "貼上"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-7-apps",
    "platform": "macos",
    "modifiers": 8,
    "key": 7,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Cut",
      "zh": "剪切",
      "zh-TW": "剪下"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-6-apps",
    "platform": "macos",
    "modifiers": 8,
    "key": 6,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Undo",
      "zh": "撤销",
      "zh-TW": "還原"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-0-apps",
    "platform": "macos",
    "modifiers": 8,
    "key": 0,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Select all",
      "zh": "全选",
      "zh-TW": "全選"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-1-apps",
    "platform": "macos",
    "modifiers": 8,
    "key": 1,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Save",
      "zh": "保存",
      "zh-TW": "儲存"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-3-apps",
    "platform": "macos",
    "modifiers": 8,
    "key": 3,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Find",
      "zh": "查找",
      "zh-TW": "尋找"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-5-apps",
    "platform": "macos",
    "modifiers": 8,
    "key": 5,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Next match",
      "zh": "下一个匹配",
      "zh-TW": "下一個符合項目"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-35-apps",
    "platform": "macos",
    "modifiers": 8,
    "key": 35,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Print",
      "zh": "打印",
      "zh-TW": "列印"
    },
    "source": "apple"
  },
  {
    "id": "macos-12-6-apps",
    "platform": "macos",
    "modifiers": 12,
    "key": 6,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Redo",
      "zh": "重做",
      "zh-TW": "重做"
    },
    "source": "apple"
  },
  {
    "id": "macos-12-5-apps",
    "platform": "macos",
    "modifiers": 12,
    "key": 5,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Previous match",
      "zh": "上一个匹配",
      "zh-TW": "上一個符合項目"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-12-macos",
    "platform": "macos",
    "modifiers": 8,
    "key": 12,
    "scope": "macOS",
    "category": "system",
    "title": {
      "en": "Quit app",
      "zh": "退出应用",
      "zh-TW": "結束應用程式"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-13-macos",
    "platform": "macos",
    "modifiers": 8,
    "key": 13,
    "scope": "macOS",
    "category": "system",
    "title": {
      "en": "Close window",
      "zh": "关闭窗口",
      "zh-TW": "關閉視窗"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-4-macos",
    "platform": "macos",
    "modifiers": 8,
    "key": 4,
    "scope": "macOS",
    "category": "system",
    "title": {
      "en": "Hide app",
      "zh": "隐藏应用",
      "zh-TW": "隱藏應用程式"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-46-macos",
    "platform": "macos",
    "modifiers": 8,
    "key": 46,
    "scope": "macOS",
    "category": "system",
    "title": {
      "en": "Minimize",
      "zh": "最小化",
      "zh-TW": "最小化"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-48-macos",
    "platform": "macos",
    "modifiers": 8,
    "key": 48,
    "scope": "macOS",
    "category": "system",
    "title": {
      "en": "Switch app",
      "zh": "切换应用",
      "zh-TW": "切換應用程式"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-49-macos",
    "platform": "macos",
    "modifiers": 8,
    "key": 49,
    "scope": "macOS",
    "category": "system",
    "title": {
      "en": "Spotlight",
      "zh": "聚焦搜索",
      "zh-TW": "聚焦搜尋"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-50-macos",
    "platform": "macos",
    "modifiers": 8,
    "key": 50,
    "scope": "macOS",
    "category": "system",
    "title": {
      "en": "Switch app window",
      "zh": "切换应用内窗口",
      "zh-TW": "切換應用程式內的視窗"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-43-apps",
    "platform": "macos",
    "modifiers": 8,
    "key": 43,
    "scope": "Apps",
    "category": "system",
    "title": {
      "en": "App settings",
      "zh": "应用设置",
      "zh-TW": "應用程式設定"
    },
    "source": "apple"
  },
  {
    "id": "macos-12-20-macos",
    "platform": "macos",
    "modifiers": 12,
    "key": 20,
    "scope": "macOS",
    "category": "capture",
    "title": {
      "en": "Screen capture",
      "zh": "全屏截图",
      "zh-TW": "全螢幕截圖"
    },
    "source": "apple"
  },
  {
    "id": "macos-12-21-macos",
    "platform": "macos",
    "modifiers": 12,
    "key": 21,
    "scope": "macOS",
    "category": "capture",
    "title": {
      "en": "Area capture",
      "zh": "区域截图",
      "zh-TW": "區域截圖"
    },
    "source": "apple"
  },
  {
    "id": "macos-12-23-macos",
    "platform": "macos",
    "modifiers": 12,
    "key": 23,
    "scope": "macOS",
    "category": "capture",
    "title": {
      "en": "Capture controls",
      "zh": "截屏工具",
      "zh-TW": "截圖工具"
    },
    "source": "apple"
  },
  {
    "id": "macos-9-3-macos",
    "platform": "macos",
    "modifiers": 9,
    "key": 3,
    "scope": "macOS",
    "category": "system",
    "title": {
      "en": "Toggle fullscreen",
      "zh": "切换全屏",
      "zh-TW": "切換全螢幕"
    },
    "source": "apple"
  },
  {
    "id": "macos-9-49-macos",
    "platform": "macos",
    "modifiers": 9,
    "key": 49,
    "scope": "macOS",
    "category": "system",
    "title": {
      "en": "Character viewer",
      "zh": "字符检视器",
      "zh-TW": "字元檢視器"
    },
    "source": "apple"
  },
  {
    "id": "macos-9-12-macos",
    "platform": "macos",
    "modifiers": 9,
    "key": 12,
    "scope": "macOS",
    "category": "system",
    "title": {
      "en": "Lock screen",
      "zh": "锁定屏幕",
      "zh-TW": "鎖定螢幕"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-45-finder",
    "platform": "macos",
    "modifiers": 8,
    "key": 45,
    "scope": "Finder",
    "category": "files",
    "title": {
      "en": "New window",
      "zh": "新建窗口",
      "zh-TW": "新增視窗"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-31-finder",
    "platform": "macos",
    "modifiers": 8,
    "key": 31,
    "scope": "Finder",
    "category": "files",
    "title": {
      "en": "Open selection",
      "zh": "打开所选项目",
      "zh-TW": "開啟所選項目"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-34-finder",
    "platform": "macos",
    "modifiers": 8,
    "key": 34,
    "scope": "Finder",
    "category": "files",
    "title": {
      "en": "Item information",
      "zh": "项目信息",
      "zh-TW": "項目資訊"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-51-finder",
    "platform": "macos",
    "modifiers": 8,
    "key": 51,
    "scope": "Finder",
    "category": "files",
    "title": {
      "en": "Move to Trash",
      "zh": "移到废纸篓",
      "zh-TW": "移到垃圾桶"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-2-finder",
    "platform": "macos",
    "modifiers": 8,
    "key": 2,
    "scope": "Finder",
    "category": "files",
    "title": {
      "en": "Duplicate item",
      "zh": "复制项目",
      "zh-TW": "複製項目"
    },
    "source": "apple"
  },
  {
    "id": "macos-12-45-finder",
    "platform": "macos",
    "modifiers": 12,
    "key": 45,
    "scope": "Finder",
    "category": "files",
    "title": {
      "en": "New folder",
      "zh": "新建文件夹",
      "zh-TW": "新增檔案夾"
    },
    "source": "apple"
  },
  {
    "id": "macos-12-5-finder",
    "platform": "macos",
    "modifiers": 12,
    "key": 5,
    "scope": "Finder",
    "category": "files",
    "title": {
      "en": "Go to folder",
      "zh": "前往文件夹",
      "zh-TW": "前往檔案夾"
    },
    "source": "apple"
  },
  {
    "id": "macos-12-0-finder",
    "platform": "macos",
    "modifiers": 12,
    "key": 0,
    "scope": "Finder",
    "category": "files",
    "title": {
      "en": "Applications folder",
      "zh": "应用程序文件夹",
      "zh-TW": "應用程式檔案夾"
    },
    "source": "apple"
  },
  {
    "id": "macos-12-4-finder",
    "platform": "macos",
    "modifiers": 12,
    "key": 4,
    "scope": "Finder",
    "category": "files",
    "title": {
      "en": "Home folder",
      "zh": "个人文件夹",
      "zh-TW": "個人檔案夾"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-123-text-editors",
    "platform": "macos",
    "modifiers": 8,
    "key": 123,
    "scope": "Text editors",
    "category": "editing",
    "title": {
      "en": "Line start",
      "zh": "行首",
      "zh-TW": "行首"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-124-text-editors",
    "platform": "macos",
    "modifiers": 8,
    "key": 124,
    "scope": "Text editors",
    "category": "editing",
    "title": {
      "en": "Line end",
      "zh": "行尾",
      "zh-TW": "行尾"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-126-text-editors",
    "platform": "macos",
    "modifiers": 8,
    "key": 126,
    "scope": "Text editors",
    "category": "editing",
    "title": {
      "en": "Document start",
      "zh": "文稿开头",
      "zh-TW": "文件開頭"
    },
    "source": "apple"
  },
  {
    "id": "macos-8-125-text-editors",
    "platform": "macos",
    "modifiers": 8,
    "key": 125,
    "scope": "Text editors",
    "category": "editing",
    "title": {
      "en": "Document end",
      "zh": "文稿末尾",
      "zh-TW": "文件結尾"
    },
    "source": "apple"
  },
  {
    "id": "windows-1-8-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 8,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Copy",
      "zh": "复制",
      "zh-TW": "複製"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-9-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 9,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Paste",
      "zh": "粘贴",
      "zh-TW": "貼上"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-7-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 7,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Cut",
      "zh": "剪切",
      "zh-TW": "剪下"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-6-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 6,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Undo",
      "zh": "撤销",
      "zh-TW": "還原"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-16-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 16,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Redo",
      "zh": "重做",
      "zh-TW": "重做"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-0-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 0,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Select all",
      "zh": "全选",
      "zh-TW": "全選"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-3-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 3,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Find",
      "zh": "查找",
      "zh-TW": "尋找"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-4-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 4,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Replace",
      "zh": "替换",
      "zh-TW": "取代"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-11-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 11,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Bold",
      "zh": "加粗",
      "zh-TW": "粗體"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-34-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 34,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Italic",
      "zh": "斜体",
      "zh-TW": "斜體"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-32-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 32,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Underline",
      "zh": "下划线",
      "zh-TW": "底線"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-51-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 51,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Delete preceding word",
      "zh": "删除前一个词",
      "zh-TW": "刪除前一個字詞"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-117-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 117,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Delete following word",
      "zh": "删除后一个词",
      "zh-TW": "刪除後一個字詞"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-123-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 123,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Previous word",
      "zh": "上一个词",
      "zh-TW": "上一個字詞"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-124-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 124,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Next word",
      "zh": "下一个词",
      "zh-TW": "下一個字詞"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-115-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 115,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Document start",
      "zh": "文档开头",
      "zh-TW": "文件開頭"
    },
    "source": "windows"
  },
  {
    "id": "windows-1-119-apps",
    "platform": "windows",
    "modifiers": 1,
    "key": 119,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Document end",
      "zh": "文档末尾",
      "zh-TW": "文件結尾"
    },
    "source": "windows"
  },
  {
    "id": "windows-5-9-supported-apps",
    "platform": "windows",
    "modifiers": 5,
    "key": 9,
    "scope": "Supported apps",
    "category": "editing",
    "title": {
      "en": "Plain-text paste",
      "zh": "粘贴纯文本",
      "zh-TW": "貼上純文字"
    },
    "source": "windows"
  },
  {
    "id": "windows-2-48-windows",
    "platform": "windows",
    "modifiers": 2,
    "key": 48,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Switch window",
      "zh": "切换窗口",
      "zh-TW": "切換視窗"
    },
    "source": "windows"
  },
  {
    "id": "windows-2-118-windows",
    "platform": "windows",
    "modifiers": 2,
    "key": 118,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Close window",
      "zh": "关闭窗口",
      "zh-TW": "關閉視窗"
    },
    "source": "windows"
  },
  {
    "id": "windows-2-49-windows",
    "platform": "windows",
    "modifiers": 2,
    "key": 49,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Window menu",
      "zh": "窗口菜单",
      "zh-TW": "視窗選單"
    },
    "source": "windows"
  },
  {
    "id": "windows-5-53-windows",
    "platform": "windows",
    "modifiers": 5,
    "key": 53,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Task Manager",
      "zh": "任务管理器",
      "zh-TW": "工作管理員"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-14-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 14,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "File Explorer",
      "zh": "文件资源管理器",
      "zh-TW": "檔案總管"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-2-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 2,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Show desktop",
      "zh": "显示桌面",
      "zh-TW": "顯示桌面"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-37-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 37,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Lock computer",
      "zh": "锁定电脑",
      "zh-TW": "鎖定電腦"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-34-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 34,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Settings",
      "zh": "设置",
      "zh-TW": "設定"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-15-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 15,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Run dialog",
      "zh": "运行窗口",
      "zh-TW": "執行視窗"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-1-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 1,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Search",
      "zh": "搜索",
      "zh-TW": "搜尋"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-48-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 48,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Task View",
      "zh": "任务视图",
      "zh-TW": "工作檢視"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-9-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 9,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Clipboard history",
      "zh": "剪贴板历史",
      "zh-TW": "剪貼簿歷程記錄"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-123-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 123,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Snap left",
      "zh": "窗口贴靠左侧",
      "zh-TW": "視窗貼齊左側"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-124-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 124,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Snap right",
      "zh": "窗口贴靠右侧",
      "zh-TW": "視窗貼齊右側"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-126-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 126,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Maximize",
      "zh": "最大化",
      "zh-TW": "最大化"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-125-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 125,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Minimize / restore",
      "zh": "最小化或还原",
      "zh-TW": "最小化或還原"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-49-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 49,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Next input language",
      "zh": "切换输入语言",
      "zh-TW": "切換輸入語言"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-47-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 47,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "Emoji panel",
      "zh": "表情面板",
      "zh-TW": "表情符號面板"
    },
    "source": "windows"
  },
  {
    "id": "windows-12-1-windows",
    "platform": "windows",
    "modifiers": 12,
    "key": 1,
    "scope": "Windows",
    "category": "capture",
    "title": {
      "en": "Area capture",
      "zh": "区域截图",
      "zh-TW": "區域截圖"
    },
    "source": "windows"
  },
  {
    "id": "linux-1-0-apps",
    "platform": "linux",
    "modifiers": 1,
    "key": 0,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Select all",
      "zh": "全选",
      "zh-TW": "全選"
    },
    "source": "gnome"
  },
  {
    "id": "linux-1-8-apps",
    "platform": "linux",
    "modifiers": 1,
    "key": 8,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Copy",
      "zh": "复制",
      "zh-TW": "複製"
    },
    "source": "gnome"
  },
  {
    "id": "linux-1-9-apps",
    "platform": "linux",
    "modifiers": 1,
    "key": 9,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Paste",
      "zh": "粘贴",
      "zh-TW": "貼上"
    },
    "source": "gnome"
  },
  {
    "id": "linux-1-7-apps",
    "platform": "linux",
    "modifiers": 1,
    "key": 7,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Cut",
      "zh": "剪切",
      "zh-TW": "剪下"
    },
    "source": "gnome"
  },
  {
    "id": "linux-1-6-apps",
    "platform": "linux",
    "modifiers": 1,
    "key": 6,
    "scope": "Apps",
    "category": "editing",
    "title": {
      "en": "Undo",
      "zh": "撤销",
      "zh-TW": "還原"
    },
    "source": "gnome"
  },
  {
    "id": "linux-8-48-gnome",
    "platform": "linux",
    "modifiers": 8,
    "key": 48,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Switch app",
      "zh": "切换应用",
      "zh-TW": "切換應用程式"
    },
    "source": "gnome"
  },
  {
    "id": "linux-8-50-gnome",
    "platform": "linux",
    "modifiers": 8,
    "key": 50,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Switch app window",
      "zh": "切换应用内窗口",
      "zh-TW": "切換應用程式內的視窗"
    },
    "source": "gnome"
  },
  {
    "id": "linux-8-0-gnome",
    "platform": "linux",
    "modifiers": 8,
    "key": 0,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "App grid",
      "zh": "应用列表",
      "zh-TW": "應用程式清單"
    },
    "source": "gnome"
  },
  {
    "id": "linux-8-116-gnome",
    "platform": "linux",
    "modifiers": 8,
    "key": 116,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Previous workspace",
      "zh": "上一个工作区",
      "zh-TW": "上一個工作區"
    },
    "source": "gnome"
  },
  {
    "id": "linux-8-121-gnome",
    "platform": "linux",
    "modifiers": 8,
    "key": 121,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Next workspace",
      "zh": "下一个工作区",
      "zh-TW": "下一個工作區"
    },
    "source": "gnome"
  },
  {
    "id": "linux-8-37-gnome",
    "platform": "linux",
    "modifiers": 8,
    "key": 37,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Lock screen",
      "zh": "锁定屏幕",
      "zh-TW": "鎖定螢幕"
    },
    "source": "gnome"
  },
  {
    "id": "linux-8-9-gnome",
    "platform": "linux",
    "modifiers": 8,
    "key": 9,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Notifications",
      "zh": "通知列表",
      "zh-TW": "通知清單"
    },
    "source": "gnome"
  },
  {
    "id": "linux-12-48-gnome",
    "platform": "linux",
    "modifiers": 12,
    "key": 48,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Previous app",
      "zh": "上一个应用",
      "zh-TW": "上一個應用程式"
    },
    "source": "gnome"
  },
  {
    "id": "linux-12-116-gnome",
    "platform": "linux",
    "modifiers": 12,
    "key": 116,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Move to previous workspace",
      "zh": "移到上一个工作区",
      "zh-TW": "移到上一個工作區"
    },
    "source": "gnome"
  },
  {
    "id": "linux-12-121-gnome",
    "platform": "linux",
    "modifiers": 12,
    "key": 121,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Move to next workspace",
      "zh": "移到下一个工作区",
      "zh-TW": "移到下一個工作區"
    },
    "source": "gnome"
  },
  {
    "id": "linux-12-123-gnome",
    "platform": "linux",
    "modifiers": 12,
    "key": 123,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Move to left display",
      "zh": "移到左侧显示器",
      "zh-TW": "移到左側顯示器"
    },
    "source": "gnome"
  },
  {
    "id": "linux-12-124-gnome",
    "platform": "linux",
    "modifiers": 12,
    "key": 124,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Move to right display",
      "zh": "移到右侧显示器",
      "zh-TW": "移到右側顯示器"
    },
    "source": "gnome"
  },
  {
    "id": "linux-2-120-gnome",
    "platform": "linux",
    "modifiers": 2,
    "key": 120,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Run command",
      "zh": "运行命令",
      "zh-TW": "執行指令"
    },
    "source": "gnome"
  },
  {
    "id": "linux-2-53-gnome",
    "platform": "linux",
    "modifiers": 2,
    "key": 53,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Cycle workspace windows",
      "zh": "切换工作区内窗口",
      "zh-TW": "切換工作區內的視窗"
    },
    "source": "gnome"
  },
  {
    "id": "linux-3-48-gnome",
    "platform": "linux",
    "modifiers": 3,
    "key": 48,
    "scope": "GNOME",
    "category": "system",
    "title": {
      "en": "Focus top bar",
      "zh": "聚焦顶栏",
      "zh-TW": "聚焦頂端列"
    },
    "source": "gnome"
  },
  {
    "id": "linux-7-15-gnome",
    "platform": "linux",
    "modifiers": 7,
    "key": 15,
    "scope": "GNOME",
    "category": "capture",
    "title": {
      "en": "Screen recording",
      "zh": "屏幕录制",
      "zh-TW": "螢幕錄製"
    },
    "source": "gnome"
  },
  {
    "id": "linux-5-8-terminal",
    "platform": "linux",
    "modifiers": 5,
    "key": 8,
    "scope": "Terminal",
    "category": "terminal",
    "title": {
      "en": "Copy",
      "zh": "复制",
      "zh-TW": "複製"
    },
    "source": "gnome-terminal"
  },
  {
    "id": "linux-5-9-terminal",
    "platform": "linux",
    "modifiers": 5,
    "key": 9,
    "scope": "Terminal",
    "category": "terminal",
    "title": {
      "en": "Paste",
      "zh": "粘贴",
      "zh-TW": "貼上"
    },
    "source": "gnome-terminal"
  },
  {
    "id": "linux-5-17-terminal",
    "platform": "linux",
    "modifiers": 5,
    "key": 17,
    "scope": "Terminal",
    "category": "terminal",
    "title": {
      "en": "New tab",
      "zh": "新建标签页",
      "zh-TW": "新增分頁"
    },
    "source": "gnome-terminal"
  },
  {
    "id": "linux-5-45-terminal",
    "platform": "linux",
    "modifiers": 5,
    "key": 45,
    "scope": "Terminal",
    "category": "terminal",
    "title": {
      "en": "New window",
      "zh": "新建窗口",
      "zh-TW": "新增視窗"
    },
    "source": "gnome-terminal"
  },
  {
    "id": "linux-5-13-terminal",
    "platform": "linux",
    "modifiers": 5,
    "key": 13,
    "scope": "Terminal",
    "category": "terminal",
    "title": {
      "en": "Close tab",
      "zh": "关闭标签页",
      "zh-TW": "關閉分頁"
    },
    "source": "gnome-terminal"
  },
  {
    "id": "linux-5-12-terminal",
    "platform": "linux",
    "modifiers": 5,
    "key": 12,
    "scope": "Terminal",
    "category": "terminal",
    "title": {
      "en": "Close window",
      "zh": "关闭窗口",
      "zh-TW": "關閉視窗"
    },
    "source": "gnome-terminal"
  },
  {
    "id": "linux-5-3-terminal",
    "platform": "linux",
    "modifiers": 5,
    "key": 3,
    "scope": "Terminal",
    "category": "terminal",
    "title": {
      "en": "Search",
      "zh": "搜索",
      "zh-TW": "搜尋"
    },
    "source": "gnome-terminal"
  },
  {
    "id": "linux-5-5-terminal",
    "platform": "linux",
    "modifiers": 5,
    "key": 5,
    "scope": "Terminal",
    "category": "terminal",
    "title": {
      "en": "Next match",
      "zh": "下一个匹配",
      "zh-TW": "下一個符合項目"
    },
    "source": "gnome-terminal"
  },
  {
    "id": "linux-5-4-terminal",
    "platform": "linux",
    "modifiers": 5,
    "key": 4,
    "scope": "Terminal",
    "category": "terminal",
    "title": {
      "en": "Previous match",
      "zh": "上一个匹配",
      "zh-TW": "上一個符合項目"
    },
    "source": "gnome-terminal"
  },
  {
    "id": "linux-1-116-terminal",
    "platform": "linux",
    "modifiers": 1,
    "key": 116,
    "scope": "Terminal",
    "category": "terminal",
    "title": {
      "en": "Previous tab",
      "zh": "上一个标签页",
      "zh-TW": "上一個分頁"
    },
    "source": "gnome-terminal"
  },
  {
    "id": "linux-1-121-terminal",
    "platform": "linux",
    "modifiers": 1,
    "key": 121,
    "scope": "Terminal",
    "category": "terminal",
    "title": {
      "en": "Next tab",
      "zh": "下一个标签页",
      "zh-TW": "下一個分頁"
    },
    "source": "gnome-terminal"
  },
  {
    "id": "linux-1-29-terminal",
    "platform": "linux",
    "modifiers": 1,
    "key": 29,
    "scope": "Terminal",
    "category": "terminal",
    "title": {
      "en": "Reset zoom",
      "zh": "重置缩放",
      "zh-TW": "重設縮放"
    },
    "source": "gnome-terminal"
  }
];

Object.assign(keyLabels, {65847:'PrtSc',65606:'Scroll Lock',66117:'Pause'});

entries.push(...[
  {
    "id": "windows-2-65847-windows",
    "platform": "windows",
    "modifiers": 2,
    "key": 65847,
    "scope": "Windows",
    "category": "capture",
    "title": {
      "en": "Capture active window",
      "zh": "截取当前窗口",
      "zh-TW": "擷取目前視窗"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-65847-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 65847,
    "scope": "Windows",
    "category": "capture",
    "title": {
      "en": "Save screen capture",
      "zh": "保存全屏截图",
      "zh-TW": "儲存全螢幕截圖"
    },
    "source": "windows"
  },
  {
    "id": "windows-8-66117-windows",
    "platform": "windows",
    "modifiers": 8,
    "key": 66117,
    "scope": "Windows",
    "category": "system",
    "title": {
      "en": "System information",
      "zh": "系统信息",
      "zh-TW": "系統資訊"
    },
    "source": "windows"
  }
]);

// Each group shares a documented platform, context, source and category.
function addReferences(platform, scope, source, category, rows) {
  const suffix = scope.toLowerCase().replaceAll(' ', '-');
  for (const [modifiers, key, en, zh, traditional] of rows) {
    entries.push({
      id: [platform, modifiers, key, suffix].join('-'), platform, modifiers, key, scope, source, category,
      title: { en, zh, 'zh-TW': traditional }
    });
  }
}

addReferences("macos", "Apps", "apple", "files", [
  [8, 31, "Open a file or selection", "打开文件或所选项目", "打開檔案或所選項目"]
]);

addReferences("macos", "Apps", "mac-guide", "files", [
  [8, 45, "New document or window", "新建文稿或窗口", "新增文件或視窗"]
]);

addReferences("macos", "Apps", "apple", "system", [
  [8, 17, "Open a new tab", "新建标签页", "新增標籤頁"],
  [10, 4, "Hide other apps", "隐藏其他应用", "隱藏其他 App"],
  [10, 13, "Close all windows of this app", "关闭当前应用的所有窗口", "關閉目前 App 的所有視窗"],
  [10, 46, "Minimize all windows of this app", "最小化当前应用的所有窗口", "將目前 App 的所有視窗縮到最小"]
]);

addReferences("macos", "Supported apps", "apple", "system", [
  [8, 15, "Refresh or reload", "刷新或重新载入", "重新整理或重新載入"]
]);

addReferences("macos", "macOS", "apple", "system", [
  [10, 53, "Force quit an app", "强制退出应用", "強制結束 App"],
  [12, 12, "Log out with confirmation", "确认后退出登录", "確認後登出"],
  [12, 50, "Previous window of this app", "切换到当前应用的上一个窗口", "切換到目前 App 的上一個視窗"],
  [1, 49, "Previous input source", "切换到上一个输入法", "切換到上一個輸入法"],
  [3, 49, "Next input source", "切换到下一个输入法", "切換到下一個輸入法"],
  [1, 126, "Open Mission Control", "打开调度中心", "打開指揮中心"],
  [1, 125, "Show windows of the front app", "显示前台应用的所有窗口", "顯示最前方 App 的所有視窗"],
  [10, 2, "Show or hide the Dock", "显示或隐藏程序坞", "顯示或隱藏 Dock"]
]);

addReferences("macos", "macOS", "apple-screenshots", "capture", [
  [13, 20, "Copy full screenshot to clipboard", "将全屏截图复制到剪贴板", "將全螢幕截圖複製到剪貼簿"],
  [13, 21, "Screenshot", "截图", "截圖"]
]);

addReferences("macos", "Text editors", "apple", "editing", [
  [8, 11, "Toggle bold", "切换粗体", "切換粗體"],
  [8, 34, "Toggle italics", "切换斜体", "切換斜體"],
  [8, 32, "Toggle underline", "切换下划线", "切換底線"],
  [8, 40, "Insert a link", "插入链接", "插入連結"],
  [8, 17, "Show or hide fonts", "显示或隐藏字体窗口", "顯示或隱藏字體視窗"],
  [12, 1, "Save as or duplicate the document", "另存为或创建文稿副本", "另存新檔或製作文件副本"],
  [10, 3, "Focus the search field", "移到搜索栏", "移到搜尋欄"],
  [12, 33, "Align left", "左对齐", "靠左對齊"],
  [12, 30, "Align right", "右对齐", "靠右對齊"],
  [12, 44, "Open the Help menu", "打开帮助菜单", "打開輔助說明選單"],
  [10, 8, "Copy formatting", "拷贝样式", "拷貝樣式"],
  [10, 9, "Paste formatting", "粘贴样式", "貼上樣式"],
  [14, 9, "Paste and match surrounding style", "粘贴并匹配周围文本样式", "貼上並符合周圍文字樣式"],
  [12, 126, "Select to document start", "选至文稿开头", "選取至文件開頭"],
  [12, 125, "Select to document end", "选至文稿末尾", "選取至文件結尾"],
  [12, 123, "Select to line start", "选至行首", "選取至行首"],
  [12, 124, "Select to line end", "选至行尾", "選取至行尾"],
  [2, 123, "Move to the previous word boundary", "移到上一词的开头", "移到上一個字詞的開頭"],
  [2, 124, "Move to the next word boundary", "移到下一词的末尾", "移到下一個字詞的結尾"],
  [6, 123, "Extend selection left by word", "向左按词扩选", "向左以字詞為單位延伸選取範圍"],
  [6, 124, "Extend selection right by word", "向右按词扩选", "向右以字詞為單位延伸選取範圍"],
  [2, 51, "Delete the word to the left", "删除左侧的词", "刪除左側的字詞"],
  [1, 0, "Move to line or paragraph start", "移到行首或段首", "移到行首或段首"],
  [1, 14, "Move to line or paragraph end", "移到行尾或段尾", "移到行尾或段尾"],
  [1, 40, "Cut selection or text to paragraph end", "剪切选区或剪至段尾", "剪下選取文字或剪至段落結尾"],
  [1, 2, "Delete the next character", "删除后一个字符", "刪除後一個字元"],
  [1, 4, "Delete the previous character", "删除前一个字符", "刪除前一個字元"],
  [1, 3, "Move forward one character", "向前移动一个字符", "向前移動一個字元"],
  [1, 11, "Move back one character", "向后移动一个字符", "向後移動一個字元"],
  [1, 35, "Move up one line", "上移一行", "上移一行"],
  [1, 45, "Move down one line", "下移一行", "下移一行"],
  [1, 31, "Insert a line after the caret", "在插入点后换行", "在插入點後換行"],
  [1, 17, "Transpose adjacent characters", "交换插入点两侧的字符", "交換插入點兩側的字元"],
  [9, 2, "Look up selected word", "查询所选词的释义", "查詢所選字詞的定義"],
  [8, 24, "Increase selection size", "放大所选项目", "放大所選項目"],
  [12, 24, "Increase selection size", "放大所选项目", "放大所選項目"],
  [12, 27, "Decrease selection size", "缩小所选项目", "縮小所選項目"]
]);

addReferences("macos", "Safari", "apple-safari", "system", [
  [8, 27, "Zoom out", "缩小网页", "縮小網頁"],
  [8, 29, "Reset page zoom", "恢复网页默认大小", "還原網頁預設大小"],
  [12, 24, "Zoom in", "放大网页", "放大網頁"]
]);

addReferences("macos", "Finder", "apple", "files", [
  [8, 15, "Reveal original of selected alias", "显示所选替身的原始文件", "顯示所選替身的原始檔案"],
  [8, 17, "Toggle tab bar with one tab open", "单个标签页时切换标签栏", "只有一個標籤頁時切換標籤列"],
  [8, 14, "Eject selected volume", "推出所选磁盘", "退出所選磁碟"],
  [8, 38, "Show view options", "显示查看选项", "顯示顯示方式選項"],
  [8, 16, "Preview selected files", "快速查看所选文件", "快速查看所選檔案"],
  [8, 44, "Show or hide status bar", "显示或隐藏状态栏", "顯示或隱藏狀態列"],
  [8, 18, "Icon view", "图标视图", "圖像顯示方式"],
  [8, 19, "List view", "列表视图", "列表顯示方式"],
  [8, 20, "Column view", "分栏视图", "直欄顯示方式"],
  [8, 21, "Gallery view", "画廊视图", "圖庫顯示方式"],
  [8, 33, "Go to previous folder", "返回上一个文件夹", "返回上一個檔案夾"],
  [8, 30, "Go to next folder", "前往下一个文件夹", "前往下一個檔案夾"],
  [12, 2, "Open Desktop folder", "打开桌面文件夹", "打開桌面檔案夾"],
  [12, 31, "Open Documents folder", "打开文稿文件夹", "打開文件檔案夾"],
  [12, 3, "Open Recents", "打开最近使用", "打開最近項目"],
  [12, 15, "Open AirDrop", "打开隔空投送", "打開 AirDrop"],
  [12, 32, "Open Utilities", "打开实用工具", "打開工具程式"],
  [12, 34, "Open iCloud Drive", "打开 iCloud 云盘", "打開 iCloud 雲碟"],
  [12, 40, "Open Network", "打开网络窗口", "打開網路視窗"],
  [10, 37, "Open Downloads", "打开下载文件夹", "打開下載項目"],
  [12, 51, "Empty Trash", "清倒废纸篓", "清空垃圾桶"],
  [10, 1, "Show or hide sidebar", "显示或隐藏边栏", "顯示或隱藏側邊欄"],
  [10, 35, "Show or hide path bar", "显示或隐藏路径栏", "顯示或隱藏路徑列"],
  [9, 0, "Create an alias", "制作替身", "製作替身"]
]);

addReferences("windows", "Windows", "windows", "system", [
  [8, 46, "Minimize all windows", "最小化所有窗口", "將所有視窗最小化"],
  [8, 115, "Toggle other windows minimized", "最小化或还原其他窗口", "最小化或還原其他視窗"],
  [12, 123, "Move window to left monitor", "将窗口移到左侧显示器", "將視窗移至左側顯示器"],
  [12, 124, "Move window to right monitor", "将窗口移到右侧显示器", "將視窗移至右側顯示器"],
  [9, 2, "Create a virtual desktop", "新建虚拟桌面", "新增虛擬桌面"],
  [9, 118, "Close current virtual desktop", "关闭当前虚拟桌面", "關閉目前的虛擬桌面"],
  [9, 123, "Switch to desktop on the left", "切到左侧虚拟桌面", "切換到左側虛擬桌面"],
  [9, 124, "Switch to desktop on the right", "切到右侧虚拟桌面", "切換到右側虛擬桌面"],
  [8, 0, "Open action center or quick settings", "打开操作中心或快速设置", "開啟控制中心或快速設定"],
  [8, 45, "Open notifications and calendar", "打开通知中心和日历", "開啟通知中心和行事曆"],
  [8, 4, "Start voice typing", "启动语音输入", "啟動語音輸入"],
  [8, 7, "Open Quick Link menu", "打开快速链接菜单", "開啟快速連結功能表"],
  [8, 35, "Choose display projection mode", "选择投影模式", "選擇投影模式"],
  [8, 40, "Connect to a wireless display", "连接无线显示器", "連線到無線顯示器"],
  [8, 5, "Open Game Bar", "打开游戏栏", "開啟遊戲列"],
  [12, 24, "Zoom in with Magnifier", "用放大镜放大", "使用放大鏡放大"],
  [8, 27, "Zoom out with Magnifier", "用放大镜缩小", "使用放大鏡縮小"],
  [8, 53, "Close Magnifier", "关闭放大镜", "關閉放大鏡"]
]);

addReferences("windows", "Game Bar", "ms-game", "capture", [
  [10, 15, "Start or stop recording", "开始或停止录制", "開始或停止錄製"]
]);

addReferences("windows", "File Explorer", "windows", "files", [
  [5, 45, "Create a folder", "新建文件夹", "新增資料夾"],
  [2, 36, "Show item properties", "显示所选项目的属性", "顯示所選項目的內容"],
  [2, 123, "Go to previous folder", "返回上一个文件夹", "返回上一個資料夾"],
  [2, 124, "Go to next folder", "前往下一个文件夹", "前往下一個資料夾"],
  [2, 126, "Open parent folder", "打开上级文件夹", "開啟上層資料夾"],
  [1, 48, "Next tab", "下一个标签页", "下一個索引標籤"],
  [5, 48, "Previous tab", "上一个标签页", "上一個索引標籤"]
]);

addReferences("windows", "Dialogs", "windows", "system", [
  [1, 48, "Next tab in dialog", "切到对话框的下一个选项卡", "切換到對話方塊的下一個索引標籤"],
  [5, 48, "Previous tab in dialog", "切到对话框的上一个选项卡", "切換到對話方塊的上一個索引標籤"]
]);

addReferences("windows", "Text editors", "windows", "editing", [
  [5, 123, "Extend selection one word left", "向左按词扩选", "向左以字詞為單位延伸選取範圍"],
  [5, 124, "Extend selection one word right", "向右按词扩选", "向右以字詞為單位延伸選取範圍"]
]);

addReferences("windows", "Microsoft Edge", "ms-edge", "files", [
  [1, 1, "Save this page", "保存当前网页", "儲存目前網頁"],
  [1, 45, "New browser window", "新建浏览器窗口", "新增瀏覽器視窗"],
  [1, 31, "Open a local file", "打开本地文件", "開啟本機檔案"],
  [1, 35, "Print this page", "打印当前网页", "列印目前網頁"],
  [1, 13, "Close this tab", "关闭当前标签页", "關閉目前索引標籤"],
  [1, 17, "Open a new tab", "打开新标签页", "開啟新索引標籤"],
  [5, 24, "Zoom in", "放大网页", "放大網頁"],
  [1, 27, "Zoom out", "缩小网页", "縮小網頁"],
  [1, 29, "Reset page zoom", "重置网页缩放", "重設網頁縮放"]
]);

addReferences("linux", "GNOME", "gnome-settings", "system", [
  [8, 4, "Hide window", "隐藏窗口", "隱藏視窗"],
  [8, 126, "Maximize window", "最大化窗口", "最大化視窗"],
  [8, 125, "Restore window", "还原窗口", "還原視窗"],
  [8, 123, "Tile window to the left", "将窗口平铺到左侧", "將視窗並排於左側"],
  [8, 124, "Tile window to the right", "将窗口平铺到右侧", "將視窗並排於右側"],
  [8, 49, "Next input source", "下一个输入法", "下一個輸入法"],
  [12, 49, "Previous input source", "上一个输入法", "上一個輸入法"],
  [8, 45, "Focus active notification", "聚焦当前通知", "移至目前通知"],
  [8, 1, "Open system menu", "打开系统菜单", "開啟系統選單"],
  [8, 115, "Go to first workspace", "切换到第一个工作区", "切換至第一個工作區"],
  [8, 119, "Go to last workspace", "切换到最后一个工作区", "切換至最後一個工作區"],
  [12, 126, "Move window to monitor above", "将窗口移到上方显示器", "將視窗移至上方顯示器"],
  [12, 125, "Move window to monitor below", "将窗口移到下方显示器", "將視窗移至下方顯示器"]
]);

addReferences("linux", "GNOME", "gnome-settings", "capture", [
  [4, 65847, "Save full-screen screenshot", "保存全屏截图", "儲存全螢幕截圖"],
  [2, 65847, "Save window screenshot", "保存窗口截图", "儲存視窗截圖"]
]);

addReferences("windows", "Taskbar", "windows", "system", [
  [8, 29, "Open or switch to the taskbar app assigned to 0", "打开或切到任务栏中数字 0 对应的应用", "開啟或切換到工作列中數字 0 對應的 App"],
  [8, 18, "Open or switch to pinned app 1", "打开或切到任务栏第 1 个固定应用", "開啟或切換到工作列第 1 個釘選 App"],
  [8, 19, "Open or switch to pinned app 2", "打开或切到任务栏第 2 个固定应用", "開啟或切換到工作列第 2 個釘選 App"],
  [8, 20, "Open or switch to pinned app 3", "打开或切到任务栏第 3 个固定应用", "開啟或切換到工作列第 3 個釘選 App"],
  [8, 21, "Open or switch to pinned app 4", "打开或切到任务栏第 4 个固定应用", "開啟或切換到工作列第 4 個釘選 App"],
  [8, 23, "Open or switch to pinned app 5", "打开或切到任务栏第 5 个固定应用", "開啟或切換到工作列第 5 個釘選 App"],
  [8, 22, "Open or switch to pinned app 6", "打开或切到任务栏第 6 个固定应用", "開啟或切換到工作列第 6 個釘選 App"],
  [8, 26, "Open or switch to pinned app 7", "打开或切到任务栏第 7 个固定应用", "開啟或切換到工作列第 7 個釘選 App"],
  [8, 28, "Open or switch to pinned app 8", "打开或切到任务栏第 8 个固定应用", "開啟或切換到工作列第 8 個釘選 App"],
  [8, 25, "Open or switch to pinned app 9", "打开或切到任务栏第 9 个固定应用", "開啟或切換到工作列第 9 個釘選 App"]
]);

function chordParts(row){
 const out=[],mac=row.platform==='macos';
 if(row.modifiers&1)out.push('Ctrl');if(row.modifiers&2)out.push(mac?'Option':'Alt');
 if(row.modifiers&4)out.push('Shift');if(row.modifiers&8)out.push(mac?'⌘':row.platform==='linux'?'Super':'Win');
 out.push(keyLabels[row.key]||('Key '+row.key));return out;
}
const referenceIndex=new Map();
const chordId=row=>row.platform+':'+row.modifiers+':'+row.key;
for(const entry of entries){const id=chordId(entry),related=referenceIndex.get(id)||[];related.push(entry);referenceIndex.set(id,related);}
// Look up annotations only. Counts and displayed keycaps always use the recorded chord.
const keypadAliases = new Map([[76, 36], [78, 27], [81, 24], [69, 24]]);
function matches(row) {
  const exact = referenceIndex.get(chordId(row));
  if (exact) return exact;
  const key = keypadAliases.get(row.key);
  if (key === undefined) return [];
  const lookup = modifiers => referenceIndex.get(chordId({platform: row.platform, modifiers, key})) || [];
  const plain = lookup(row.modifiers);
  if (row.key !== 69 || (row.modifiers & 4)) return plain;
  // Both = and Shift+= can mean zoom in. Show each distinct documented use once.
  const shifted = lookup(row.modifiers | 4);
  return plain.concat(shifted.filter(b => !plain.some(a =>
    a.scope === b.scope && a.source === b.source && a.title.en === b.title.en)));
}
const scopeNames = {
  "en": ["Apps", "Text editors", "Supported apps", "Terminal", "File Explorer", "Dialogs", "Taskbar", "Game Bar", "Microsoft Edge", "Safari"],
  "zh-CN": ["应用通用", "文本编辑器", "支持此操作的应用", "终端", "文件资源管理器", "对话框", "任务栏", "Xbox 游戏栏", "Microsoft Edge", "Safari"],
  "zh-TW": ["應用程式通用", "文字編輯器", "支援此操作的應用程式", "終端機", "檔案總管", "對話方塊", "工作列", "Xbox 遊戲列", "Microsoft Edge", "Safari"],
  "ja": ["アプリ共通", "テキストエディタ", "対応アプリ", "端末", "エクスプローラー", "ダイアログ", "タスク バー", "Xbox Game Bar", "Microsoft Edge", "Safari"],
  "ko": ["앱 공통", "텍스트 편집기", "지원하는 앱", "터미널", "파일 탐색기", "대화 상자", "작업 표시줄", "Xbox Game Bar", "Microsoft Edge", "Safari"],
  "de": ["Apps", "Texteditoren", "Unterstützte Apps", "Terminal", "Datei-Explorer", "Dialogfelder", "Taskleiste", "Xbox Game Bar", "Microsoft Edge", "Safari"],
  "fr": ["Applications", "Éditeurs de texte", "Applications compatibles", "Terminal", "Explorateur de fichiers", "Boîtes de dialogue", "Barre des tâches", "Xbox Game Bar", "Microsoft Edge", "Safari"],
  "es": ["Aplicaciones", "Editores de texto", "Aplicaciones compatibles", "Terminal", "Explorador de archivos", "Cuadros de diálogo", "Barra de tareas", "Xbox Game Bar", "Microsoft Edge", "Safari"]
};
function scope(entry,locale="en"){const index=scopeNames.en.indexOf(entry.scope);return scopeNames[locale]?.[index]||entry.scope;}
function title(entry,locale){return entry.title[locale==='zh-TW'?'zh-TW':String(locale).startsWith('zh')?'zh':'en'];}
function searchTokens(value){
 return String(value).normalize('NFKC').toLowerCase()
  .replace(/[⌘]/g,' meta ').replace(/[⌥]/g,' alt ').replace(/[⇧]/g,' shift ').replace(/[⌃]/g,' ctrl ')
  .replace(/←/g,' left ').replace(/→/g,' right ').replace(/↑/g,' up ').replace(/↓/g,' down ')
  .replace(/\bwindows(?=\s*\+)/g,'meta').replace(/\b(command|cmd|win|super)\b/g,'meta').replace(/\b(control|strg)\b/g,'ctrl')
  .replace(/\b(maj|umschalt|mayús)\b/g,'shift').replace(/screenshots?|screen capture|截屏|截圖|截图/g,'capture')
  .replace(/\+\s*\+|^\s*\+\s*$/g,' plus ')
  .replace(/\boption\b/g,'alt').replace(/\breturn\b/g,'enter').replace(/\bescape\b/g,'esc')
  .replace(/\b(page[ -]?up|pgup)\b/g,'pageup').replace(/\b(page[ -]?down|pgdn|pgdown)\b/g,'pagedown')
  .replace(/\b(print[ -]?screen|prtscn|prtscr|prt[ -]sc)\b/g,'prtsc')
  .replace(/\bdel\b/g,'delete').replace(/\bins\b/g,'insert')
  .split(/[\s+]+/).filter(Boolean);
}
// Retain only the current query, never a growing history of entered searches.
let lastQuery='',queryTokens=[];
const keySearchTokens=new Map(Object.entries(keyLabels).map(([key,label])=>[Number(key),searchTokens(label)]));
function searchWords(entry){return searchTokens([...Object.values(entry.title),entry.scope,entry.category,...Object.keys(scopeNames).map(locale=>scope(entry,locale))].join(' ')).join(' ');}
const entryWords=new Map(entries.map(entry=>[entry,searchWords(entry)]));
function matchesQuery(row,related,query){
 if(query!==lastQuery){lastQuery=query;queryTokens=searchTokens(query);}
 if(!queryTokens.length)return true;
 const keyTokens=[...(keySearchTokens.get(row.key)||searchTokens('Key '+row.key))];
 for(const [flag,name]of [[1,'ctrl'],[2,'alt'],[4,'shift'],[8,'meta']])if(row.modifiers&flag)keyTokens.push(name);
 const words=[row.platform==='linux'?'linux ubuntu gnome':row.platform==='macos'?'mac macos':'windows',...related.map(e=>entryWords.get(e)||searchWords(e))].join(' ');
 return queryTokens.every(token=>{
  // A key or modifier is exact: searching C must not match Ctrl or Capture.
  if(token==='plus')return row.key===69||(row.key===24&&!!(row.modifiers&4));
  const keyLike=/^[a-z0-9]$|^f\d{1,2}$/.test(token)||['ctrl','alt','shift','meta','enter','esc','tab','space','backspace','delete','insert','left','right','up','down','home','end','pageup','pagedown','prtsc'].includes(token);
  return keyLike?keyTokens.includes(token)||(queryTokens.length>1&&words.split(' ').includes(token)):words.includes(token)||keyTokens.some(k=>k.includes(token));
 });
}
function search(entry,query){return matchesQuery(entry,[entry],query);}
return {sources,keyLabels,entries,scope,chordParts,matches,title,search,matchesQuery};
})();
if(typeof module!=='undefined')module.exports=ClickPulseShortcutCatalog;
