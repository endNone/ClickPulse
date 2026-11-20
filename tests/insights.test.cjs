'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const C=require('../ui/shortcuts-catalog.js'),M=require('../ui/insights.js');
const createI18n=require('../ui/i18n.js'),messages=require('../ui/locales.json');
const Recipes=require('../ui/report-recipes.js'),Facts=require('../ui/report-facts.js');
function viewHarness({lateBrand=false,fonts,platform='macos',preview=false,timeZone='UTC',systemTimeZone='UTC',realLocale=false}={}){
 const elements=new Map(),texts=[],sent=[],toasts=[];let draws=0,exports=0;
 const ctx=new Proxy({measureText:text=>({width:text.length*10}),fillRect:()=>{draws++;},fillText:(text,x,y,max)=>texts.push({text,x,y,max,font:ctx.font,align:ctx.textAlign,color:ctx.fillStyle})}, {get:(target,key)=>target[key]||(()=>{})});
 function element(){return {children:[],options:[{}],dataset:{},style:{},handlers:{},attributes:{},replacements:0,width:1080,height:1350,
  addEventListener(type,handler){this.handlers[type]=handler;},setAttribute(key,value){this.attributes[key]=value;},getContext:()=>ctx,toDataURL:()=>{exports++;return 'data:image/png;base64,test';},
  focus(){document.activeElement=this;},select(){this.selectionStart=0;this.selectionEnd=this.value.length;},
  append(...children){this.children.push(...children);},replaceChildren(...children){this.children=children;this.replacements++;},
  fire(type,value){if(value!==undefined)this.value=value;return this.handlers[type]?.({target:this});}};}
 const get=id=>{if(!elements.has(id))elements.set(id,element());return elements.get(id);};
 const periods=['daily','weekly'].map(period=>{const el=element();el.dataset.reportPeriod=period;return el;});
 const brand=element();brand.complete=false;brand.naturalWidth=0;
 const document={fonts,getElementById:get,createElement:element,createElementNS:element,querySelector:selector=>selector==='.brand-icon img'&&lateBrand?brand:null,querySelectorAll:selector=>selector==='[data-report-period]'?periods:[],documentElement:{dataset:{theme:'blue'}}};
 global.ClickPulseShortcutCatalog=C;
 const window={captureSessionState(){},clickpulsePlatform:platform,clickpulsePreview:preview};
 const I=realLocale?createI18n(messages,'en',platform):{locale:'en',timezone:timeZone,effectiveTimeZone:timeZone,t:(key,vars)=>key+(vars?.count||''),number:String,percent:(ratio,digits)=>(ratio*100).toFixed(digits)+'%',day:String,date:String};
 if(realLocale)I.setTimeZone(timeZone);
 const view=M.create({document,window,I,send:(...args)=>sent.push(args),toast:message=>toasts.push(message),getSystemTimeZone:()=>systemTimeZone});
 delete global.ClickPulseShortcutCatalog;
 return {view,get,periods,document,window,brand,I,texts,sent,toasts,setSystemTimeZone:zone=>{systemTimeZone=zone;},draws:()=>draws,exports:()=>exports};
}
function report(day='2026-11-17',ranking=[],timezone='UTC'){
 return {day,period:'daily',timezone,ranking,hours:[[day,9,10,2]]};
}
const sampleCard=()=>({report:report(),now:'2026-11-17T12:00:00Z',history_days:[{day:'2026-11-17',counts:{keyboard:10,mouse:2}},{day:'2026-11-16',counts:{keyboard:5,mouse:1}}]});
test('leaving share frees its canvas and returning redraws unchanged data',()=>{
 const a=viewHarness(),data=sampleCard();a.view.render('share',data);assert.equal(a.draws(),1);
 a.view.render('overview',data);assert.equal(a.get('share-canvas').width,0);
 a.view.render('share',data);assert.equal(a.get('share-canvas').width,1080);assert.equal(a.get('share-canvas').height,1350);assert.equal(a.draws(),2);
});
test('share restores date/style across renderer disposal without preserving search text',()=>{
 const a=viewHarness(),data=sampleCard();a.view.render('share',data);
 a.get('share-style-select').fire('change','ink');a.get('share-date').fire('change','2026-11-16');
 a.get('shortcut-search').fire('input','private query');
 const saved=a.view.sessionState();assert.equal(saved.share_style,'ink');assert.equal(saved.share_day,'2026-11-16');assert(!JSON.stringify(saved).includes('private'));
 const b=viewHarness();b.view.restoreSession(saved);b.view.render('share',data);
 assert.equal(b.get('share-date').value,'2026-11-16');assert.equal(b.get('share-style-select').value,'ink');
 b.get('share-date').fire('change','2026-11-20');assert.equal(b.get('share-date').value,'2026-11-16');
 b.get('share-date').fire('change','2026-02-30');assert.equal(b.get('share-date').value,'2026-11-16');
 const later={...data,history_days:[...data.history_days,{day:'2026-01-01',counts:{keyboard:999,mouse:1}}]};b.view.render('share',later);assert.equal(b.draws(),1);
});
test('shortcut list is bounded, searchable, and opens the external shortcut atlas',()=>{
 const a=viewHarness();const data={shortcuts:Array.from({length:250},(_,i)=>({platform:'macos',modifiers:8+(i%2),key:Math.floor(i/2),count:i+1}))};
 a.view.render('shortcuts',data);assert.equal(a.get('shortcut-list').children.length,100);
 a.get('shortcut-more').fire('click');assert.equal(a.get('shortcut-list').children.length,200);
 a.get('shortcut-atlas').fire('click');assert.deepEqual(a.sent.at(-1),['shortcut_atlas']);
 a.get('shortcut-search').fire('input','Cmd+C');assert(a.get('shortcut-list').children.length<20);
});
test('combined Heatmap refreshes mouse buttons only while that page is active',()=>{
 const a=viewHarness(),data={lifetime:{mouse:2},mouse_buttons:{left:2}};
 const left=()=>a.get('mouse-distribution').children[0].children[1].textContent;
 a.view.render('keys',data);assert.equal(left(),'2');
 const changed={lifetime:{mouse:3},mouse_buttons:{left:3}};
 a.view.render('overview',changed);assert.equal(left(),'2','hidden charts do not refresh');
 a.view.render('keys',changed);assert.equal(left(),'3');
 const drawing=a.get('mouse-diagram').children[0],replacements=a.get('mouse-distribution').replacements;
 a.view.render('keys',changed);assert.equal(a.get('mouse-distribution').replacements,replacements);assert.equal(a.get('mouse-diagram').children[0],drawing);
});
test('mouse diagram, donut and legend share a theme-aligned scale for every button',()=>{
 const a=viewHarness();
 function check(snapshot){
  a.view.render('keys',snapshot);
  const rows=M.mouseBreakdown(snapshot);
  const legend=new Map(a.get('mouse-distribution').children.map(row=>{
   const [swatch,name]=row.children[0].children;
   return [name.textContent.slice('chart.mouse_'.length),swatch.style.backgroundColor];
  }));
  function checkShape(el){
   const key=el.attributes['data-mouse-button'];
   if(key&&legend.has(key))assert.equal(el.attributes.fill,legend.get(key));
   el.children.forEach(checkShape);
  }
  checkShape(a.get('mouse-diagram'));
  const arcs=a.get('mouse-chart').children[0].children[0].children.slice(1);
  assert.deepEqual(arcs.map(arc=>arc.attributes.stroke),rows.filter(row=>row.count).map(row=>legend.get(row.key)));
  const levels={};
  for(const [key,color] of legend){
   const [r,g,b]=color.match(/\d+/g).map(Number);
   levels[key]=0.2126*r+0.7152*g+0.0722*b;
  }
  return levels;
 }
 const skewed=check({mouse_buttons:{left:773640,right:15022,middle:169}});
 assert(skewed.left<skewed.right && skewed.right<skewed.middle);
 const swapped=check({mouse_buttons:{left:15022,right:169,middle:773640}});
 assert.equal(swapped.middle,skewed.left);assert.equal(swapped.left,skewed.right);
 const legacy=check({lifetime:{mouse:1000},mouse_buttons:{other:60,side:20,right:20}});
 assert(legacy.unknown<legacy.other && legacy.other<legacy.side && legacy.side<legacy.left);
 assert.equal(legacy.side,legacy.right);
 const empty=Object.values(check({mouse_buttons:{}}));
 assert.equal(new Set(empty).size,1);assert(empty[0]>225);
});
test('mouse donut accounts for legacy unknown clicks and handles empty or single-button history',()=>{
 const a=viewHarness(),arcs=()=>a.get('mouse-chart').children[0].children[0].children.slice(1);
 a.view.render('keys',{lifetime:{mouse:100},mouse_buttons:{left:60,right:25,middle:5}});
 assert.deepEqual(arcs().map(arc=>Number(arc.attributes['stroke-dasharray'].split(' ')[0])),[60,25,5,10]);
 assert.deepEqual(arcs().map(arc=>arc.attributes['stroke-dashoffset']),[-0,-60,-85,-90]);
 const unknown=a.get('mouse-distribution').children.at(-1);
 assert.equal(unknown.children[1].textContent,'10');assert.equal(unknown.children[2].textContent,'10.0%');
 a.view.render('keys',{lifetime:{mouse:0},mouse_buttons:{}});assert.equal(arcs().length,0);
 assert.equal(a.get('mouse-chart').children[0].children[1].children[0].textContent,'—');
 assert.equal(a.get('mouse-chart').children[0].children[1].children[1].textContent,'common.empty');
 a.view.render('keys',{lifetime:{mouse:3},mouse_buttons:{right:3}});
 assert.equal(arcs().length,1);assert.equal(arcs()[0].attributes['stroke-dasharray'],'100 100');
 assert.equal(a.get('mouse-chart').children[0].children[1].children[1].textContent,'chart.mouse_right');
});
test('restoring an old library or system choice keeps activity scoped to the installed platform',()=>{
 for(const platform of ['macos','windows','linux']){
  const a=viewHarness({platform});
  a.view.restoreSession({shortcut_library:true,shortcut_system:platform==='macos'?'windows':'macos',shortcut_rows:200});
  a.view.render('shortcuts',{shortcuts:[]});
  assert.equal(a.get('shortcut-dashboard').hidden,false);
  assert.equal(a.view.sessionState().shortcut_rows,200);
  assert.equal('shortcut_library' in a.view.sessionState(),false);
  assert.equal('shortcut_system' in a.view.sessionState(),false);
 }
});
test('live share updates preserve an in-progress date edit until it is committed',()=>{
 const a=viewHarness(),data=sampleCard();a.view.render('share',data);
 const input=a.get('share-date');a.document.activeElement=input;input.value='2026-11-16';
 a.view.render('share',{...data,history_days:[{day:'2026-11-17',counts:{keyboard:11,mouse:2}},data.history_days[1]]});
 assert.equal(input.value,'2026-11-16');assert.equal(a.view.sessionState().share_day,null);
 a.document.activeElement=null;input.fire('change');
 assert.equal(a.view.sessionState().share_day,'2026-11-16');
});
test('a late logo redraws the visible share card without recreating a hidden canvas',()=>{
 const a=viewHarness({lateBrand:true}),data=sampleCard();a.view.render('share',data);
 assert.equal(a.draws(),1);a.brand.complete=true;a.brand.naturalWidth=128;a.brand.fire('load');
 assert.equal(a.draws(),2);
 a.view.render('overview',data);a.brand.fire('load');assert.equal(a.draws(),2);assert.equal(a.get('share-canvas').width,0);
});
test('a historical share card keeps its date but allows the new day after midnight',()=>{
 const a=viewHarness(),data=sampleCard();a.view.render('share',data);
 a.get('share-date').fire('change','2026-11-16');
 a.view.render('share',{...data,now:'2026-11-18T00:01:00Z'});
 assert.equal(a.get('share-date').max,'2026-11-18');
 assert.equal(a.get('share-date').value,'2026-11-16');
 a.get('share-date').fire('change','2026-11-18');
 assert.equal(a.view.sessionState().share_day,null);
});
test('live counts update shortcut numbers in place until the ranking changes',()=>{
 const a=viewHarness(),rows=[{platform:'macos',modifiers:8,key:8,count:10},{platform:'macos',modifiers:8,key:9,count:5}];
 a.view.render('shortcuts',{shortcuts:rows});const host=a.get('shortcut-list'),first=host.children[0];
 a.view.render('shortcuts',{shortcuts:[{...rows[0],count:11},rows[1]]});
 assert.equal(host.children[0],first);assert.equal(host.replacements,1);
 assert.equal(first.children[2].children[0].textContent,'11');
 a.view.render('shortcuts',{shortcuts:[rows[0],{...rows[1],count:20}]});
 assert.equal(host.children[0].children[2].children[0].textContent,'20');
});
test('imported activity from another system leaves the local empty state intact',()=>{
 const a=viewHarness({platform:'windows'});a.view.render('shortcuts',{shortcuts:[]});
 a.view.render('shortcuts',{shortcuts:[{platform:'macos',modifiers:8,key:8,count:1}]});
 assert.equal(a.get('shortcut-list').children[0].textContent,'shortcuts.empty');
 assert.equal(a.get('shortcut-ranking').children[0].children[1].textContent,'shortcuts.empty');
});
test('unrelated keyboard and hourly updates do not rebuild mouse button graphics',()=>{
 const a=viewHarness(),data={today:{mouse:2},week:{mouse:2},lifetime:{mouse:2},mouse_buttons:{left:2},hourly:[{start_ms:100,end_ms:200,counts:{mouse:2}}]};
 a.view.render('mouse',data);const drawing=a.get('mouse-diagram').children[0],distribution=a.get('mouse-distribution').children[0];
 a.view.render('mouse',{...data,today:{mouse:2,keyboard:50},hourly:[{...data.hourly[0],end_ms:300}]});
 assert.equal(a.get('mouse-diagram').children[0],drawing);
 assert.equal(a.get('mouse-distribution').children[0],distribution);
 a.view.render('mouse',{...data,mouse_buttons:{right:2}});
 assert.notEqual(a.get('mouse-diagram').children[0],drawing);
});
test('shortcut search accepts navigation key abbreviations and arrow symbols',()=>{
 const row={platform:'windows',modifiers:1,key:121};
 for(const query of ['Ctrl+PgDn','Ctrl+PgDown','Control + Page Down'])assert(C.matchesQuery(row,[],query),query);
 for(const [key,query]of [[116,'Ctrl+PgUp'],[117,'Ctrl+Del'],[123,'Ctrl+←'],[124,'Ctrl+→'],[125,'Ctrl+↓'],[126,'Ctrl+↑']])assert(C.matchesQuery({...row,key},[],query),query);
 assert(!C.matchesQuery(row,[],'Ctrl+PgUp'));
 for(const query of ['Win+Print Screen','Win+PrtScn','Win+Prt Sc'])assert(C.matchesQuery({platform:'windows',modifiers:8,key:65847},[],query),query);
});
test('shortcut searches accept typed combinations and Mac modifier symbols',()=>{
 const win=C.entries.find(e=>e.platform==='windows'&&e.modifiers===1&&e.key===8);
 const mac=C.entries.find(e=>e.platform==='macos'&&e.modifiers===12&&e.key===20);
 for(const query of ['Ctrl+C','Control + C','Ｃｔｒｌ＋Ｃ'])assert(C.search(win,query),query);
 for(const query of ['⌘⇧3','Cmd+Shift+3','Command Shift 3'])assert(C.search(mac,query),query);
 assert(!C.search(win,'Ctrl+V'));assert(C.search(win,'复制'));assert(C.search(win,'Copy'));
 assert.deepEqual(C.chordParts({platform:'macos',modifiers:8,key:105}),['⌘','F13']);
 assert.deepEqual(C.chordParts({platform:'windows',modifiers:1,key:84}),['Ctrl','Num 2']);
});
test('reference matches exact platform, modifiers and canonical key without assigning a command',()=>{
 const ids=new Set();for(const e of C.entries){assert(!ids.has(e.id));ids.add(e.id);assert(C.sources[e.source].url.startsWith('https://'));assert(C.keyLabels[e.key]);assert(e.title.en&&e.title.zh);assert(e.modifiers>0&&e.modifiers<16);}
 assert.equal(C.matches({platform:'macos',modifiers:8,key:8})[0].title.en,'Copy');
 assert.equal(C.matches({platform:'windows',modifiers:8,key:8}).length,0);
 assert.equal(C.matches({platform:'linux',modifiers:5,key:8})[0].scope,'Terminal');
 assert.equal(C.matches({platform:'macos',modifiers:4,key:8}).length,0);
 assert.deepEqual(C.chordParts({platform:'linux',modifiers:12,key:123}),['Shift','Super','Left']);
});
test('recorded Mac chords keep one count while displaying each documented use in context',()=>{
 const cases=[
  [13,21,[['Screenshot','macOS']]],
  [8,36,[]],
  [8,11,[['Toggle bold','Text editors']]],
  [8,27,[['Zoom out','Safari']]],
  [8,34,[['Item information','Finder'],['Toggle italics','Text editors']]],
  [9,21,[]]
 ];
 const shortcuts=cases.map(([modifiers,key],i)=>({platform:'macos',modifiers,key,count:10-i}));
 const a=viewHarness();a.view.render('shortcuts',{shortcuts});
 const rows=a.get('shortcut-list').children;assert.equal(rows.length,cases.length);
 for(const [i,[modifiers,key,expected]] of cases.entries()){
  assert.deepEqual(C.matches({platform:'macos',modifiers,key}).map(e=>[e.title.en,e.scope]),expected);
  const annotation=rows[i].children[1].children;
  if(expected.length){assert.deepEqual(annotation.map(e=>e.textContent),expected.flat());}
  else{assert.equal(annotation[0].textContent,'shortcuts.unknown');assert.equal(annotation[1].textContent,'shortcuts.suggest');}
  assert.equal(rows[i].children[2].children[0].textContent,String(shortcuts[i].count));
 }
 assert.equal(C.matches({platform:'macos',modifiers:12,key:21})[0].title.en,'Area capture');
 assert.equal(C.matches({platform:'windows',modifiers:13,key:21}).length,0);
 a.get('shortcut-search').fire('input','Safari');assert.equal(a.get('shortcut-list').children.length,1);
 assert.equal(a.get('shortcut-list').children[0].children[2].children[0].textContent,'7');
});
test('shortcut ranking keeps unknown chords and searches both languages',()=>{
 const rows=[{platform:'macos',modifiers:8,key:8,count:7},{platform:'macos',modifiers:9,key:7,count:9},{platform:'windows',modifiers:1,key:8,count:2}];
 const sorted=M.shortcutRows(rows,C,'','macos');assert.equal(sorted[0].count,9);assert.equal(sorted[0].entries.length,0);
 assert.equal(M.shortcutRows(rows,C,'复制','macos').length,1);
 assert.equal(M.shortcutRows(rows,C,'copy','windows')[0].count,2);
});
test('shortcut ranking includes unlisted combinations and limits the bars to eight',()=>{
 const raw=Array.from({length:10},(_,i)=>({platform:'macos',modifiers:9,key:200+i,count:10-i}));
 const rows=M.shortcutRows([...raw,{...raw[0],count:0},{...raw[0],count:-5}],C,'','macos');
 const summary=M.shortcutSummary(rows);
 assert.equal(summary.total,55);assert.equal(summary.top.length,8);
 assert.equal(summary.top[0].count,10);assert.equal(summary.top[7].count,3);
 assert.deepEqual(M.shortcutSummary([]),{total:0,top:[]});
});
test('statistics, ranking and search automatically use the installed platform',()=>{
 const data={shortcuts:[
  {platform:'macos',modifiers:8,key:8,count:30},{platform:'macos',modifiers:8,key:9,count:10},
  {platform:'windows',modifiers:1,key:8,count:60},{platform:'linux',modifiers:1,key:8,count:17}
 ]};
 for(const [platform,total,distinct,leadingShare] of [['macos',40,2,'75.0%'],['windows',60,1,'100.0%'],['linux',17,1,'100.0%']]){
  const a=viewHarness({platform});a.view.render('shortcuts',data);
  const ranking=a.get('shortcut-ranking');
  assert.equal(ranking.children.length,distinct);
  assert.equal(ranking.children[0].children[2].children[1].textContent,leadingShare);
  assert.equal(ranking.children.reduce((sum,row)=>sum+Number(row.children[2].children[0].textContent),0),total);
  const first=ranking.children[0];a.get('shortcut-search').fire('input','Cmd+V');
  const results=a.get('shortcut-list').children;
  if(platform==='macos'){assert.equal(results.length,1);assert.equal(results[0].children[2].children[0].textContent,'10');}
  else assert.equal(results[0].textContent,'shortcuts.no_results');
  assert.equal(ranking.children[0],first);
 }
});
test('empty dashboard has no invented bars or percentage',()=>{
 const a=viewHarness();a.view.render('shortcuts',{shortcuts:[]});
 assert.equal(a.get('shortcut-ranking').children[0].className,'shortcut-chart-empty');
 assert.equal(a.get('shortcut-dashboard').hidden,false);
});
test('live shortcut charts update percentages when counts change outside the top eight',()=>{
 const a=viewHarness(),rows=Array.from({length:10},(_,i)=>({platform:'macos',modifiers:8,key:i,count:100-i*10}));
 a.view.render('shortcuts',{shortcuts:rows});const first=a.get('shortcut-ranking').children[0];
 assert.equal(first.children[2].children[1].textContent,'18.2%');
 a.view.render('shortcuts',{shortcuts:[...rows.slice(0,9),{...rows[9],count:19}]});
 assert.equal(a.get('shortcut-ranking').children[0].children[2].children[1].textContent,'17.9%');
});
test('mouse detail preserves unknown historic clicks without treating them as side buttons',()=>{
 const result=M.mouseBreakdown({lifetime:{mouse:120},mouse_buttons:{left:60,right:30,middle:4,side:6,other:5}});
 assert.equal(result.find(r=>r.key==='unknown').count,15);assert.equal(result.reduce((s,r)=>s+r.count,0),120);
 assert.deepEqual(M.mouseBreakdown({lifetime:{mouse:0}}).map(r=>r.count),[0,0,0,0,0]);
});
test('share cards follow chosen timezone and actual day rows, including missing dates and DST',()=>{
 const snapshot={now:'2026-11-02T01:30:00Z',today:{keyboard:99999,mouse:99999},history_days:[{day:'2026-11-01',counts:{keyboard:25,mouse:10}},{day:'2026-10-30',counts:{keyboard:3,mouse:1}}],demo:false};
 assert.equal(M.dayKey(snapshot.now,'America/Los_Angeles'),'2026-11-01');
 snapshot.report={day:'2026-11-01',timezone:'America/Los_Angeles',period:'daily',ranking:[],hours:[['2026-11-01',1,25,10]]};
 const card=M.cardData(snapshot,'2026-11-01','America/Los_Angeles');assert.equal(card.keyboard,25);assert.equal(card.mouse,10);assert(card.partial);assert.equal(card.hours.length,24);assert.equal(card.hours[1],35);
 assert.equal(M.cardData(snapshot,'2026-10-31','UTC').keyboard,0);assert.equal(M.cardData(snapshot,'2026-02-30','UTC'),null);
 assert(!Object.hasOwn(card,'top_keys'));assert(!Object.hasOwn(card,'shortcuts'));
});

test('usage always omits Shift-only chords, even after restoring an older session',()=>{
 const a=viewHarness({platform:'windows'});
 const shortcuts=[{platform:'windows',modifiers:4,key:123,count:100},{platform:'windows',modifiers:5,key:123,count:10},{platform:'windows',modifiers:1,key:8,count:5}];
 a.view.restoreSession({shortcut_show_shift:true});
 a.view.render('shortcuts',{shortcuts});
 const ranking=a.get('shortcut-ranking').children;
 assert.equal(ranking.length,2);
 assert.equal(ranking[0].children[2].children[0].textContent,'10');
 assert.equal(ranking[0].children[2].children[1].textContent,'66.7%');
 assert.equal(a.get('shortcut-list').children.length,2);
 assert(!Object.hasOwn(a.view.sessionState(),'shortcut_show_shift'));
 a.get('shortcut-search').fire('input','Ctrl+Shift+Left');
 assert.equal(a.get('shortcut-list').children.length,1);
 assert.equal(a.get('shortcut-list').children[0].children[2].children[0].textContent,'10');
});
test('unrequested shortcut data shows loading, never a false zero or the previous ranking',()=>{
 const a=viewHarness();a.view.render('shortcuts',{});
 assert.equal(a.get('shortcut-loading').hidden,false);
 assert.equal(a.get('shortcut-dashboard').hidden,true);assert.equal(a.get('shortcut-details').hidden,true);
 a.view.render('shortcuts',{shortcuts:[]});assert.equal(a.get('shortcut-loading').hidden,true);
 a.view.render('shortcuts',{shortcuts:[{platform:'macos',modifiers:8,key:8,count:12}]});
 a.view.render('overview',{});a.view.render('shortcuts',{});
 assert.equal(a.get('shortcut-dashboard').hidden,true);assert.equal(a.get('shortcut-details').hidden,true);
});
test('a hidden Shift-only history is described as filtered, not as never recorded',()=>{
 const a=viewHarness();a.view.render('shortcuts',{shortcuts:[{platform:'macos',modifiers:4,key:48,count:20}]});
 assert.equal(a.get('shortcut-list').children[0].textContent,'shortcuts.no_results');
 assert.equal(a.get('shortcut-ranking').children[0].children[1].textContent,'shortcuts.no_results');
});

const settle=()=>new Promise(resolve=>setImmediate(resolve));
function deferred(){let resolve,reject;const promise=new Promise((ok,no)=>{resolve=ok;reject=no;});return {promise,resolve,reject};}
test('share waits for the exact font and ready before drawing, then redraws on loadingdone',async()=>{
 const load=deferred(),ready=deferred(),requests=[],events={};
 const a=viewHarness({fonts:{load:spec=>{requests.push(spec);return load.promise;},ready:ready.promise,addEventListener:(name,fn)=>events[name]=fn}});
 a.view.render('share',sampleCard());assert.equal(a.draws(),0);assert.equal(a.get('share-save').disabled,true);
 load.resolve([]);await settle();assert.equal(a.draws(),0);
 ready.resolve();await settle();assert.equal(a.draws(),1);assert.equal(a.get('share-save').disabled,false);
 assert.equal(requests[0],'750 146px "ClickPulse UI"');
 events.loadingdone();await settle();assert.equal(a.draws(),2);
 a.view.render('overview',sampleCard());events.loadingdone();await settle();assert.equal(a.draws(),2);assert.equal(a.get('share-canvas').width,0);
});
test('late font completions cannot recreate a hidden canvas or draw an obsolete date',async()=>{
 const load=deferred();const a=viewHarness({fonts:{load:()=>load.promise,ready:Promise.resolve(),addEventListener(){}}});
 a.view.render('share',sampleCard());a.get('share-date').fire('change','2026-11-16');
 a.view.render('overview',sampleCard());load.resolve([]);await settle();assert.equal(a.draws(),0);assert.equal(a.get('share-canvas').width,0);
 a.view.render('share',{...sampleCard(),report:report('2026-11-16')});await settle();assert.equal(a.draws(),1);assert(a.texts.some(t=>t.text==='2026-11-16'));
});
test('daily and weekly shortcuts use the same summary row typography and alignment as peak hours',()=>{
 for(const period of ['daily','weekly']){
  const a=viewHarness({realLocale:true}),snap=sampleCard();
  a.view.restoreSession({share_period:period});snap.report.period=period;
  snap.report.ranking=[{platform:'macos',modifiers:8,key:9,count:5}];
  a.view.render('share',snap);
  const row=label=>{
   const left=a.texts.find(t=>t.text===label);
   const right=a.texts.find(t=>t.y===left.y&&t.align==='right');
   assert(left.x+left.max+24<=right.x-right.max);
   return [left,right];
  };
  const [busy,peak]=row(a.I.t('report.peak')),[label,chord]=row(a.I.t('share.top_shortcut'));
  assert(chord.text.includes('⌘ + V')&&label.y>busy.y);
  for(const key of ['x','align','font','color']){
   assert.equal(label[key],busy[key]);assert.equal(chord[key],peak[key]);
  }
 }
});
test('card footer has the project URL with no time zone; native copy and save share the current canvas',async()=>{
 const a=viewHarness();a.view.render('share',sampleCard());
 assert(!a.texts.some(t=>t.x===72&&t.y===1282));
 const footer=a.texts.find(t=>t.x===1008&&t.y===1282);assert.equal(footer.text,'github.com/endNone/ClickPulse');assert.equal(footer.align,'right');assert(footer.font.startsWith('500 30px'));
 assert(!a.texts.some(t=>/UTC|Shanghai|Los_Angeles/.test(t.text)));
 await a.get('share-copy').fire('click');assert.equal(a.sent[0][0],'copy_share_card');assert.equal(a.sent[0][1].day,'2026-11-17');
 await a.get('share-save').fire('click');assert.equal(a.sent[1][0],'save_share_card');assert.equal(a.sent[1][1].png,a.sent[0][1].png);assert.equal(a.exports(),2);
 for(const platform of ['macos','windows','linux']){
  const b=viewHarness({platform});b.view.render('share',sampleCard());assert.equal(b.get('share-copy').hidden,platform==='linux');
  await b.get('share-copy').fire('click');assert.equal(b.sent.length,platform==='linux'?0:1);
 }
 const preview=viewHarness({preview:true});assert.equal(preview.get('share-copy').hidden,true);
 const latePreview=viewHarness();latePreview.window.clickpulsePreview=true;
 latePreview.view.render('share',sampleCard());assert.equal(latePreview.get('share-copy').hidden,true);
 await latePreview.get('share-copy').fire('click');assert.equal(latePreview.sent.length,0);assert.equal(latePreview.exports(),0);
});
test('unavailable fonts never export a stale or incomplete image',async()=>{
 const a=viewHarness({fonts:{load:()=>Promise.reject(Error('font unavailable')),ready:Promise.resolve(),addEventListener(){}}});
 a.view.render('share',sampleCard());await settle();await a.get('share-copy').fire('click');
 assert.equal(a.draws(),0);assert.equal(a.exports(),0);assert.equal(a.sent.length,0);assert(a.toasts.includes('share.failed'));
});


for(const platform of ['macos','windows','linux']){
test(`${platform}: rankings use recording-local dates; activity uses the selected time zone`,async()=>{
 const key={platform,modifiers:platform==='macos'?8:1,key:8,count:5};
 const a=viewHarness({platform,timeZone:'UTC',systemTimeZone:'Asia/Shanghai'});
 const snap={...sampleCard(),report:report('2026-11-17',[key])};a.view.render('share',snap);
 const chord=C.chordParts(key).join(' + ');
 assert(a.texts.some(t=>t.text==='share.top_shortcut_local_date'));
 assert(a.texts.some(t=>t.text.startsWith(chord+'  ·  ')));
 assert(!a.texts.some(t=>t.text==='share.top_shortcut'));assert(a.texts.some(t=>t.text==='12'));assert.equal(a.get('share-save').disabled,false);
 a.texts.length=0;
 a.I.effectiveTimeZone='Asia/Chongqing';snap.report.timezone='Asia/Shanghai';a.view.render('share',snap);
 assert(a.texts.some(t=>t.text==='share.top_shortcut'));assert(!a.texts.some(t=>t.text==='share.top_shortcut_local_date'));
 a.texts.length=0;a.setSystemTimeZone('America/Los_Angeles');a.view.render('share',snap);
 assert(!a.texts.some(t=>t.text==='share.top_shortcut'));
});
}
test('all card languages keep the English total unit and qualify the local-date winner in both periods',()=>{
 for(const period of ['daily','weekly']){
  const a=viewHarness({realLocale:true,timeZone:'Asia/Shanghai',systemTimeZone:'America/Los_Angeles'}),snap=sampleCard();
  a.view.restoreSession({share_period:period});
  const leader={platform:'macos',modifiers:8,key:9,count:7},second={...leader,key:8,count:3};
  snap.report={...snap.report,period,timezone:'Asia/Shanghai',ranking:[{...second,platform:'windows',count:88},leader,second]};
  for(const locale of Object.keys(messages)){
   a.I.setLocale(locale);a.texts.length=0;a.view.render('share',snap);
   const printed=a.texts.map(t=>t.text),label=messages[locale]['share.top_shortcut_local_date'];
   assert(printed.includes('keys & clicks'));assert(printed.includes(label));
   assert.deepEqual(printed.filter(text=>text.includes('⌘')),['⌘ + V  ·  '+a.I.t('share.shortcut_count',{count:a.I.number(7)})]);
   const description=a.get('share-canvas').attributes['aria-label'];
   assert(description.includes('keys & clicks')&&description.includes(label)&&!description.includes('America/'));
  }
  a.texts.length=0;snap.report.ranking=[];a.view.render('share',snap);
  assert(!a.texts.some(t=>t.text.includes('⌘')||t.text===a.I.t('share.top_shortcut_local_date')));
 }
});
test('weekly cards distinguish an empty shortcut history from a pending report and from all-time counts',()=>{
 const a=viewHarness({realLocale:true}),snap=sampleCard(),winner={platform:'macos',modifiers:8,key:9,count:11};
 a.view.restoreSession({share_period:'weekly'});
 snap.shortcuts=[{...winner,count:240}];
 const canvas=a.get('share-canvas');
 a.view.render('share',snap);
 assert.equal(canvas.attributes['aria-label'],a.I.t('common.loading'));
 snap.report.period='weekly';
 snap.report.ranking=[{...winner,platform:'windows'}];
 a.view.render('share',snap);
 const empty=a.I.t('share.no_shortcuts_in_period');
 assert(a.texts.some(t=>t.text===empty));assert(canvas.attributes['aria-label'].includes(empty));
 assert(!canvas.attributes['aria-label'].includes('⌘'));
 snap.report.hours=[];snap.report.ranking=[winner];a.texts.length=0;
 a.view.render('share',snap);
 assert(canvas.attributes['aria-label'].includes('⌘ + V  ·  11 times'));
 assert(!a.texts.some(t=>t.text===empty));assert.equal(a.get('share-empty').hidden,true);
});
test('stale dates or zones cannot export or retain another period’s ranking',async()=>{
 const a=viewHarness(),data=sampleCard(),top={platform:'macos',modifiers:8,key:8,count:5};
 a.view.render('share',{...data,shortcuts:[{...top,count:999}],report:report('2026-11-17',[top])});
 assert(a.texts.some(t=>t.text.includes('⌘ + C')));assert(!a.texts.some(t=>t.text.includes('999')));
 a.texts.length=0;a.get('share-date').fire('change','2026-11-16');
 assert(!a.texts.some(t=>t.text==='share.top_shortcut'));assert.equal(a.get('share-save').disabled,true);
 await a.get('share-copy').fire('click');assert.equal(a.exports(),0);
 a.view.render('share',{...data,report:report('2026-11-16',[top],'Asia/Shanghai')});assert.equal(a.get('share-save').disabled,true);
 a.view.render('share',{...data,report:report('2026-11-16',[])});assert.equal(a.get('share-save').disabled,false);
});
test('five Mini/hide/reopen cycles release the bitmap and redraw identical share data',()=>{
 const a=viewHarness(),data=sampleCard();a.view.render('share',data);
 for(let cycle=0;cycle<5;cycle++){
  a.view.setWindowState({compact:true,visible:true});
  assert.equal(a.get('share-canvas').width,0);assert.equal(a.get('share-canvas').height,0);
  a.view.render('share',data);assert.equal(a.get('share-canvas').width,0);
  a.view.setWindowState({compact:true,visible:false});
  a.view.setWindowState({compact:false,visible:false});assert.equal(a.get('share-canvas').width,0);
  a.view.setWindowState({compact:false,visible:true});
  assert.equal(a.get('share-canvas').width,1080);assert.equal(a.get('share-canvas').height,1350);
 }
 assert.equal(a.draws(),6);
});
test('Mini cancels pending font draws and late events do not recreate the bitmap',async()=>{
 const load=deferred(),events={};const a=viewHarness({lateBrand:true,fonts:{load:()=>load.promise,ready:Promise.resolve(),addEventListener:(key,fn)=>events[key]=fn}});
 a.view.render('share',sampleCard());a.view.setWindowState({compact:true,visible:true});
 load.resolve([]);events.loadingdone();a.brand.fire('load');await settle();
 assert.equal(a.draws(),0);assert.equal(a.get('share-canvas').width,0);
 a.view.setWindowState({compact:false,visible:true});await settle();assert.equal(a.draws(),1);
});
test('save and copy hand off a complete image before Mini or hiding releases the canvas',async()=>{
 for(const copy of [false,true]){
  const a=viewHarness();a.view.render('share',sampleCard());
  const pending=a.get(copy?'share-copy':'share-save').fire('click');
  a.view.setWindowState({compact:copy,visible:false});await pending;
  assert.equal(a.get('share-canvas').width,0);assert.equal(a.sent.length,1);
  assert.equal(a.sent[0][0],copy?'copy_share_card':'save_share_card');
  assert.deepEqual(a.sent[0][1],{png:'data:image/png;base64,test',day:'2026-11-17'});
 }
});

test('Traditional Chinese annotations and translated contexts share one catalog',()=>{
 const C=require('../ui/shortcuts-catalog.js');
 for(const entry of C.entries)assert.equal(typeof entry.title['zh-TW'],'string');
 const copy=C.entries.find(e=>e.title.en==='Copy'),paste=C.entries.find(e=>e.title.en==='Paste');
 assert.equal(C.title(copy,'zh-TW'),'複製');assert.equal(C.title(paste,'zh-TW'),'貼上');
 assert.equal(C.title(copy,'zh-CN'),'复制');
 assert.equal(C.scope({scope:'Text editors'},'zh-TW'),'文字編輯器');
 assert.equal(C.scope({scope:'Text editors'},'fr'),'Éditeurs de texte');
 assert.equal(C.scope({scope:'Finder'},'zh-TW'),'Finder');
});


test('catalog search indexes every translated title and context',()=>{
 for(const query of ['貼上','儲存','分頁','终端','終端機','文本编辑器','文字編輯器','テキストエディタ','터미널','Éditeurs de texte','Editores de texto']){
  const entries=C.entries.filter(e=>C.search(e,query));
  assert(entries.length>0,query);
  assert(entries.every(e=>C.matchesQuery(e,C.matches(e),query)),query+' in usage list');
 }
 for(const e of C.entries){
  for(const text of Object.values(e.title))assert(C.search(e,text),text);
 }
});


test('warm card redraw does not reload fonts or leave buttons disabled',async()=>{
 let loads=0;const a=viewHarness({fonts:{load:()=>{loads++;return Promise.resolve([]);},ready:Promise.resolve(),addEventListener(){}}});
 const data=sampleCard();a.view.render('share',data);await settle();
 data.report.hours[0][2]++;
 a.view.render('share',data);
 assert.equal(a.get('share-save').disabled,false);assert.equal(a.draws(),2);assert.equal(loads,1);
});
test('clamping a future selected day publishes follow-today to the backend',()=>{
 const a=viewHarness();let state;
 a.window.captureSessionState=()=>{state=a.view.sessionState();};
 a.view.restoreSession({share_day:'2026-11-18'});a.view.render('share',sampleCard());
 assert.equal(state.share_day,null);assert.equal(a.get('share-save').disabled,false);
});

test('catalog search supports screen capture, literal plus, platform and translated modifier aliases',()=>{
 for(const query of ['screenshot','截屏','截图','Windows','Ubuntu','Strg+C','Maj+Tab','Umschalt+Tab','Mayús+Tab'])assert(C.entries.some(e=>C.search(e,query)),query);
 // The catalog need not contain a plus shortcut; user-recorded chords may.
 const plus={platform:'windows',key:69,modifiers:1};
 assert(C.matchesQuery(plus,[],'+'));assert(C.matchesQuery(plus,[],'Ctrl++'));
 const windowsCopy=C.entries.find(e=>e.platform==='windows'&&e.title.en==='Copy');
 assert(C.search(windowsCopy,'Windows'));assert(C.search(windowsCopy,'Windows Copy'));
 assert(!C.search(windowsCopy,'Windows+C'));assert(!C.search(windowsCopy,'Ctrl++'));
 assert(C.entries.filter(e=>C.search(e,'screenshot')).every(e=>e.category==='capture'));
});

test('optional mouse buttons enter the distribution on the first click, even below 0.1 percent',()=>{
 const a=viewHarness();
 const labels=()=>a.get('mouse-distribution').children.map(row=>row.children[0].children[1].textContent);
 const data={mouse_buttons:{left:100000,right:0,middle:0,side:0}};
 a.view.render('keys',data);
 assert.deepEqual(labels(),['chart.mouse_left','chart.mouse_right']);
 const next={mouse_buttons:{...data.mouse_buttons,middle:1,side:1}};
 a.view.render('keys',next);
 assert.deepEqual(labels(),['chart.mouse_left','chart.mouse_right','chart.mouse_middle','chart.mouse_side']);
 assert.equal(a.get('mouse-distribution').children[2].children[2].textContent,'<0.1%');
});

test('a suggestion includes only the physical chord in native and preview mode',()=>{
 for(const preview of [false,true]){
  const a=viewHarness({preview});let opened;
  a.window.open=(...args)=>{opened=args;};
  const row={platform:'macos',modifiers:8,key:76,count:7281};
  a.view.render('shortcuts',{shortcuts:[row]});
  const [keys,annotation]=a.get('shortcut-list').children[0].children;
  assert.equal(keys.children.at(-1).textContent,'Num Enter');
  assert.equal(annotation.children[0].textContent,'shortcuts.unknown');
  annotation.children[1].onclick();
  if(preview){
   const url=new URL(opened[0]);assert.equal(url.origin+url.pathname,'https://github.com/endNone/ClickPulse/issues/new');
   assert.deepEqual(Object.fromEntries(url.searchParams),{template:'shortcut-suggestion.yml',platform:'macos',chord:'⌘ + Num Enter'});
   assert.deepEqual(opened.slice(1),['_blank','noopener,noreferrer']);assert.deepEqual(a.sent,[]);
  }else assert.deepEqual(a.sent,[['suggest_shortcut',{platform:'macos',modifiers:8,key:76}]]);
 }
});

const H=require('../ui/history.js');
function promptForCard(card,focus,depth=3){
 const recipe=Recipes.createPicker(()=>0).next(M.recipeContext(card));
 const eligible=Facts.available(card,H),budget=Recipes.registry.medium[recipe.medium].budget;
 const ids=focus?focus.filter(id=>eligible.includes(id)):M.selectFocus(eligible,budget,[],true,()=>0);
 return M.reportPrompt(card,recipe,ids,depth);
}
function promptFor(snapshot,period='daily',zone='UTC',system='UTC',focus){
 const card=M.cardData(snapshot,snapshot.report.day,zone,system,period);
 return promptForCard(card,focus);
}
test('report budgets fill from eligible highlights and keep shortcut depths mutually exclusive',()=>{
 const weights={shortcut:1,ranking:2,mix:1,peak:1,rhythm:2,book:1};
 const ids=Object.keys(weights);
 for(const budget of [2,3,4]){
  const seen=new Set();
  for(let i=0;i<100;i++){
   const selected=M.selectFocus(ids,budget,[],true,()=>i/100);
   const signature=selected.join('/');
   seen.add(signature);
   assert.equal(selected.reduce((sum,id)=>sum+weights[id],0),budget);
   assert(selected.length<=3 && !(selected.includes('ranking')&&selected.includes('shortcut')));
   if(budget===2)assert(!(selected.includes('peak')&&selected.includes('rhythm')));
   assert.notDeepEqual(M.selectFocus(ids,budget,selected,true,()=>0),selected);
   assert.deepEqual(M.selectFocus(ids,budget,selected,false,()=>0),selected);
  }
  assert(seen.has({2:'shortcut/book',3:'peak/rhythm',4:'ranking/mix/peak'}[budget]));
 }
 assert.deepEqual(M.selectFocus([],4),[]);
 assert.deepEqual(M.selectFocus(['mix'],4,['mix'],true),['mix']);
 assert.deepEqual(M.selectFocus(['mix','peak','rhythm'],4,['ranking','rhythm'],false),['mix','peak','rhythm']);
});

test('copy includes only the highlights selected in the prompt, even across repeated regeneration',async()=>{
 const a=viewHarness({realLocale:true}),snap=sampleCard(),day=snap.report.day;
 snap.report.hours=[[day,9,24000,123],[day,10,21,80],[day,11,12,30]];
 snap.report.ranking=[8,9,6,0,1].map((key,i)=>({platform:'macos',modifiers:8,key,count:17-i}));
 const card=M.cardData(snap,day,'UTC','UTC');
 a.view.render('share',snap);
 const field=openReportPrompt(a);
 let copied;
 a.document.execCommand=()=>{copied=field.value.slice(field.selectionStart,field.selectionEnd);return true;};
 for(let i=0;i<24;i++){
  if(i)a.get('share-prompt-regenerate').fire('click');
  await a.get('share-prompt-copy').fire('click');
  assert.equal(copied,field.value);
  const labels=copied.split('\n');
  const selectedKeys=new Set();
  for(const id of Facts.available(card,H)){
   const parts=Facts.section(id,card,a.I,C,H,2);
   if(id==='shortcut'||id==='ranking'){
    let depth=0;
    if(id==='shortcut'&&labels.includes(parts.label))depth=1;
    if(id==='ranking')for(const n of [2,3]){
     if(labels.includes(Facts.section(id,card,a.I,C,H,n).label))depth=n;
    }
    for(const row of card.ranking.slice(0,depth))selectedKeys.add(row.key);
   }else{
    for(const line of parts.lines)assert.equal(copied.includes(line),labels.includes(parts.label),id);
   }
  }
  for(const row of card.ranking)assert.equal(copied.includes(C.chordParts(row).join(' + ')),selectedKeys.has(row.key));
  assert(copied.includes('Total inputs')&&copied.includes('09:00   24,123'));
 }
 a.view.render('share',{...snap,report:{...snap.report,hours:[],ranking:[]}});
 await a.get('share-prompt-copy').fire('click');assert(!copied.includes('⌘'));
});
test('daily prompts keep a 24-hour axis with only measured nonzero rows',()=>{
 const snap=sampleCard();snap.database_path='/private/marker';snap.lifetime={keyboard:100428};
 const prompt=promptFor(snap,'daily');
 const lines=prompt.split('\n').filter(l=>/^\d\d:00   /.test(l));
 assert.deepEqual(lines,['09:00   12']);assert(prompt.includes('Observed through the 12:00 hour'));
 assert(prompt.includes(messages.en['report.hours']));
 assert(!prompt.includes(messages.en['report_prompt.week']));
 assert(!prompt.includes('/private/marker')&&!prompt.includes('UTC')&&!prompt.includes('2026-11-16'));
 assert.doesNotMatch(prompt,/\{[a-z_]+\}|report_prompt\./);
});
test('report charts keep quiet hours and dates in place and label just the first positive peak',()=>{
 for(const period of ['daily','weekly']){
  const a=viewHarness({realLocale:true}),snap=sampleCard(),day=snap.report.day;
  const weekly=period==='weekly',dates=weekly?['2026-11-12','2026-11-15',day]:[day,day,day];
  const hours=weekly?[9,9,9]:[2,9,12],bins=weekly?[1,4,6]:hours;
  snap.report.period=period;snap.report.hours=dates.map((date,i)=>[date,hours[i],[7,30,30][i],0]);
  if(weekly)a.view.restoreSession({share_period:period});
  const ctx=a.get('share-canvas').getContext('2d'),bars=[];
  ctx.arcTo=(x,y,nextX,bottom)=>{if(x===nextX&&bottom===876&&y<bottom)bars.push({right:x,height:bottom-y});};
  a.view.render('share',snap);
  const step=888/(weekly?7:24);
  assert.equal(bars.length,3);
  for(const i of [1,2])assert(Math.abs(bars[i].right-bars[0].right-(bins[i]-bins[0])*step)<1e-8);
  assert(Math.abs(bars[0].height/bars[1].height-7/30)<1e-8);
  const labels=a.texts.filter(row=>row.y>=700&&row.y<876);
  assert.deepEqual(labels.map(row=>row.text),['30']);
  assert(Math.abs(labels[0].x-(96+(bins[1]+.5)*step))<1e-8);
  const field=openReportPrompt(a),factsAt=field.value.indexOf('Total inputs');
  assert(field.value.indexOf(messages.en['report_prompt.compose'])>=0&&field.value.indexOf(messages.en['report_prompt.compose'])<factsAt);
  assert(field.value.includes('proportional zero-baseline chart'));
  assert(!/^\S+ {3}0$/m.test(field.value));
 }
});
test('empty reports and constant observed hours have no chart value callouts',()=>{
 for(const flat of [false,true]){
  const a=viewHarness({realLocale:true}),snap=sampleCard();
  snap.report.hours=flat?Array.from({length:13},(_,h)=>[snap.report.day,h,8,0]):[];
  a.view.render('share',snap);
  assert.deepEqual(a.texts.filter(row=>row.y>=700&&row.y<876),[]);
  if(!flat)assert(openReportPrompt(a).value.includes('No inputs recorded in the observed period.'));
 }
});
test('daily and weekly PNGs keep the leader while prompt rankings expose exactly the selected depth',()=>{
 for(const period of ['daily','weekly']){
  const a=viewHarness({realLocale:true}),snap=sampleCard();
  a.view.restoreSession({share_period:period});
  snap.report.period=period;
  snap.report.ranking=[8,9,6,0,1].map((key,index)=>({key,modifiers:8,platform:'macos',count:5-index}));
  a.view.render('share',snap);
  const printed=a.texts.map(t=>t.text).filter(text=>text.includes('⌘'));
  assert.deepEqual(printed,['⌘ + C  ·  5 times']);
  const card=M.cardData(snap,snap.report.day,'UTC','UTC',period);
  for(const depth of [0,1,2,3]){
   const focus=depth===0?[]:[depth===1?'shortcut':'ranking'];
   const prompt=promptForCard(card,focus,depth);
   for(const [i,row] of card.ranking.entries()){
    assert.equal(prompt.includes(C.chordParts(row).join(' + ')),i<depth);
   }
  }
 }
});
test('daily and weekly card and prompt rankings discard foreign shortcuts before taking the top five',()=>{
 for(const period of ['daily','weekly'])for(const platform of ['macos','windows','linux']){
  const foreign=platform==='windows'?'macos':'windows';
  const snap=sampleCard();snap.report.period=period;
  const local={platform,modifiers:platform==='macos'?8:1,key:8,count:3};
  snap.report.ranking=[...[0,9,6,1,3].map(key=>({...local,platform:foreign,key,count:80})),local];
  const card=M.cardData(snap,snap.report.day,'UTC','UTC',period,platform);
  assert.deepEqual(card.ranking,[local]);
  const a=viewHarness({realLocale:true,platform});
  if(period==='weekly')a.view.restoreSession({share_period:period});
  a.view.render('share',snap);
  const chord=C.chordParts(local).join(' + ');
  assert(a.texts.some(row=>row.text===chord+'  ·  3 times'));
  const prompt=promptForCard(card,['shortcut']);
  assert(prompt.includes(chord));
  assert(!prompt.includes(C.chordParts(snap.report.ranking[0]).join(' + ')));
 }
});
test('weekly reports roll up seven dates, separate the single busiest hour from the most active clock hour, and keep the ranked candidates available to the selector',()=>{
 const snap=sampleCard();const {day}=snap.report;
 const rows=[['2026-11-10',17,9999,0],['2026-11-11',10,10,2],['2026-11-12',10,0,13],['2026-11-15',19,15,3],['2026-11-17',9,2,0]];
 snap.report={day,period:'weekly',timezone:'UTC',hours:rows,ranking:[8,9,6,0,1].map((key,i)=>({key,count:10-i,platform:'macos',modifiers:8}))};
 const card=M.cardData(snap,day,'UTC','UTC','weekly');
 assert.equal(card.start,'2026-11-11');assert.equal(card.days.length,7);assert.deepEqual(card.days.map(d=>d.total),[12,13,0,0,18,0,2]);
 assert.equal(card.keyboard,27);assert.equal(card.mouse,18);assert.deepEqual(card.peak,{day:'2026-11-15',hour:19,total:18});
 assert.deepEqual(card.clockPeak,{hour:10,dailyAverage:25/7});assert.equal(card.ranking.length,5);
 for(const locale of Object.keys(messages)){
  const prompt=promptFor(snap,'weekly','UTC','UTC',['peak','rhythm']);assert(prompt.includes('2026-11-15 · 19:00–20:00'));
  const a=viewHarness({realLocale:true});a.I.setLocale(locale);a.view.restoreSession({share_period:'weekly'});a.view.render('share',snap);
  const average='10:00–11:00 · '+a.I.t('report.daily_average',{count:a.I.number(3.6)});
  assert(a.texts.some(row=>row.text===average));assert(prompt.includes('10:00–11:00 · 3.6 inputs/day'));assert(!prompt.includes('⌘'));assert(!prompt.includes('2026-11-10'));
  assert.equal(prompt.split('\n').filter(l=>/^2026-11-\d\d   /.test(l)).length,4);
  assert(!prompt.includes(messages.en['report_prompt.hour_chart']));
 }
 const foreign=promptFor(snap,'weekly','UTC','Asia/Shanghai',['ranking']);assert(!foreign.includes('⌘'));
 snap.report.hours=[['2026-11-11',11,3,0],['2026-11-12',12,1,2]];
 const tied=M.cardData(snap,day,'UTC','UTC','weekly');assert.equal(tied.ties,2);assert.equal(tied.clockPeak,null);
 snap.report.hours=[];const empty=M.cardData(snap,day,'UTC','UTC','weekly');assert.equal(empty.peak,null);assert.equal(empty.ranking.length,0);assert(!promptFor(snap,'weekly').includes('Share of inputs'));
});
test('switching report period blocks stale exports, updates the prompt, and restores the selected period',async()=>{
 const a=viewHarness({realLocale:true}),snap=sampleCard();a.view.render('share',snap);
 const text=openReportPrompt(a);assert(text.value.includes('09:00   12'));
 a.periods[1].fire('click');assert.equal(a.get('share-save').disabled,true);assert.equal(text.value,'');assert.equal(a.view.sessionState().share_period,'weekly');
 a.view.render('share',snap);assert.equal(a.get('share-save').disabled,true);
 const weekly={...snap,report:{...snap.report,period:'weekly'}};
 a.view.render('share',weekly);assert.equal(a.get('share-save').disabled,false);assert(text.value.includes('2026-11-11'));
 await a.get('share-save').fire('click');assert.equal(a.sent.at(-1)[1].period,'weekly');
 const b=viewHarness();b.view.restoreSession(a.view.sessionState());b.view.render('share',weekly);assert.equal(b.periods[1].attributes['aria-pressed'],'true');
});
test('current daily and weekly reports show only the date and an optional demo label',()=>{
 for(const period of ['daily','weekly']){
  const snap=sampleCard();snap.report.period=period;
  const dates=period==='weekly'?'2026-11-11 — 2026-11-17':'2026-11-17';
  for(const demo of [false,true]){
   snap.demo=demo;
   const a=viewHarness({realLocale:true});
   if(period==='weekly')a.view.restoreSession({share_period:period});
   a.view.render('share',snap);
   const notes=a.texts.filter(t=>t.x===72&&t.y===1282);
   assert.deepEqual(notes.map(t=>t.text),demo?['Demo data']:[]);
   if(demo)assert(notes[0].x+notes[0].max+24<=1008-('github.com/endNone/ClickPulse'.length*10)-12-32);
   const prompt=promptFor(snap,period);
   const dateLine=prompt.split('\n').find(line=>line.startsWith(dates));
   assert.equal(dateLine,demo?dates+' · '+messages.en['share.demo']:dates);
   if(period==='daily')assert(prompt.includes('Observed through the 12:00 hour'));
  }
 }
});
test('AI report prompts distinguish rounded shares, partial reports, and sparse weekly dates',()=>{
 const snap=sampleCard();snap.now='2026-11-18T12:00:00Z';
 for(const [k,m] of [[1,99999],[99999,1]]){
  snap.report.hours=[[snap.report.day,9,k,m]];const text=promptFor(snap,'daily','UTC','UTC',['mix']);
  assert(text.includes('<0.1%')&&text.includes('>99.9%'));assert(!text.includes('23:00   —'));
 }
 snap.report.period='weekly';
 const ledger=Recipes.describe(Recipes.createPicker(()=>0).next({kind:'week',sparse:true}),'en').chart.instruction;
 snap.report.hours=[];assert(promptFor(snap,'weekly').includes(ledger));
 snap.report.hours.push([snap.report.day,9,1,0]);assert(promptFor(snap,'weekly').includes(ledger));
 snap.report.hours.push(['2026-11-16',9,2,0]);assert(!promptFor(snap,'weekly').includes(ledger));
});
test('the Report panel matches the active interface language and omits zero rows',()=>{
 for(const locale of Object.keys(messages)){
  const a=viewHarness({realLocale:true}),snap=sampleCard();
  a.I.setLocale(locale);snap.demo=true;a.view.render('share',snap);
  const field=openReportPrompt(a);
  assert(field.value.startsWith(messages[locale]['report_prompt.brief']),locale);
  assert(field.value.includes('2026-11-17 · '+messages[locale]['share.demo']),locale);
  assert.equal(field.value.includes('   0\n'),false,locale);
  field.value='My edited brief';field.fire('input');a.I.setLocale(locale==='en'?'fr':'en');a.view.render('share',snap);
  assert.equal(field.value,'My edited brief');
 }
});
test('all AI prompt translations have the same parameters as the English template',()=>{
 const parameters=text=>[...new Set(text.match(/\{[a-zA-Z_]+\}/g)||[])].sort();
 const keys=table=>Object.keys(table).filter(key=>key.startsWith('report_prompt.')).sort();
 const expected=keys(messages.en);assert(expected.length>0);
 for(const [locale,table] of Object.entries(messages)){
  assert.deepEqual(keys(table),expected,locale);
  for(const key of expected){
   assert(table[key].trim(),locale+' '+key);
   assert.deepEqual(parameters(table[key]),parameters(messages.en[key]),locale+' '+key);
  }
 }
});

function openReportPrompt(a){
 const panel=a.get('share-prompt-panel');panel.open=true;panel.fire('toggle');
 return a.get('share-prompt-text');
}

test('changing the report date clears the old prompt until its shortcut data arrives; leaving releases it',()=>{
 const a=viewHarness({realLocale:true}),data={...sampleCard(),report:report('2026-11-17',[{platform:'macos',modifiers:8,key:8,count:5}])};
 a.view.render('share',data);
 const text=openReportPrompt(a),button=a.get('share-prompt-copy');
 assert(text.value.includes('2026-11-17'));
 a.get('share-date').fire('change','2026-11-16');
 assert.equal(text.value,'');assert.equal(button.disabled,true);
 const historical={...data,report:report('2026-11-16')};
 a.view.render('share',historical);openReportPrompt(a);
 assert(text.value.includes('2026-11-16')&&!text.value.includes('⌘ + C'));assert.equal(button.disabled,false);
 for(const view of [{compact:true,visible:true},{compact:false,visible:false}]){
  a.view.setWindowState(view);assert.equal(text.value,'');
  a.view.setWindowState({compact:false,visible:true});openReportPrompt(a);
  assert(text.value.includes('2026-11-16'));
 }
 a.view.render('overview',data);assert.equal(text.value,'');
});

test('regeneration rotates compatible designs and preserves report figures and the PNG',()=>{
 const mediumCount=Object.keys(Recipes.registry.medium).length;
 for(const period of ['daily','weekly']){
  const a=viewHarness({realLocale:true}),snap=sampleCard();
  snap.report.period=period;
  snap.report.hours.push([snap.report.day,10,3,1],['2026-11-16',8,8,2]);
  if(period==='weekly')a.periods[1].fire('click');
  a.view.render('share',snap);
  const field=openReportPrompt(a),regenerate=a.get('share-prompt-regenerate');
  const base=()=>field.value.split('\n').filter(line=>/^2026-11-|^Total inputs|^\d{2}:\d{2}   /.test(line));
  const originalBase=base(),initialDraws=a.draws();
  assert(originalBase.some(line=>line.startsWith('Total inputs')));assert.equal(regenerate.disabled,false);
  let lastMedium='';
  for(let cycle=0;cycle<2;cycle++){
   const seen=new Set();
   for(let i=0;i<mediumCount;i++){
    if(i||cycle)regenerate.fire('click');
    const medium=Object.values(Recipes.registry.medium).find(m=>field.value.split('\n').some(line=>line.startsWith(m.words.en.label+' — ')))?.words.en.label;
    assert(medium && medium!==lastMedium);assert(!seen.has(medium));seen.add(medium);lastMedium=medium;
    assert.deepEqual(base(),originalBase);
    const before=field.value;
    a.view.render('share',snap);assert.equal(field.value,before);
   }
  }
  assert.equal(a.draws(),initialDraws);assert.equal(a.exports(),0);assert.deepEqual(a.sent,[]);
  const design=field.value;
  a.get('share-style-select').fire('change','ink');assert.equal(field.value,design);
  a.get('share-date').fire('change','2026-11-16');
  assert(regenerate.disabled);regenerate.fire('click');assert.equal(field.value,'');
 }
});

test('the live prompt keeps its medium and adapts the chart as more data arrives',()=>{
 const a=viewHarness({realLocale:true}),snap=sampleCard();
 a.view.render('share',snap);const text=openReportPrompt(a);
 const medium=Object.values(Recipes.registry.medium).find(m=>text.value.includes(m.words.en.label+' — ')).words.en.label;
 const ledger=Recipes.registry.chart.ledger.en.label+' — ';
 assert(text.value.includes(ledger));
 snap.report.hours.push([snap.report.day,10,4,0]);
 a.view.render('share',snap);
 assert(text.value.includes(medium+' — '));assert(!text.value.includes(ledger));
});

test('copying an AI prompt copies only text and leaves it selected for manual copy on failure',async()=>{
 for(const result of [true,false]){
  const a=viewHarness({realLocale:true,platform:'linux'});a.view.render('share',sampleCard());
  const text=openReportPrompt(a),expected=text.value;assert(expected.length>0);
  let attempts=0;
  a.document.execCommand=command=>{
   assert.equal(command,'copy');assert.equal(text.value.slice(text.selectionStart,text.selectionEnd),expected);
   attempts++;return result;
  };
  await a.get('share-prompt-copy').fire('click');
  assert.equal(attempts,1);assert.deepEqual(a.sent,[]);assert.equal(a.exports(),0);
  assert.deepEqual(a.toasts,[a.I.t(result?'report_prompt.copied':'report_prompt.manual')]);
  if(!result){
   assert.equal(a.document.activeElement,text);
   assert.equal(text.value.slice(text.selectionStart,text.selectionEnd),expected);
  }
 }
});

test('edited report text survives live refreshes and copies verbatim until a new prompt is requested',()=>{
 const a=viewHarness({realLocale:true}),snap=sampleCard();
 a.view.render('share',snap);
 const text=openReportPrompt(a),copy=a.get('share-prompt-copy'),regenerate=a.get('share-prompt-regenerate');
 const draft='我的海报\n保留留白。';
 assert.equal(text.disabled,false);
 text.fire('input',draft);
 snap.report.hours.push([snap.report.day,10,8,1]);
 a.view.render('share',snap);
 assert.equal(text.value,draft);
 const panel=a.get('share-prompt-panel');panel.open=false;panel.fire('toggle');
 openReportPrompt(a);
 let copied;
 a.document.execCommand=()=>{copied=text.value.slice(text.selectionStart,text.selectionEnd);return true;};
 copy.fire('click');assert.equal(copied,draft);
 text.fire('input','');
 a.view.setWindowState({compact:true});a.view.setWindowState({compact:false,visible:true});
 assert.equal(text.value,'');assert.equal(copy.disabled,true);assert.equal(regenerate.disabled,false);
 regenerate.fire('click');
 assert(text.value.includes(snap.report.day));assert.equal(copy.disabled,false);
 text.fire('input',draft);a.get('share-date').fire('change','2026-11-16');
 assert.equal(text.disabled,true);assert.equal(copy.disabled,true);
 a.view.render('share',{...snap,report:report('2026-11-16')});
 assert.equal(text.disabled,false);assert(text.value.includes('2026-11-16'));assert(!text.value.includes(draft));
});

test('weekly dates enforce 2 to 31 day bounds; only the matching range can be copied or exported',async()=>{
 const a=viewHarness({realLocale:true}),snap=sampleCard();
 a.view.render('share',snap);
 assert.equal(a.get('share-start-field').hidden,true);
 a.periods[1].fire('click');
 const start=a.get('share-start'),end=a.get('share-date');
 assert.equal(a.get('share-start-field').hidden,false);
 assert.equal(start.value,'2026-11-11');
 assert.equal(start.min,'2026-10-18');
 assert.equal(start.max,'2026-11-16');
 start.fire('change','2026-10-17');assert.equal(start.value,'2026-11-11');
 start.fire('change','2026-11-17');assert.equal(start.value,'2026-11-11');
 start.fire('change','2026-11-14');
 assert.equal(end.value,'2026-11-17');assert.equal(end.min,'2026-11-15');
 end.fire('change','2026-11-14');assert.equal(end.value,'2026-11-17');
 end.fire('change','2026-11-16');assert.equal(start.value,'2026-11-14');
 start.fire('change','2026-11-16');assert.equal(start.value,'2026-11-14');
 assert.equal(start.max,'2026-11-15');
 const native={...snap,report:{...report(end.value),period:'weekly',start:start.value}};
 const text=openReportPrompt(a);
 a.view.render('share',{...native,report:{...native.report,start:'2026-11-13'}});
 assert.equal(text.value,'');assert.equal(a.get('share-save').disabled,true);
 a.view.render('share',native);
 assert(text.value.includes('2026-11-14 — 2026-11-16'));
 assert(text.value.includes(messages.en['report_prompt.range_chart']));
 assert(!text.value.includes(messages.en['report_prompt.week']));
 await a.get('share-save').fire('click');
 assert.equal(a.sent.at(-1)[1].start,'2026-11-14');assert.equal(a.sent.at(-1)[1].day,'2026-11-16');
 const b=viewHarness();b.view.restoreSession(a.view.sessionState());b.view.render('share',native);
 assert.equal(b.get('share-start').value,start.value);assert.equal(b.get('share-date').value,end.value);
 a.periods[0].fire('click');assert.equal(a.get('share-start-field').hidden,true);assert.equal(end.min,'');
});

test('multi-week selections keep one unmerged daily column per date and clamp out-of-range starts',()=>{
 const snap=sampleCard(),end=snap.report.day,start='2026-10-31';
 snap.report={...snap.report,start,period:'weekly',hours:[['2026-10-30',10,500,0],[start,23,2,0],['2026-11-04',1,0,4],[end,12,8,1],['2026-11-18',0,500,0]]};
 const card=M.cardData(snap,end,'UTC','UTC','weekly','macos',start);
 assert.equal(card.span,18);assert.equal(card.days.length,18);
 assert.equal(card.days[0].day,start);assert.equal(card.days.at(-1).day,end);
 for(const col of card.days)assert.equal(col.day,col.end);
 assert.equal(card.days.reduce((sum,bin)=>sum+bin.total,0),15);assert.equal(card.keyboard,10);assert.equal(card.mouse,5);
 const prompt=promptForCard(card);
 assert(prompt.includes(messages.en['report_prompt.range_series']));
 for(const bin of card.days)assert.equal(prompt.includes(bin.day+'   '+bin.total),bin.total>0);
 // Same-day start clamps to minimum 2 days; >31 days clamps to 31 days.
 snap.report.start='2026-11-16';
 const clampedMin=M.cardData(snap,end,'UTC','UTC','weekly','macos',end);
 assert.equal(clampedMin.span,2);assert.equal(clampedMin.start,'2026-11-16');
 snap.report.start='2026-10-18';
 const clampedMax=M.cardData(snap,end,'UTC','UTC','weekly','macos','2026-01-01');
 assert.equal(clampedMax.span,31);assert.equal(clampedMax.start,'2026-10-18');
});

test('card colors survive a renderer restart and are independent of the AI prompt',()=>{
 const snap=sampleCard(),a=viewHarness({realLocale:true});
 a.view.render('share',snap);const prompt=openReportPrompt(a),original=prompt.value;
 const colors={pine:'#31664e',clay:'#984d28',lavender:'#70538c',rose:'#99495c'};
 for(const [style,accent] of Object.entries(colors)){
  a.texts.length=0;a.get('share-style-select').fire('change',style);
  assert.equal(a.texts.find(row=>row.y===372).color,accent);
  assert.equal(prompt.value,original);
  const b=viewHarness();b.view.restoreSession(a.view.sessionState());b.view.render('share',snap);
  assert.equal(b.get('share-style-select').value,style);
  assert.equal(b.texts.find(row=>row.y===372).color,accent);
 }
 a.view.restoreSession({share_style:'unknown'});a.view.render('share',snap);
 assert.equal(a.get('share-style-select').value,'blue');
});
