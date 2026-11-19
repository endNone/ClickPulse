'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const C=require('../ui/shortcuts-catalog.js'),M=require('../ui/insights.js');
function viewHarness({lateBrand=false,fonts,platform='macos',preview=false,timeZone='UTC',systemTimeZone='UTC'}={}){
 const elements=new Map(),texts=[],sent=[],toasts=[];let draws=0,exports=0;
 const ctx=new Proxy({measureText:text=>({width:text.length*10}),fillRect:()=>{draws++;},fillText:(text,x,y,max)=>texts.push({text,x,y,max,font:ctx.font,align:ctx.textAlign,color:ctx.fillStyle})}, {get:(target,key)=>target[key]||(()=>{})});
 function element(){return {children:[],options:[{}],dataset:{},style:{},handlers:{},attributes:{},replacements:0,width:1080,height:1350,
  addEventListener(type,handler){this.handlers[type]=handler;},setAttribute(key,value){this.attributes[key]=value;},getContext:()=>ctx,toDataURL:()=>{exports++;return 'data:image/png;base64,test';},
  append(...children){this.children.push(...children);},replaceChildren(...children){this.children=children;this.replacements++;},
  fire(type,value){if(value!==undefined)this.value=value;return this.handlers[type]?.({target:this});}};}
 const get=id=>{if(!elements.has(id))elements.set(id,element());return elements.get(id);};
 const styles=['blue','paper','ink'].map(style=>{const el=element();el.dataset.shareStyle=style;return el;});
 const brand=element();brand.complete=false;brand.naturalWidth=0;
 const document={fonts,getElementById:get,createElement:element,createElementNS:element,querySelector:()=>lateBrand?brand:null,querySelectorAll:selector=>selector==='[data-share-style]'?styles:[],documentElement:{dataset:{theme:'blue'}}};
 global.ClickPulseShortcutCatalog=C;
 const window={captureSessionState(){},clickpulsePlatform:platform,clickpulsePreview:preview},synced=[];
 const I={locale:'en',timezone:timeZone,effectiveTimeZone:timeZone,t:(key,vars)=>key+(vars?.count||''),number:String,percent:(ratio,digits)=>(ratio*100).toFixed(digits)+'%',day:String,date:String};
 const view=M.create({document,window,I,T:{get:()=>({empty:'#eee',accent:'#123',mouse:'#456',navy:'#789'})},send:(...args)=>sent.push(args),toast:message=>toasts.push(message),getSystemTimeZone:()=>systemTimeZone,syncSelect(id){synced.push([id,get(id).value]);}});
 delete global.ClickPulseShortcutCatalog;
 return {view,get,styles,document,window,brand,I,synced,texts,sent,toasts,setSystemTimeZone:zone=>{systemTimeZone=zone;},draws:()=>draws,exports:()=>exports};
}
const sampleCard=()=>({share_shortcut:{day:'2026-11-17',top:null},now:'2026-11-17T12:00:00Z',history_days:[{day:'2026-11-17',counts:{keyboard:10,mouse:2}},{day:'2026-11-16',counts:{keyboard:5,mouse:1}}]});
test('leaving share frees its canvas and returning redraws unchanged data',()=>{
 const a=viewHarness(),data=sampleCard();a.view.render('share',data);assert.equal(a.draws(),1);
 a.view.render('overview',data);assert.equal(a.get('share-canvas').width,0);
 a.view.render('share',data);assert.equal(a.get('share-canvas').width,1080);assert.equal(a.get('share-canvas').height,1350);assert.equal(a.draws(),2);
});
test('share restores date/style across renderer disposal without preserving search text',()=>{
 const a=viewHarness(),data=sampleCard();a.view.render('share',data);
 a.styles[2].fire('click');a.get('share-date').fire('change','2026-11-16');
 a.get('shortcut-search').fire('input','private query');
 const saved=a.view.sessionState();assert.equal(saved.share_style,'ink');assert.equal(saved.share_day,'2026-11-16');assert(!JSON.stringify(saved).includes('private'));
 const b=viewHarness();b.view.restoreSession(saved);b.view.render('share',data);
 assert.equal(b.get('share-date').value,'2026-11-16');assert.equal(b.styles[2].attributes['aria-pressed'],'true');
 b.get('share-date').fire('change','2026-11-20');assert.equal(b.get('share-date').value,'2026-11-16');
 b.get('share-date').fire('change','2026-02-30');assert.equal(b.get('share-date').value,'2026-11-16');
 const later={...data,history_days:[...data.history_days,{day:'2026-01-01',counts:{keyboard:999,mouse:1}}]};b.view.render('share',later);assert.equal(b.draws(),1);
});
test('shortcut list is bounded, searchable, and library rows survive live count updates',()=>{
 const a=viewHarness();const data={shortcuts:Array.from({length:250},(_,i)=>({platform:'macos',modifiers:8+(i%2),key:Math.floor(i/2),count:i+1}))};
 a.view.render('shortcuts',data);assert.equal(a.get('shortcut-list').children.length,100);
 a.get('shortcut-more').fire('click');assert.equal(a.get('shortcut-list').children.length,200);
 a.get('shortcut-library').fire('click');assert.equal(a.get('shortcut-list').children.length,100);
 const replacements=a.get('shortcut-list').replacements;
 a.view.render('shortcuts',{shortcuts:[{platform:'macos',modifiers:8,key:8,count:9999}]});assert.equal(a.get('shortcut-list').replacements,replacements);
 a.get('shortcut-search').fire('input','Ctrl+C');assert(a.get('shortcut-list').children.length<20);
 a.get('shortcut-system').fire('change','linux');assert.equal(a.view.sessionState().shortcut_system,'linux');
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
test('mouse button grayscale follows relative counts, including the wheel, ties and empty history',()=>{
 const a=viewHarness();
 const levels=()=>{
  const found={};
  function walk(el){const key=el.attributes?.['data-mouse-button'];if(key){const rgb=el.attributes.fill.match(/\d+/g).map(Number);assert.equal(rgb[0],rgb[1]);assert.equal(rgb[1],rgb[2]);found[key]=rgb[0];}el.children.forEach(walk);}
  walk(a.get('mouse-diagram'));return found;
 };
 a.view.render('keys',{lifetime:{mouse:10000},mouse_buttons:{left:100,right:20,middle:2,side:0}});
 let color=levels();assert(color.left<color.right);assert(color.right<color.middle);assert(color.middle<color.side);
 const unchanged={...color};a.view.render('keys',{lifetime:{mouse:1000000},mouse_buttons:{left:100,right:20,middle:2,side:0}});assert.deepEqual(levels(),unchanged,'unknown history does not distort physical heat');
 a.view.render('keys',{mouse_buttons:{left:2,right:20,middle:100,side:20}});
 color=levels();assert(color.middle<color.right);assert(color.right<color.left);assert.equal(color.side,color.right);
 a.view.render('keys',{mouse_buttons:{}});assert.equal(new Set(Object.values(levels())).size,1);assert(Object.values(levels()).every(level=>level>230));
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
test('renderer restore synchronizes the platform picker and keeps expanded result rows',()=>{
 const a=viewHarness(),data={shortcuts:[]};a.view.render('shortcuts',data);
 a.get('shortcut-library').fire('click');a.get('shortcut-more').fire('click');
 const b=viewHarness();b.view.restoreSession(a.view.sessionState());b.view.render('shortcuts',data);
 assert.equal(b.get('shortcut-list').children.length,C.entries.length);
 b.view.restoreSession({...a.view.sessionState(),shortcut_system:'windows'});b.view.render('shortcuts',data);
 assert(b.synced.some(([id,value])=>id==='shortcut-system'&&value==='windows'));
 assert.equal(b.get('shortcut-list').children.length,C.entries.filter(e=>e.platform==='windows').length);
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
test('filtered shortcut empty state changes when activity arrives on another platform',()=>{
 const a=viewHarness();a.view.render('shortcuts',{shortcuts:[]});a.get('shortcut-system').fire('change','windows');
 assert.equal(a.get('shortcut-list').children[0].textContent,'shortcuts.empty');
 a.view.render('shortcuts',{shortcuts:[{platform:'macos',modifiers:8,key:8,count:1}]});
 assert.equal(a.get('shortcut-list').children[0].textContent,'shortcuts.no_results');
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
 assert.equal(C.entries.length,111);
 const ids=new Set();for(const e of C.entries){assert(!ids.has(e.id));ids.add(e.id);assert(C.sources[e.source].url.startsWith('https://'));assert(C.keyLabels[e.key]);assert(e.title.en&&e.title.zh);assert(e.modifiers>0&&e.modifiers<16);}
 assert.equal(C.matches({platform:'macos',modifiers:8,key:8})[0].title.en,'Copy');
 assert.equal(C.matches({platform:'windows',modifiers:8,key:8}).length,0);
 assert.equal(C.matches({platform:'linux',modifiers:5,key:8})[0].scope,'Terminal');
 assert.equal(C.matches({platform:'macos',modifiers:4,key:8}).length,0);
 assert.deepEqual(C.chordParts({platform:'linux',modifiers:12,key:123}),['Shift','Super','Left']);
});
test('shortcut ranking keeps unknown chords and searches both languages',()=>{
 const rows=[{platform:'macos',modifiers:8,key:8,count:7},{platform:'macos',modifiers:9,key:7,count:9},{platform:'windows',modifiers:1,key:8,count:2}];
 const sorted=M.shortcutRows(rows,C,'','');assert.equal(sorted[0].count,9);assert.equal(sorted[0].entries.length,0);
 assert.equal(M.shortcutRows(rows,C,'复制','macos').length,1);
 assert.equal(M.shortcutRows(rows,C,'copy','windows')[0].count,2);
});
test('shortcut chart totals include unlisted combinations and split top five exactly',()=>{
 const raw=Array.from({length:10},(_,i)=>({platform:'macos',modifiers:9,key:200+i,count:10-i}));
 const rows=M.shortcutRows([...raw,{...raw[0],count:0},{...raw[0],count:-5}],C,'','');
 const summary=M.shortcutSummary(rows);
 assert.equal(summary.total,55);assert.equal(summary.distinct,10);assert.equal(summary.top.length,8);
 assert.equal(summary.topFive,40);assert.equal(summary.others,15);
 assert.equal(summary.top[0].count,10);assert.equal(summary.top[7].count,3);
 assert.deepEqual(M.shortcutSummary([]),{total:0,distinct:0,top:[],topFive:0,others:0});
});
test('platform changes scope all charts while detail search leaves totals unchanged',()=>{
 const a=viewHarness(),data={shortcuts:[{platform:'macos',modifiers:8,key:8,count:30},{platform:'macos',modifiers:8,key:9,count:10},{platform:'windows',modifiers:1,key:8,count:60}]};
 a.view.render('shortcuts',data);
 const metrics=a.get('shortcut-metrics');assert.equal(metrics.children[0].children[1].textContent,'100');
 const ranking=a.get('shortcut-ranking');assert.equal(ranking.children.length,3);
 const chart=ranking.children[0];a.get('shortcut-search').fire('input','Cmd+V');
 assert.equal(a.get('shortcut-list').children.length,1);assert.equal(metrics.children[0].children[1].textContent,'100');assert.equal(ranking.children[0],chart);
 a.get('shortcut-system').fire('change','macos');assert.equal(metrics.children[0].children[1].textContent,'40');
 assert.equal(metrics.children[1].children[1].textContent,'2');assert.equal(ranking.children.length,2);
 assert.equal(ranking.children[0].children[2].children[1].textContent,'75.0%');
 a.get('shortcut-system').fire('change','windows');assert.equal(metrics.children[0].children[1].textContent,'60');
 assert.equal(ranking.children.length,1);assert.equal(a.get('shortcut-list').children[0].textContent,'shortcuts.no_results');
});
test('empty dashboard has no invented bars or percentage and encyclopedia hides statistics',()=>{
 const a=viewHarness();a.view.render('shortcuts',{shortcuts:[]});
 assert.equal(a.get('shortcut-metrics').children[0].children[1].textContent,'0');
 assert.equal(a.get('shortcut-ranking').children[0].className,'shortcut-chart-empty');
 const ring=a.get('shortcut-concentration').children[0];assert.equal(ring.children[0].children.length,1);assert.equal(ring.children[1].children[0].textContent,'—');
 a.get('shortcut-library').fire('click');assert(a.get('shortcut-dashboard').hidden);assert(a.get('shortcut-period').hidden);
 a.get('shortcut-activity').fire('click');assert.equal(a.get('shortcut-dashboard').hidden,false);
});
test('live shortcut charts update percentages when counts change outside the top eight',()=>{
 const a=viewHarness(),rows=Array.from({length:10},(_,i)=>({platform:'macos',modifiers:8,key:i,count:100-i*10}));
 a.view.render('shortcuts',{shortcuts:rows});const first=a.get('shortcut-ranking').children[0];
 assert.equal(first.children[2].children[1].textContent,'18.2%');
 a.view.render('shortcuts',{shortcuts:[...rows.slice(0,9),{...rows[9],count:19}]});
 assert.equal(a.get('shortcut-metrics').children[0].children[1].textContent,'559');
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
 const card=M.cardData(snapshot,'2026-11-01','America/Los_Angeles');assert.equal(card.keyboard,25);assert.equal(card.mouse,10);assert(card.partial);assert.equal(card.week.length,7);assert.equal(card.week[0].day,'2026-10-26');
 assert.equal(M.cardData(snapshot,'2026-10-31','UTC').keyboard,0);assert.equal(M.cardData(snapshot,'2026-02-30','UTC'),null);
 assert(!Object.hasOwn(card,'top_keys'));assert(!Object.hasOwn(card,'shortcuts'));
});

test('Shift-only combinations are hidden by default, included on request, and restored per session',()=>{
 const a=viewHarness(),rows=[{platform:'windows',modifiers:4,key:123,count:100},{platform:'windows',modifiers:5,key:123,count:10},{platform:'windows',modifiers:1,key:8,count:5}];
 a.view.render('shortcuts',{shortcuts:rows});
 assert.equal(a.get('shortcut-metrics').children[0].children[1].textContent,'15');
 assert.equal(a.get('shortcut-list').children.length,2);
 const toggle=a.get('shortcut-show-shift');toggle.checked=true;toggle.fire('change');
 assert.equal(a.get('shortcut-metrics').children[0].children[1].textContent,'115');
 assert.equal(a.get('shortcut-list').children.length,3);
 const b=viewHarness();b.view.restoreSession(a.view.sessionState());b.view.render('shortcuts',{shortcuts:rows});
 assert.equal(b.get('shortcut-show-shift').checked,true);
 assert.equal(b.get('shortcut-list').children.length,3);
 assert.equal(viewHarness().view.sessionState().shortcut_show_shift,false);
 toggle.checked=false;toggle.fire('change');a.get('shortcut-library').fire('click');
 assert.equal(a.get('shortcut-shift-filter').hidden,true);
 a.get('shortcut-search').fire('input','Shift+Tab');
 assert(a.get('shortcut-list').children.some(row=>row.className==='shortcut-row'));
 a.get('shortcut-activity').fire('click');assert.equal(a.get('shortcut-shift-filter').hidden,false);
});
test('unrequested shortcut data shows loading, never a false zero or the previous ranking',()=>{
 const a=viewHarness();a.view.render('shortcuts',{});
 assert.equal(a.get('shortcut-loading').hidden,false);
 assert.equal(a.get('shortcut-dashboard').hidden,true);assert.equal(a.get('shortcut-details').hidden,true);
 a.view.render('shortcuts',{shortcuts:[]});assert.equal(a.get('shortcut-loading').hidden,true);
 assert.equal(a.get('shortcut-metrics').children[0].children[1].textContent,'0');
 a.view.render('shortcuts',{shortcuts:[{platform:'macos',modifiers:8,key:8,count:12}]});
 a.view.render('overview',{});a.view.render('shortcuts',{});
 assert.equal(a.get('shortcut-dashboard').hidden,true);assert.equal(a.get('shortcut-details').hidden,true);
 a.get('shortcut-library').fire('click');assert.equal(a.get('shortcut-loading').hidden,true);
 assert.equal(a.get('shortcut-details').hidden,false);assert(a.get('shortcut-list').children.length>0);
 a.get('shortcut-activity').fire('click');assert.equal(a.get('shortcut-loading').hidden,false);
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
 a.view.render('share',sampleCard());await settle();assert.equal(a.draws(),1);assert(a.texts.some(t=>t.text==='2026-11-16'));
});
test('card footer has the project URL with no time zone; native copy and save share the current canvas',async()=>{
 const a=viewHarness();a.view.render('share',sampleCard());
 const note=a.texts.find(t=>t.x===72&&t.y===1282);assert.equal(note.max,580);
 const footer=a.texts.find(t=>t.x===1008&&t.y===1282);assert.equal(footer.text,'github.com/endNone/ClickPulse');assert.equal(footer.align,'right');assert(footer.font.startsWith('400 18px'));
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
test(`${platform}: UTC cards omit recording-local shortcuts on an Asia/Shanghai system and still export`,async()=>{
 const a=viewHarness({platform,timeZone:'UTC',systemTimeZone:'Asia/Shanghai'}),canCopy=platform!=='linux';
 const data={...sampleCard(),timezone:'UTC',share_shortcut:{day:'2026-11-17',top:{platform,modifiers:platform==='macos'?8:1,key:8,count:99}}};
 a.view.render('share',data);
 assert(!a.texts.some(t=>t.text==='share.top_shortcut'||t.text.includes('share.shortcut_count')));
 assert(!a.get('share-canvas').attributes['aria-label'].includes('share.top_shortcut'));
 assert(a.texts.some(t=>t.text==='12'));assert(a.texts.some(t=>t.text==='10'));assert(a.texts.some(t=>t.text==='2'));
 assert.equal(a.get('share-save').disabled,false);
 assert.equal(a.get('share-copy').hidden,!canCopy);assert.equal(a.get('share-copy').disabled,!canCopy);
 await a.get('share-save').fire('click');await a.get('share-copy').fire('click');
 assert.deepEqual(a.sent.map(item=>item[0]),canCopy?['save_share_card','copy_share_card']:['save_share_card']);
 assert(a.sent.every(item=>item[1].day==='2026-11-17'));
 // No need to wait for a query whose result cannot be shown in this time zone.
 const {share_shortcut,...pending}=data;a.view.render('share',pending);
 assert.equal(a.get('share-save').disabled,false);assert.equal(a.get('share-copy').disabled,!canCopy);
});

test(`${platform}: card shortcut visibility follows timezone changes, including aliases and system changes`,()=>{
 const a=viewHarness({platform,timeZone:'Asia/Shanghai',systemTimeZone:'Asia/Shanghai'});
 const data={...sampleCard(),share_shortcut:{day:'2026-11-17',top:{platform,modifiers:platform==='macos'?8:1,key:8,count:5}}};
 a.view.render('share',data);assert(a.texts.some(t=>t.text==='share.top_shortcut'));
 assert(a.texts.some(t=>t.text===(platform==='macos'?'⌘ + C':'Ctrl + C')+'  ·  share.shortcut_count5'));
 a.texts.length=0;a.I.effectiveTimeZone='UTC';a.I.timezone='UTC';a.view.render('share',data);
 assert(!a.texts.some(t=>t.text==='share.top_shortcut'));
 a.texts.length=0;a.I.effectiveTimeZone='Asia/Chongqing';a.view.render('share',data);
 assert(a.texts.some(t=>t.text==='share.top_shortcut'));
 a.texts.length=0;a.setSystemTimeZone('America/Los_Angeles');a.view.render('share',data);
 assert(!a.texts.some(t=>t.text==='share.top_shortcut'));
 assert(M.cardData(data,'2026-11-17','system','Asia/Shanghai').shortcut);
});
}

test('card shows only the selected day leader, never cumulative or stale-date shortcuts',async()=>{
 const a=viewHarness(),data=sampleCard(),top={platform:'macos',modifiers:8,key:8,count:5};
 a.view.render('share',{...data,shortcuts:[{...top,count:999}],share_shortcut:{day:'2026-11-17',top}});
 assert.equal(a.texts.filter(t=>t.text==='share.top_shortcut').length,1);
 const value=a.texts.find(t=>t.text.includes('⌘ + C'));assert(value);assert.equal(value.text,'⌘ + C  ·  share.shortcut_count5');assert.equal(value.align,'right');assert.equal(value.x,1008);
 assert(!a.texts.some(t=>t.text.includes('999')));
 a.texts.length=0;a.get('share-date').fire('change','2026-11-16');
 assert(!a.texts.some(t=>t.text==='share.top_shortcut'));assert.equal(a.get('share-save').disabled,true);
 await a.get('share-copy').fire('click');assert.equal(a.exports(),0);
 a.view.render('share',{...data,share_shortcut:{day:'2026-11-16',top:{...top,platform:'windows',modifiers:1,count:12}}});
 assert(a.texts.some(t=>t.text==='Ctrl + C  ·  share.shortcut_count12'));
 assert.equal(a.get('share-copy').disabled,false);await a.get('share-copy').fire('click');assert.equal(a.sent[0][1].day,'2026-11-16');
 a.texts.length=0;a.view.render('share',{...data,share_shortcut:{day:'2026-11-16',top:null}});
 assert(!a.texts.some(t=>t.text==='share.top_shortcut'));assert.equal(a.get('share-save').disabled,false);
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
 data.history_days[0].counts.keyboard++;
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
