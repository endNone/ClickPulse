'use strict';
// Curated reference data, shared by the offline App and static handbook.
const ClickPulseShortcutCatalog=(()=>{
const sources={
  "apple": {
    "name": "Apple · Mac keyboard shortcuts",
    "url": "https://support.apple.com/en-us/102650"
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

function chordParts(row){
 const out=[],mac=row.platform==='macos';
 if(row.modifiers&1)out.push('Ctrl');if(row.modifiers&2)out.push(mac?'Option':'Alt');
 if(row.modifiers&4)out.push('Shift');if(row.modifiers&8)out.push(mac?'⌘':row.platform==='linux'?'Super':'Win');
 out.push(keyLabels[row.key]||('Key '+row.key));return out;
}
const referenceIndex=new Map();
const chordId=row=>row.platform+':'+row.modifiers+':'+row.key;
for(const entry of entries){const id=chordId(entry),related=referenceIndex.get(id)||[];related.push(entry);referenceIndex.set(id,related);}
function matches(row){return referenceIndex.get(chordId(row))||[];}
const scopeNames={"zh-CN": ["应用通用", "文本编辑器", "支持此操作的应用", "终端"], "zh-TW": ["應用程式通用", "文字編輯器", "支援此操作的應用程式", "終端機"], "ja": ["アプリ共通", "テキストエディタ", "対応アプリ", "端末"], "ko": ["앱 공통", "텍스트 편집기", "지원하는 앱", "터미널"], "de": ["Apps", "Texteditoren", "Unterstützte Apps", "Terminal"], "fr": ["Applications", "Éditeurs de texte", "Applications compatibles", "Terminal"], "es": ["Aplicaciones", "Editores de texto", "Aplicaciones compatibles", "Terminal"]};
function scope(entry,locale="en"){const index=["Apps","Text editors","Supported apps","Terminal"].indexOf(entry.scope);return scopeNames[locale]?.[index]||entry.scope;}
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
