'use strict';
// Entries are already in the HTML. JavaScript translates, filters, renders the
// interactive keyboard preview/filter, and builds the cross-OS comparison view.
(()=>{
 const C=ClickPulseShortcutCatalog,$=id=>document.getElementById(id);
 const qAll=selector=>Array.from(document.querySelectorAll(selector)||[]);
 const win=typeof window!=='undefined'?window:null;
 let locale='zh-CN',category='',query='',viewMode='table';
 let selectedMods=0,selectedKey=null,capturing=false,toastTimer=null;

 const pagePlatform=document.body?.dataset?.pagePlatform||'';
 let kbOs=pagePlatform||'macos';

 const translations=qAll('[data-en]').map(element=>({element,zh:element.textContent,en:element.dataset.en}));
 const byId=new Map(C.entries.map(entry=>[entry.id,entry]));
 const rows=qAll('[data-entry-id]').map(element=>({element,entry:byId.get(element.dataset.entryId)}));
 const categories=qAll('[data-category]');
 const osButtons=qAll('[data-kb-os]');
 const viewButtons=qAll('[data-view]');

 // Modifier bitmasks matching ui/shortcuts-catalog.js: Ctrl=1, Alt=2, Shift=4, Meta=8
 const MOD_CTRL=1,MOD_ALT=2,MOD_SHIFT=4,MOD_META=8;

 // Concept normalization for Cross-OS Comparison View
 const CONCEPT_ALIASES={
  'copy':'copy','paste':'paste','cut':'cut','undo':'undo','redo':'redo',
  'select all':'select_all','save':'save','find':'find','print':'print',
  'next match':'find_next','previous match':'find_prev',
  'open a new tab':'new_tab','new tab':'new_tab','close tab':'close_tab','close current tab':'close_tab',
  'reopen closed tab':'reopen_tab','reopen last closed tab':'reopen_tab',
  'new window':'new_window','new document or window':'new_window',
  'close window':'close_window','quit app':'quit_app','close active item or exit app':'quit_app',
  'minimize':'minimize','minimize window':'minimize','toggle fullscreen':'fullscreen','full screen':'fullscreen',
  'switch app':'switch_app','switch between open apps':'switch_app','switch windows':'switch_app',
  'switch app window':'switch_same_app','switch windows of an app':'switch_same_app',
  'lock screen':'lock_screen','lock pc':'lock_screen','lock the screen':'lock_screen',
  'spotlight':'launcher','open start':'launcher','activities overview':'launcher',
  'force quit an app':'force_quit','open task manager':'force_quit',
  'screen capture':'screenshot_full','take a screenshot':'screenshot_full','save full screenshot':'screenshot_full',
  'area capture':'screenshot_area','take a screenshot of part of your screen':'screenshot_area','interactive screenshot':'screenshot_area',
  'capture controls':'screenshot_tool','snipping tool':'screenshot_tool',
  'refresh or reload':'refresh','reload page':'refresh','refresh active window':'refresh',
  'new folder':'new_folder','move to trash':'delete_item','delete selected item':'delete_item',
  'rename selected item':'rename_item','item information':'properties','display properties':'properties',
  'open settings':'settings','app settings':'settings',
  'show or hide the dock':'desktop_show','display and hide the desktop':'desktop_show','hide all normal windows':'desktop_show',
  'character viewer':'emoji','open emoji panel':'emoji',
  'line start':'line_start','line end':'line_end','document start':'doc_start','document end':'doc_end',
  'open terminal':'terminal','new terminal tab':'terminal_tab','copy in terminal':'terminal_copy','paste in terminal':'terminal_paste'
 };

 const CONCEPT_LABELS={
  copy:['复制','Copy'],paste:['粘贴','Paste'],cut:['剪切','Cut'],undo:['撤销','Undo'],redo:['重做','Redo'],
  select_all:['全选','Select all'],save:['保存','Save'],find:['查找','Find'],print:['打印','Print'],
  find_next:['下一个匹配','Next match'],find_prev:['上一个匹配','Previous match'],
  new_tab:['新建标签页','New tab'],close_tab:['关闭标签页','Close tab'],reopen_tab:['恢复关闭的标签页','Reopen closed tab'],
  new_window:['新建窗口','New window'],close_window:['关闭窗口','Close window'],quit_app:['退出应用','Quit app'],
  minimize:['最小化窗口','Minimize window'],fullscreen:['切换全屏','Toggle fullscreen'],
  switch_app:['切换应用','Switch app'],switch_same_app:['切换同应用窗口','Switch app window'],
  lock_screen:['锁定屏幕','Lock screen'],launcher:['系统搜索 / 启动器','System launcher / Search'],
  force_quit:['强制退出 / 任务管理器','Force quit / Task Manager'],
  screenshot_full:['全屏截图','Full screenshot'],screenshot_area:['区域截图','Area screenshot'],screenshot_tool:['截图工具','Screenshot tool'],
  refresh:['刷新 / 重新载入','Refresh / Reload'],new_folder:['新建文件夹','New folder'],
  delete_item:['移到废纸篓 / 删除','Move to Trash / Delete'],rename_item:['重命名','Rename'],
  properties:['查看属性 / 简介','Properties / Info'],settings:['打开设置','Open settings'],
  desktop_show:['显示桌面 / 隐藏程序坞','Show desktop / Dock'],emoji:['表情与符号面板','Emoji & symbols'],
  line_start:['跳到行首','Line start'],line_end:['跳到行尾','Line end'],
  doc_start:['文稿开头','Document start'],doc_end:['文稿末尾','Document end'],
  terminal:['打开终端','Open terminal'],terminal_tab:['终端新建标签','New terminal tab'],
  terminal_copy:['终端复制','Copy in terminal'],terminal_paste:['终端粘贴','Paste in terminal']
 };

 // ANSI physical keyboard rows using ClickPulse macOS virtual keycodes
 const KB_ROWS=[
  [
   {code:53,label:'Esc',u:1.15,fn:true},{spacer:true,u:.45,fn:true},
   {code:122,label:'F1',fn:true},{code:120,label:'F2',fn:true},{code:99,label:'F3',fn:true},{code:118,label:'F4',fn:true},{spacer:true,u:.3,fn:true},
   {code:96,label:'F5',fn:true},{code:97,label:'F6',fn:true},{code:98,label:'F7',fn:true},{code:100,label:'F8',fn:true},{spacer:true,u:.3,fn:true},
   {code:101,label:'F9',fn:true},{code:109,label:'F10',fn:true},{code:103,label:'F11',fn:true},{code:111,label:'F12',fn:true},{spacer:true,u:.3,fn:true},
   {code:117,label:'Del',u:1.1,fn:true}
  ],
  [
   {code:50,label:'`'},{code:18,label:'1'},{code:19,label:'2'},{code:20,label:'3'},{code:21,label:'4'},{code:23,label:'5'},
   {code:22,label:'6'},{code:26,label:'7'},{code:28,label:'8'},{code:25,label:'9'},{code:29,label:'0'},
   {code:27,label:'-'},{code:24,label:'='},{code:51,label:'⌫',u:1.9}
  ],
  [
   {code:48,label:'Tab',u:1.45},{code:12,label:'Q'},{code:13,label:'W'},{code:14,label:'E'},{code:15,label:'R'},{code:17,label:'T'},
   {code:16,label:'Y'},{code:32,label:'U'},{code:34,label:'I'},{code:31,label:'O'},{code:35,label:'P'},
   {code:33,label:'['},{code:30,label:']'},{code:42,label:'\\',u:1.45}
  ],
  [
   {label:'Caps',u:1.75,disabled:true},{code:0,label:'A'},{code:1,label:'S'},{code:2,label:'D'},{code:3,label:'F'},{code:5,label:'G'},
   {code:4,label:'H'},{code:38,label:'J'},{code:40,label:'K'},{code:37,label:'L'},{code:41,label:';'},
   {code:39,label:"'"},{code:36,label:'Enter',u:2.15}
  ],
  [
   {mod:MOD_SHIFT,cls:'mod-shift',label:'Shift',u:2.25},{code:6,label:'Z'},{code:7,label:'X'},{code:8,label:'C'},{code:9,label:'V'},
   {code:11,label:'B'},{code:45,label:'N'},{code:46,label:'M'},{code:43,label:','},{code:47,label:'.'},
   {code:44,label:'/'},{mod:MOD_SHIFT,cls:'mod-shift',label:'Shift',u:2.65}
  ],
  'bottom'
 ];

 // Browser KeyboardEvent.code -> ClickPulse keycode
 const EVENT_TO_CODE={
  KeyA:0,KeyS:1,KeyD:2,KeyF:3,KeyH:4,KeyG:5,KeyZ:6,KeyX:7,KeyC:8,KeyV:9,KeyB:11,KeyQ:12,KeyW:13,KeyE:14,KeyR:15,
  KeyY:16,KeyT:17,Digit1:18,Digit2:19,Digit3:20,Digit4:21,Digit6:22,Digit5:23,Equal:24,Digit9:25,Digit7:26,
  Minus:27,Digit8:28,Digit0:29,BracketRight:30,KeyO:31,KeyU:32,BracketLeft:33,KeyI:34,KeyP:35,Enter:36,KeyL:37,
  KeyJ:38,Quote:39,KeyK:40,Semicolon:41,Backslash:42,Comma:43,Slash:44,KeyN:45,KeyM:46,Period:47,Tab:48,Space:49,
  Backquote:50,Backspace:51,Escape:53,F5:96,F6:97,F7:98,F3:99,F8:100,F9:101,F11:103,F10:109,F12:111,
  Home:115,PageUp:116,Delete:117,F4:118,End:119,F2:120,PageDown:121,F1:122,
  ArrowLeft:123,ArrowRight:124,ArrowDown:125,ArrowUp:126
 };

 const modClass=key=>key==='⌘'||key==='Win'||key==='Super'?'mod-meta':key==='Ctrl'?'mod-ctrl':key==='Option'||key==='Alt'?'mod-alt':key==='Shift'?'mod-shift':'';
 const osName=p=>p==='macos'?'macOS':p==='windows'?'Windows':'Ubuntu';

 function showToast(msg){
  const el=$('atlas-toast');if(!el||!win)return;
  el.textContent=msg;el.hidden=false;
  win.clearTimeout(toastTimer);
  toastTimer=win.setTimeout(()=>{el.hidden=true;},1800);
 }

 function copyText(text,msg){
  if(!win)return;
  if(win.navigator?.clipboard?.writeText){
   win.navigator.clipboard.writeText(text).then(()=>showToast(msg)).catch(()=>{});
  }
 }

 // Theme support
 const themeBtn=$('theme-toggle');
 function currentTheme(){
  const attr=document.documentElement.dataset?.theme;
  if(attr==='dark'||attr==='light')return attr;
  return win?.matchMedia?.('(prefers-color-scheme: dark)').matches?'dark':'light';
 }
 function syncThemeButton(){
  if(!themeBtn)return;
  const dark=currentTheme()==='dark';
  const zh=dark?'浅色':'深色',en=dark?'Light':'Dark';
  themeBtn.textContent=locale==='zh-CN'?zh:en;
  themeBtn.dataset.en=en;
  const item=translations.find(t=>t.element===themeBtn);
  if(item){item.zh=zh;item.en=en;}
 }
 if(themeBtn){
  try{
   const saved=win?.localStorage?.getItem('clickpulse-atlas-theme');
   if(saved==='dark'||saved==='light')document.documentElement.dataset.theme=saved;
  }catch{}
  syncThemeButton();
  themeBtn.addEventListener('click',()=>{
   const next=currentTheme()==='dark'?'light':'dark';
   document.documentElement.dataset.theme=next;
   try{win?.localStorage?.setItem('clickpulse-atlas-theme',next);}catch{}
   syncThemeButton();
  });
 }

 // Interactive Keyboard rendering & state
 const kbRoot=$('atlas-keyboard'),kbHud=$('kb-hud'),kbCapture=$('kb-capture'),kbClear=$('kb-clear');
 let kbKeyButtons=[];

 function bottomRowFor(os){
  if(os==='macos'){
   return [
    {mod:MOD_CTRL,cls:'mod-ctrl',label:'Ctrl',u:1.3},{mod:MOD_ALT,cls:'mod-alt',label:'Option',u:1.3},
    {mod:MOD_META,cls:'mod-meta',label:'⌘',u:1.55},{code:49,label:'Space',u:5.8},
    {mod:MOD_META,cls:'mod-meta',label:'⌘',u:1.55},{mod:MOD_ALT,cls:'mod-alt',label:'Option',u:1.3},
    'arrows'
   ];
  }
  const meta=os==='windows'?'Win':'Super';
  return [
   {mod:MOD_CTRL,cls:'mod-ctrl',label:'Ctrl',u:1.35},{mod:MOD_META,cls:'mod-meta',label:meta,u:1.3},
   {mod:MOD_ALT,cls:'mod-alt',label:'Alt',u:1.3},{code:49,label:'Space',u:5.9},
   {mod:MOD_ALT,cls:'mod-alt',label:'Alt',u:1.3},{mod:MOD_CTRL,cls:'mod-ctrl',label:'Ctrl',u:1.35},
   'arrows'
  ];
 }

 function makeKeyButton(spec){
  if(spec.spacer){
   const s=document.createElement('span');s.className='kb-key is-spacer'+(spec.fn?' is-fn':'');
   if(spec.u)s.style.setProperty('--u',spec.u);return s;
  }
  const btn=document.createElement('button');btn.type='button';
  btn.className='kb-key'+(spec.fn?' is-fn':'')+(spec.mod?' is-mod '+spec.cls:'');
  btn.textContent=spec.label;
  if(spec.u)btn.style.setProperty('--u',spec.u);
  if(spec.disabled){btn.disabled=true;return btn;}
  if(spec.mod)btn.dataset.mod=String(spec.mod);
  if(spec.code!==undefined)btn.dataset.code=String(spec.code);
  btn.addEventListener('click',()=>{
   if(spec.mod){selectedMods^=spec.mod;}
   else if(spec.code!==undefined){selectedKey=selectedKey===spec.code?null:spec.code;}
   syncKeySelection();
   filter();
  });
  kbKeyButtons.push(btn);
  return btn;
 }

 function renderKeyboard(){
  if(!kbRoot)return;
  kbKeyButtons=[];
  const grid=document.createElement('div');grid.className='kb-grid';
  for(const rowSpec of KB_ROWS){
   const row=document.createElement('div');row.className='kb-row';
   const specs=rowSpec==='bottom'?bottomRowFor(kbOs):rowSpec;
   for(const spec of specs){
    if(spec==='arrows'){
     const cluster=document.createElement('div');cluster.className='kb-arrows';
     const blanks=[{spacer:true},{code:126,label:'↑'},{spacer:true},{code:123,label:'←'},{code:125,label:'↓'},{code:124,label:'→'}];
     for(const b of blanks)cluster.append(makeKeyButton(b));
     row.append(cluster);
    }else row.append(makeKeyButton(spec));
   }
   grid.append(row);
  }
  kbRoot.replaceChildren(grid);
  for(const b of osButtons)b.setAttribute('aria-pressed',String(b.dataset.kbOs===kbOs));
  syncKeySelection();
 }

 function syncKeySelection(){
  for(const btn of kbKeyButtons){
   const mod=Number(btn.dataset.mod||0),code=btn.dataset.code!==undefined?Number(btn.dataset.code):null;
   const selected=(mod&&(selectedMods&mod)!==0)||(code!==null&&selectedKey===code);
   btn.classList.toggle('is-selected',Boolean(selected));
  }
  if(kbClear)kbClear.hidden=!selectedMods&&selectedKey===null;
  if(!selectedMods&&selectedKey===null)resetHud();
  else updateHudSelection();
 }

 function highlightEntry(entry){
  if(!kbRoot)return;
  if(entry&&!pagePlatform&&entry.platform!==kbOs){kbOs=entry.platform;renderKeyboard();}
  for(const btn of kbKeyButtons){
   if(!entry){btn.classList.remove('is-active');continue;}
   const mod=Number(btn.dataset.mod||0),code=btn.dataset.code!==undefined?Number(btn.dataset.code):null;
   const hit=(mod&&(entry.modifiers&mod)!==0)||(code!==null&&entry.key===code);
   btn.classList.toggle('is-active',Boolean(hit));
  }
  if(!kbHud)return;
  if(!entry){syncKeySelection();return;}
  const frag=document.createDocumentFragment();
  const osTag=document.createElement('span');osTag.className='entry-system system-'+entry.platform;osTag.textContent=osName(entry.platform);
  const chord=document.createElement('span');chord.className='chord';
  for(const part of C.chordParts(entry)){const k=document.createElement('kbd');const cls=modClass(part);if(cls)k.className=cls;k.textContent=part;chord.append(k);}
  const title=document.createElement('span');title.className='kb-hud-title';title.textContent=locale==='zh-CN'?entry.title.zh:entry.title.en;
  const scope=document.createElement('span');scope.className='kb-hud-meta';scope.textContent=C.scope(entry,locale);
  frag.append(osTag,chord,title,scope);
  kbHud.replaceChildren(frag);
 }

 function resetHud(){
  if(!kbHud)return;
  const span=document.createElement('span');span.className='kb-hint';
  span.textContent=capturing
   ?(locale==='zh-CN'?'按下组合键即可筛选，按 Esc 退出':'Live key capture active: press any shortcut to filter (Esc to exit)')
   :(locale==='zh-CN'?'悬停快捷键预览键位，或点击键帽按按键反查':'Hover any shortcut to preview keys, or click keycaps to filter');
  kbHud.replaceChildren(span);
 }

 function updateHudSelection(){
  if(!kbHud)return;
  const parts=selectedKey!==null?C.chordParts({platform:kbOs,modifiers:selectedMods,key:selectedKey}):C.chordParts({platform:kbOs,modifiers:selectedMods,key:0}).slice(0,-1);
  const frag=document.createDocumentFragment();
  const label=document.createElement('span');label.className='kb-hud-meta';label.textContent=locale==='zh-CN'?'按按键筛选：':'Key filter:';
  const chord=document.createElement('span');chord.className='chord';
  for(const part of parts){const k=document.createElement('kbd');const cls=modClass(part);if(cls)k.className=cls;k.textContent=part;chord.append(k);}
  frag.append(label,chord);
  kbHud.replaceChildren(frag);
 }

 for(const btn of osButtons){
  btn.addEventListener('click',()=>{kbOs=btn.dataset.kbOs;renderKeyboard();if(selectedMods||selectedKey!==null)updateHudSelection();});
 }
 if(kbClear){
  kbClear.addEventListener('click',()=>{selectedMods=0;selectedKey=null;syncKeySelection();filter();});
 }
 if(kbCapture&&win){
  kbCapture.addEventListener('click',()=>{
   capturing=!capturing;
   kbCapture.setAttribute('aria-pressed',String(capturing));
   if(!selectedMods&&selectedKey===null)resetHud();
  });
  win.addEventListener('keydown',event=>{
   if(!capturing)return;
   if(event.target?.tagName==='INPUT')return;
   if(event.key==='Escape'&&!event.ctrlKey&&!event.altKey&&!event.metaKey&&!event.shiftKey){
    capturing=false;kbCapture.setAttribute('aria-pressed','false');resetHud();return;
   }
   event.preventDefault();
   const mods=(event.ctrlKey?MOD_CTRL:0)|(event.altKey?MOD_ALT:0)|(event.shiftKey?MOD_SHIFT:0)|(event.metaKey?MOD_META:0);
   const mapped=EVENT_TO_CODE[event.code];
   if(mapped!==undefined){selectedMods=mods;selectedKey=mapped;}
   else if(mods){selectedMods=mods;selectedKey=null;}
   syncKeySelection();
   filter();
  });
 }

 // Cross-OS Comparison View (All Systems page only)
 const compareRoot=$('compare-view'),tableRoot=$('table-view');
 function matchesEntry(entry){
  if(!entry)return false;
  if(category&&entry.category!==category)return false;
  if(!C.search(entry,query))return false;
  if(selectedMods&&(entry.modifiers&selectedMods)!==selectedMods)return false;
  if(selectedKey!==null&&entry.key!==selectedKey)return false;
  return true;
 }

 function conceptKey(entry){
  const norm=entry.title.en.toLowerCase().trim();
  return CONCEPT_ALIASES[norm]||norm;
 }

 function renderCompare(visibleEntries){
  if(!compareRoot)return;
  const groups=new Map();
  for(const entry of visibleEntries){
   const key=conceptKey(entry);
   if(!groups.has(key))groups.set(key,{key,macos:[],windows:[],linux:[],zh:entry.title.zh,en:entry.title.en,category:entry.category});
   groups.get(key)[entry.platform].push(entry);
  }
  // Prioritize concepts that exist on at least 2 operating systems
  const list=Array.from(groups.values())
   .filter(g=>(g.macos.length>0)+(g.windows.length>0)+(g.linux.length>0)>=2)
   .sort((a,b)=>((b.macos.length>0)+(b.windows.length>0)+(b.linux.length>0))-((a.macos.length>0)+(a.windows.length>0)+(a.linux.length>0)));

  const frag=document.createDocumentFragment();
  if(list.length){
   const header=document.createElement('div');header.className='compare-header';
   for(const text of [locale==='zh-CN'?'操作意图':'Action','macOS','Windows','Ubuntu / GNOME']){
    const s=document.createElement('span');s.textContent=text;header.append(s);
   }
   frag.append(header);
  }
  for(const group of list){
   const row=document.createElement('div');row.className='compare-row';
   const action=document.createElement('div');action.className='compare-action';
   const title=document.createElement('strong');
   const label=CONCEPT_LABELS[group.key];
   title.textContent=label?(locale==='zh-CN'?label[0]:label[1]):(locale==='zh-CN'?group.zh:group.en);
   const sub=document.createElement('small');
   sub.textContent=label?(locale==='zh-CN'?label[1]:label[0]):(locale==='zh-CN'?group.en:group.zh);
   action.append(title,sub);row.append(action);

   for(const p of ['macos','windows','linux']){
    const cell=document.createElement('div');cell.className='compare-cell'+(group[p].length?' has-entry':'');
    const osTag=document.createElement('span');osTag.className='compare-os-label';osTag.textContent=osName(p);
    cell.append(osTag);
    if(group[p].length){
     const primary=group[p][0];
     const chord=document.createElement('span');chord.className='chord';
     for(const part of C.chordParts(primary)){const k=document.createElement('kbd');const cls=modClass(part);if(cls)k.className=cls;k.textContent=part;chord.append(k);}
     cell.append(chord);
     cell.addEventListener('mouseenter',()=>highlightEntry(primary));
     cell.addEventListener('mouseleave',()=>highlightEntry(null));
     cell.addEventListener('click',()=>{
      highlightEntry(primary);
      const text=C.chordParts(primary).join(' + ');
      copyText(text,(locale==='zh-CN'?'已复制：':'Copied: ')+text);
     });
    }else{
     const dash=document.createElement('span');dash.className='compare-empty';dash.textContent='—';cell.append(dash);
    }
    row.append(cell);
   }
   frag.append(row);
  }
  compareRoot.replaceChildren(frag);
  return list.length;
 }

 function filter(){
  let count=0;const visibleEntries=[];
  for(const {element,entry}of rows){
   const visible=matchesEntry(entry);
   element.hidden=!visible;
   if(visible){count++;visibleEntries.push(entry);}
  }
  const showingCompare=viewMode==='compare'&&compareRoot;
  if(tableRoot)tableRoot.hidden=Boolean(showingCompare)||count===0;
  if(compareRoot)compareRoot.hidden=!showingCompare;
  if(showingCompare){
   const groups=renderCompare(visibleEntries);
   $('count').textContent=locale==='zh-CN'?groups+' 组跨系统对照（'+count+' 条词条）':groups+' cross-OS pairs ('+count+' entries)';
   $('empty').hidden=groups!==0;
  }else{
   $('count').textContent=locale==='zh-CN'?count+' 条参考词条':count+' reference '+(count===1?'entry':'entries');
   $('empty').hidden=count!==0;
  }
  for(const button of categories)button.setAttribute('aria-pressed',String(button.dataset.category===category));
  for(const button of viewButtons)button.setAttribute('aria-pressed',String(button.dataset.view===viewMode));
 }

 // Row hover/focus keyboard preview & click-to-copy link
 for(const {element,entry}of rows){
  if(!entry)continue;
  element.addEventListener('mouseenter',()=>highlightEntry(entry));
  element.addEventListener('mouseleave',()=>highlightEntry(null));
  element.addEventListener('focus',()=>highlightEntry(entry));
  element.addEventListener('blur',()=>highlightEntry(null));
  element.addEventListener('click',event=>{
   if(event.target?.tagName==='A')return;
   highlightEntry(entry);
   if(win?.location){
    const url=win.location.href.split('#')[0]+'#'+entry.id;
    copyText(url,(locale==='zh-CN'?'已复制词条链接：':'Copied link: ')+C.chordParts(entry).join(' + '));
   }
  });
 }

 for(const button of categories)button.addEventListener('click',()=>{category=button.dataset.category;filter();});
 for(const button of viewButtons)button.addEventListener('click',()=>{viewMode=button.dataset.view;filter();});
 $('query').addEventListener('input',event=>{query=event.target.value;filter();});
 $('language').addEventListener('click',()=>{
  locale=locale==='zh-CN'?'en':'zh-CN';document.documentElement.lang=locale;
  for(const {element,zh,en}of translations)element.textContent=locale==='zh-CN'?zh:en;
  $('query').placeholder=locale==='zh-CN'?'复制、截屏、Ctrl…':'Copy, screenshot, Ctrl…';
  syncThemeButton();
  if(!selectedMods&&selectedKey===null)resetHud();
  else updateHudSelection();
  filter();
 });

 renderKeyboard();
 filter();
 for(const element of qAll('[data-js]'))element.hidden=false;

 // Highlight URL hash target on load
 if(win?.location?.hash){
  const targetId=win.location.hash.slice(1);
  const match=rows.find(r=>r.entry?.id===targetId);
  if(match){
   match.element.classList.add('is-target');
   match.element.scrollIntoView?.({block:'center'});
   highlightEntry(match.entry);
  }
 }
})();
