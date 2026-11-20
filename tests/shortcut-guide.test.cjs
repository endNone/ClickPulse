'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const fs=require('node:fs'),os=require('node:os'),path=require('node:path'),vm=require('node:vm');
const {build,pages,baseUrl,html,buildMarkdown}=require('../scripts/build-shortcut-guide.cjs');
const C=require('../ui/shortcuts-catalog.js'),root=path.resolve(__dirname,'..');
const unescape=value=>value.replace(/&quot;/g,'"').replace(/&#39;/g,"'").replace(/&lt;/g,'<').replace(/&gt;/g,'>').replace(/&amp;/g,'&');
test('all four Chinese pages contain readable tables, page-specific metadata and valid local assets without JavaScript',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'clickpulse-atlas-'));
 try{
  build(dir);const canonicals=new Set(),titles=new Set();
  for(const page of pages){
   const file=path.join(dir,page.slug,'index.html'),html=fs.readFileSync(file,'utf8'),tbody=html.match(/<tbody>([\s\S]*?)<\/tbody>/)[1];
   assert(html.includes('<html lang="zh-CN">'));assert(/<meta name="description" content="[^"]*[快捷键]/.test(html));
   titles.add(html.match(/<title[^>]*>(.*?)<\/title>/)[1]);const canonical=html.match(/rel="canonical" href="([^"]+)"/)[1];canonicals.add(canonical);assert.equal(canonical,'https://endnone.github.io/ClickPulse/'+page.slug);
   const ids=Array.from(tbody.matchAll(/data-entry-id="([^"]+)"/g),m=>m[1]);assert.deepEqual(ids,C.entries.filter(e=>!page.platform||e.platform===page.platform).map(e=>e.id));
   assert.equal((tbody.match(/<tr /g)||[]).length,ids.length);assert.deepEqual(
    Array.from(tbody.matchAll(/<a href="([^"]+)"/g),match=>unescape(match[1])),
    C.entries.filter(e=>!page.platform||e.platform===page.platform).map(e=>C.sources[e.source].url)
   );
   const publishers={'support.apple.com':'Apple','support.microsoft.com':'Microsoft','help.gnome.org':'GNOME'};
   for(const [,url,name] of tbody.matchAll(/<a href="([^"]+)"[^>]*>([^<]+) ↗<\/a>/g)){
    assert.equal(name,publishers[new URL(unescape(url)).hostname],url);
   }
   assert(tbody.includes('复制'));assert(html.includes('想知道自己最常用哪些快捷键？</p>'));assert(html.includes('href="https://github.com/endNone/ClickPulse/releases/latest"'));
   for(const match of html.matchAll(/(?:href|src)="([^"]+)"/g)){
    const ref=unescape(match[1]);if(/^(?:https?:|#)/.test(ref))continue;
    assert(fs.existsSync(path.resolve(path.dirname(file),ref)),page.slug+': '+ref);
   }
  }
  assert.equal(titles.size,4);assert.equal(canonicals.size,4);
  const xml=fs.readFileSync(path.join(dir,'sitemap.xml'),'utf8');assert(xml.includes('http://www.sitemaps.org/schemas/sitemap/0.9'));assert.deepEqual(Array.from(xml.matchAll(/<loc>(.*?)<\/loc>/g),m=>m[1]),[...canonicals]);
  const first=fs.readFileSync(path.join(dir,'index.html'),'utf8');build(dir);assert.equal(fs.readFileSync(path.join(dir,'index.html'),'utf8'),first);
  build(dir,{base:'https://example.org/handbook'});assert(fs.readFileSync(path.join(dir,'mac/index.html'),'utf8').includes('https://example.org/handbook/mac/'));
 } finally{fs.rmSync(dir,{recursive:true,force:true});}
});
test('guide filters existing rows and switches language without recreating its table',()=>{
 const elem=(text='',dataset={})=>({textContent:text,dataset,hidden:false,handlers:{},attrs:{},addEventListener(name,fn){this.handlers[name]=fn;},setAttribute(k,v){this.attrs[k]=v;}});
 const titles=C.entries.map(e=>elem(e.title.zh,{en:e.title.en}));const rows=C.entries.map(e=>elem('',{entryId:e.id}));const categories=['','editing','system','files','capture','terminal'].map(c=>elem('',{category:c}));
 const elements=Object.fromEntries(['count','empty','query','language'].map(id=>[id,elem()]));const controls=[elem()];const document={documentElement:{lang:'zh-CN'},getElementById:id=>elements[id],querySelectorAll:selector=>({'[data-en]':titles,'[data-entry-id]':rows,'[data-category]':categories,'[data-js]':controls}[selector])};
 vm.runInNewContext(fs.readFileSync(path.join(root,'docs/shortcuts/app.js'),'utf8'),{ClickPulseShortcutCatalog:C,document});
 assert.equal(rows.filter(r=>!r.hidden).length,C.entries.length);assert.equal(elements.count.textContent,C.entries.length+' 条参考词条');
 elements.query.handlers.input({target:{value:'Ctrl+C'}});assert.deepEqual(rows.filter(r=>!r.hidden).map(r=>r.dataset.entryId),C.entries.filter(e=>C.search(e,'Ctrl+C')).map(e=>e.id));
 elements.language.handlers.click();assert.equal(document.documentElement.lang,'en');assert.equal(titles[0].textContent,C.entries[0].title.en);assert(elements.count.textContent.includes('reference'));
 categories.find(c=>c.dataset.category==='capture').handlers.click();assert.equal(rows.filter(r=>!r.hidden).length,0);assert.equal(elements.empty.hidden,false);
 elements.query.handlers.input({target:{value:''}});assert.equal(rows.filter(r=>!r.hidden).length,C.entries.filter(e=>e.category==='capture').length);
 elements.language.handlers.click();assert.equal(titles[0].textContent,C.entries[0].title.zh);
});
test('build rejects unsuitable canonical URLs and Pages actions are pinned to commit objects',()=>{
 for(const url of ['http://example.org','https://example.org/?token=x','https://example.org/#fragment','https://user:pass@example.org/'])assert.throws(()=>baseUrl(url));
 const workflow=fs.readFileSync(path.join(root,'.github/workflows/shortcut-guide.yml'),'utf8');assert(workflow.includes('workflow_dispatch:'));assert(!/\n\s+(push|pull_request):/.test(workflow));
 for(const action of ['configure-pages','upload-pages-artifact','deploy-pages'])assert(new RegExp('actions/'+action+'@[a-f0-9]{40}\\b').test(workflow));
});

test('checked-in handbook pages and sitemap match the deterministic builder',()=>{
 const base='https://endnone.github.io/ClickPulse/';
 for(const page of pages)assert.equal(fs.readFileSync(path.join(root,'docs/shortcuts',page.slug,'index.html'),'utf8'),html(page,base,true));
 assert.equal(fs.readFileSync(path.join(root,'docs/shortcuts.md'),'utf8'),buildMarkdown());
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'clickpulse-handbook-drift-'));
 try{build(dir);assert.equal(fs.readFileSync(path.join(root,'docs/shortcuts/sitemap.xml'),'utf8'),fs.readFileSync(path.join(dir,'sitemap.xml'),'utf8'));}finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('the atlas has unique identities, documented sources and recordable physical chords',()=>{
 const hosts=new Set(['support.apple.com','support.microsoft.com','help.gnome.org']);
 const ids=new Set(),scopes=new Set();
 const native=fs.readFileSync(path.join(root,'src/desktop/share.rs'),'utf8');
 for(const [id,source] of Object.entries(C.sources)){
  const url=new URL(source.url);
  assert.equal(url.protocol,'https:');assert(hosts.has(url.hostname),source.url);
  assert(native.includes('"'+id+'"')&&native.includes('"'+source.url+'"'),id+' missing native reference');
 }
 for(const entry of C.entries){
  const {platform,modifiers,key,scope}=entry;
  assert(['macos','windows','linux'].includes(platform));
  assert(!ids.has(entry.id),entry.id);ids.add(entry.id);
  const identity=[platform,modifiers,key,scope].join(':');
  assert(!scopes.has(identity),identity);scopes.add(identity);
  assert(C.sources[entry.source],entry.id);
  for(const lang of ['en','zh','zh-TW'])assert(entry.title[lang]?.trim(),entry.id);
  // Mirror Chord::new in src/shortcuts.rs, using its canonical, stored key codes.
  const pcSpecial=platform!=='macos'&&[65847,65606,66117].includes(key);
  const nonText=pcSpecial||[36,48,51,53,64,79,80,90,103,109,111].includes(key)
   ||(key>=71&&key<=74)||(key>=96&&key<=101)||(key>=105&&key<=107)||(key>=113&&key<=126);
  assert(Number.isInteger(modifiers)&&modifiers>=1&&modifiers<=15,identity);
  assert(Number.isInteger(key)&&((key>=0&&key<=126)||pcSpecial)&&!(key>=54&&key<=63),identity);
  assert((modifiers&9)||nonText||((modifiers&2)&&platform!=='macos'),identity);
  assert(![72,73,74].includes(key),'media key '+identity);
 }
});
test('numeric keypad aliases annotate the physical chord and never drop modifiers',()=>{
 const refs=(platform,modifiers,key)=>C.matches({platform,modifiers,key});
 assert.deepEqual(refs('windows',2,76),refs('windows',2,36));
 assert(refs('windows',2,76).some(e=>e.scope==='File Explorer'));
 assert.deepEqual(refs('macos',8,76),refs('macos',8,36));
 assert.deepEqual(refs('macos',8,36),[]);
 assert.deepEqual(refs('macos',8,78),refs('macos',8,27));
 assert.deepEqual(refs('macos',8,81),refs('macos',8,24));
 assert.deepEqual(refs('windows',1,69),refs('windows',5,24));
 assert.deepEqual(refs('macos',8,69).map(e=>[e.scope,e.title.en]),[
  ['Text editors','Increase selection size'],['Safari','Zoom in']
 ]);
 assert.deepEqual(refs('macos',12,69),refs('macos',12,24));
 assert.deepEqual(refs('macos',13,76),[]);
 assert.deepEqual(C.chordParts({platform:'windows',modifiers:2,key:76}),['Alt','Num Enter']);
 assert.deepEqual(refs('macos',8,83),[]); // Num 1 is not an alias for Finder's Command-1.
});
test('unknown shortcut and suggestion labels are available in all eight app languages',()=>{
 const locales=require('../ui/locales.json');
 assert.equal(Object.keys(locales).length,8);
 for(const [lang,messages] of Object.entries(locales)){
  for (const key of ['unknown','suggest'])assert(messages['shortcuts.'+key]?.trim(),lang+': '+key);
 }
 assert.equal(locales.en['shortcuts.unknown'],'Not listed · may be app-specific');
 assert.equal(locales['zh-CN']['shortcuts.unknown'],'未收录 · 可能由应用自定义');
 assert.equal(locales['zh-TW']['shortcuts.unknown'],'未收錄 · 可能由應用程式自訂');
});

test('the native suggestion form uses the same physical key labels as the preview',()=>{
 const native=fs.readFileSync(path.join(root,'src/desktop/shortcut_suggestion.rs'),'utf8');
 const names=native.match(/const KEY_NAMES: \[&str; 127\] = (\[[\s\S]*?\]);/)[1];
 const physical=JSON.parse(names.replace(/,\s*\]$/,']'));
 assert.deepEqual(physical,Array.from({length:127},(_,key)=>C.keyLabels[key]||''));
 const form=fs.readFileSync(path.join(root,'.github/ISSUE_TEMPLATE/shortcut-suggestion.yml'),'utf8');
 assert.deepEqual(Array.from(form.matchAll(/^    id: (\w+)$/gm),match=>match[1]),
  ['platform','chord','purpose','source']);
});
