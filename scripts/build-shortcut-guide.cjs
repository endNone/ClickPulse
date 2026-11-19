#!/usr/bin/env node
'use strict';
const fs=require('node:fs'),path=require('node:path');
const root=path.resolve(__dirname,'..');
const C=require('../ui/shortcuts-catalog.js');
const pages=[
 {slug:'',platform:'',name:'全部系统',en:'All systems',title:'快捷键大全（Mac、Windows、Ubuntu，附官方来源）· ClickPulse',heading:'快捷键百科',headingEn:'Shortcut Atlas',description:'查阅 Mac、Windows 和 Ubuntu 常用快捷键，按用途搜索中文说明、适用场景与官方来源。ClickPulse 免费开源，在本地离线统计快捷键。'},
 {slug:'mac/',platform:'macos',name:'Mac',en:'Mac',title:'Mac 快捷键大全（附官方来源）· ClickPulse',heading:'Mac 快捷键大全',headingEn:'Mac keyboard shortcuts',description:'Mac 快捷键中文大全：复制粘贴、窗口切换、Finder 与截图组合键，附 Apple 官方来源和适用场景。'},
 {slug:'windows/',platform:'windows',name:'Windows',en:'Windows',title:'Windows 快捷键大全（附官方来源）· ClickPulse',heading:'Windows 快捷键大全',headingEn:'Windows keyboard shortcuts',description:'Windows 快捷键中文大全：文字编辑、窗口管理、文件操作与截图组合键，附 Microsoft 官方来源和适用场景。'},
 {slug:'ubuntu/',platform:'linux',name:'Ubuntu',en:'Ubuntu',title:'Ubuntu 快捷键大全（GNOME，附官方来源）· ClickPulse',heading:'Ubuntu 快捷键大全',headingEn:'Ubuntu keyboard shortcuts',description:'Ubuntu GNOME 快捷键中文大全：桌面导航、窗口管理、截图与终端组合键，附 GNOME 官方来源。'}
];
const purposes=[['','全部用途','All purposes'],['editing','文字编辑','Editing'],['system','系统与窗口','System & windows'],['files','文件管理','Files'],['capture','屏幕捕获','Screen capture'],['terminal','终端','Terminal']];
const esc=value=>String(value).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const bilingual=(tag,zh,en,attrs='')=>`<${tag} data-en="${esc(en)}"${attrs?` ${attrs}`:''}>${esc(zh)}</${tag}>`;
function baseUrl(value='https://endnone.github.io/ClickPulse/'){
 const url=new URL(value);if(url.protocol!=='https:'||url.search||url.hash||url.username||url.password)throw Error('Expected a public HTTPS site base URL without query or fragment');
 url.pathname=url.pathname.replace(/\/?$/,'/');return url.href;
}
function html(page,base,sourceTree=false){
 const up=page.slug?'../':'',asset=name=>up+name;
 const catalog=sourceTree?(page.slug?'../../../':'../../')+'ui/shortcuts-catalog.js':asset('shortcuts-catalog.js');
 const brand=sourceTree?(page.slug?'../../../':'../../')+'assets/clickpulse-mark-blue-transparent-v1-128.png':asset('brand-mark.png');
 const entries=C.entries.filter(e=>!page.platform||e.platform===page.platform);
 const rows=entries.map(e=>`<tr data-entry-id="${esc(e.id)}"><th scope="row"><span class="chord">${C.chordParts(e).map(key=>`<kbd>${esc(key)}</kbd>`).join(' ')}</span></th><td>${bilingual('strong',e.title.zh,e.title.en)}</td><td><span class="entry-system">${e.platform==='macos'?'macOS':e.platform==='windows'?'Windows':'Ubuntu / GNOME'}</span>${bilingual('span',C.scope(e,'zh-CN'),e.scope,'class="scope"')}</td><td><a href="${esc(C.sources[e.source].url)}" target="_blank" rel="noopener noreferrer">${esc(e.source==='apple'?'Apple':e.source==='windows'?'Microsoft':'GNOME')} ↗</a></td></tr>`).join('\n');
 return `<!doctype html>
<html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="description" content="${esc(page.description)}"><link rel="canonical" href="${esc(new URL(page.slug,base).href)}">
<meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self'; connect-src 'none'; base-uri 'none'; form-action 'none'">
${bilingual('title',page.title,page.headingEn+' · ClickPulse')}<link rel="stylesheet" href="${asset('style.css')}"><script defer src="${catalog}"></script><script defer src="${asset('app.js')}"></script></head>
<body><header><a class="brand" href="${up||'./'}"><img class="brand-mark" src="${brand}" alt="">ClickPulse ${bilingual('span','/ 快捷键百科','/ Shortcut Atlas')}</a>${bilingual('button','English','简体中文','id="language" type="button" data-js hidden')}</header>
<main><section class="intro"><div>${bilingual('p','常用组合键 · 用途 · 官方来源','EVERYDAY SHORTCUTS · CONTEXT · OFFICIAL SOURCES','class="eyebrow"')}${bilingual('h1',page.heading,page.headingEn)}${bilingual('p',page.description,'Useful combinations, clear context and official sources. Find the shortcuts for your next step.','class="intro-description"')}
<div class="download-callout">${bilingual('p','想知道自己最常用哪些快捷键？ClickPulse 免费开源、离线统计','Discover your most-used shortcuts. ClickPulse is free, open source and counts activity offline.')}${bilingual('a','免费下载 ClickPulse ↗','Download ClickPulse ↗','class="download-button" href="https://github.com/endNone/ClickPulse/releases/latest"')}</div></div><div class="intro-keys" aria-hidden="true"><kbd>⌘</kbd><kbd>Ctrl</kbd><kbd>Super</kbd>${bilingual('span','少些寻找，多些顺手。','Less reaching. More doing.')}</div></section>
<div class="filter-bar"><nav class="systems" aria-label="Operating system">${pages.map(p=>bilingual('a',p.name,p.en,`href="${up+p.slug||'./'}"${p===page?' aria-current="page"':''}`)).join('')}</nav><label class="search" data-js hidden>${bilingual('span','搜索','Search')}<input id="query" type="search" autocomplete="off" spellcheck="false" placeholder="复制、截屏、Ctrl…"></label></div>
<div class="directory"><aside>${bilingual('h2','按用途浏览','Browse by purpose')}<div id="categories" data-js hidden>${purposes.map(([key,zh,en])=>bilingual('button',zh,en,`type="button" data-category="${key}" aria-pressed="${key===''}"`)).join('')}</div>${bilingual('p','这里收录默认用法。应用、键盘布局和自定义改键都可能改变组合键的作用。','Apps, keyboard layouts and custom mappings can change what a combination does.','class="reference-note"')}${bilingual('a','ClickPulse 如何统计 ↗','How ClickPulse counts ↗','href="#how-it-works"')}</aside>
<section class="entry-section" aria-label="Shortcuts"><div class="results-heading"><p id="count" role="status" aria-live="polite">${entries.length} 条参考词条</p>${bilingual('span','来源核对 · 2026 年 11 月','Sources checked · November 2026')}</div><div class="table-scroll"><table id="entries"><thead><tr>${[['组合键','Shortcut'],['用途','Use'],['系统 / 场景','System / Context'],['来源','Source']].map(([zh,en])=>bilingual('th',zh,en,'scope="col"')).join('')}</tr></thead><tbody>${rows}</tbody></table></div>${bilingual('p','没有匹配的词条','No matching entries','id="empty" class="empty" hidden')}<noscript>下表可直接阅读；启用 JavaScript 后可搜索、筛选及切换英文。</noscript></section></div>
<section id="how-it-works" class="explanation">${bilingual('p','关于快捷键统计','ABOUT YOUR STATISTICS','class="eyebrow"')}${bilingual('h2','识别组合键，不猜测你的操作。','A combination is an observation. An action is context.')}<div class="explainer-grid"><div>${bilingual('h3','统计什么','What gets counted')}${bilingual('p','ClickPulse 累计带修饰键的组合次数，排除普通打字。只保存按日累计，不保存时间顺序、应用名称或剪贴板。','ClickPulse counts modifier combinations, excluding ordinary typing. Only daily totals are saved, without a time sequence, app names or clipboard contents.')}</div><div>${bilingual('h3','命中意味着什么','What a match means')}${bilingual('p','注释将组合键关联到常见的文档用法，不代表哪个应用接收了它或操作成功。','Annotations link combinations to documented uses. They do not identify the receiving app or prove an action succeeded.')}</div><div>${bilingual('h3','默认用法的边界','Where defaults differ')}${bilingual('p','参考以美式键位为基础。改键、输入法和桌面设置可能影响结果。Ubuntu 词条针对 GNOME。','The reference assumes US key positions. Remapping, input methods and desktop settings can change behavior. Ubuntu entries target GNOME.')}</div></div></section>
<section class="sources">${bilingual('h2','查看原始来源','Follow the sources')}<div>${Object.values(C.sources).map(s=>`<a href="${esc(s.url)}" target="_blank" rel="noopener noreferrer">${esc(s.name)} ↗</a>`).join('')}</div></section></main><footer><a href="https://github.com/endNone/ClickPulse">ClickPulse on GitHub ↗</a>${bilingual('span','无需账号，无遥测。网页不会读取你的活动记录。','No account. No telemetry. This guide does not read your activity.')}</footer></body></html>
`;
}
function build(output,{base='https://endnone.github.io/ClickPulse/',writeSource=false}={}){
 base=baseUrl(base);output=path.resolve(output);fs.mkdirSync(output,{recursive:true});
 for(const name of ['style.css','app.js'])fs.copyFileSync(path.join(root,'docs/shortcuts',name),path.join(output,name));
 fs.copyFileSync(path.join(root,'ui/shortcuts-catalog.js'),path.join(output,'shortcuts-catalog.js'));
 fs.copyFileSync(path.join(root,'assets/clickpulse-mark-blue-transparent-v1-128.png'),path.join(output,'brand-mark.png'));
 for(const page of pages){const dir=path.join(output,page.slug);fs.mkdirSync(dir,{recursive:true});fs.writeFileSync(path.join(dir,'index.html'),html(page,base));if(writeSource){const source=path.join(root,'docs/shortcuts',page.slug);fs.mkdirSync(source,{recursive:true});fs.writeFileSync(path.join(source,'index.html'),html(page,base,true));}}
 const sitemap='<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'+pages.map(p=>`  <url><loc>${esc(new URL(p.slug,base).href)}</loc></url>`).join('\n')+'\n</urlset>\n';
 fs.writeFileSync(path.join(output,'sitemap.xml'),sitemap);if(writeSource)fs.writeFileSync(path.join(root,'docs/shortcuts/sitemap.xml'),sitemap);
 const markdown=buildMarkdown();fs.writeFileSync(path.join(output,'shortcuts.md'),markdown);if(writeSource)fs.writeFileSync(path.join(root,'docs/shortcuts.md'),markdown);
 return {output,pages:pages.length,entries:C.entries.length};
}

function buildMarkdown(){
const intro=`# ClickPulse 快捷键百科 / Shortcut Atlas

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

本地生成：\`node scripts/build-shortcut-guide.cjs\`，随后用任意静态服务器打开 \`dist/shortcut-guide/\`。

生成结果包含 canonical 和 sitemap.xml；可通过 SHORTCUT_GUIDE_BASE_URL 指定正式站点地址。

GitHub Pages 发布：在仓库 Settings → Pages 中选择 GitHub Actions，再手动运行 Shortcut guide 工作流。
该工作流仅手动触发，不随普通代码推送自动发布。部署后的地址由工作流的 environment 输出给出。

## 维护词条

词条位于 \`ui/shortcuts-catalog.js\`：平台、修饰键、键位、场景、用途、来源。
新增词条应核对官方说明；用途保持简短，避免将应用专属操作当作系统通用行为。
运行 \`node --test tests/insights.test.cjs\` 与本生成脚本可检查词条并同步文档。

## 系统索引
`;
let markdown=intro;
for(const [platform,name]of [['macos','macOS'],['windows','Windows'],['linux','Ubuntu / GNOME']]){
 markdown+='\n### '+name+'\n\n| 组合键 | 用途 / Use | 场景 | 来源 |\n| --- | --- | --- | --- |\n';
 for(const e of C.entries.filter(e=>e.platform===platform))markdown+='| '+C.chordParts(e).map(k=>k.includes('`')?'`` '+k+' ``':'`'+k+'`').join(' + ')+' | '+e.title.zh+' / '+e.title.en+' | '+e.scope+' | ['+C.sources[e.source].name+']('+C.sources[e.source].url+') |\n';
}
return markdown;
}
module.exports={build,html,baseUrl,pages,buildMarkdown};
if(require.main===module){const result=build(process.argv[2]||path.join(root,'dist/shortcut-guide'),{base:process.env.SHORTCUT_GUIDE_BASE_URL||undefined,writeSource:!process.env.SHORTCUT_GUIDE_BASE_URL});console.log(`Shortcut guide: ${result.entries} entries, ${result.pages} pages → ${result.output}`);}
