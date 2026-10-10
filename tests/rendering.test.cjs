const test=require('node:test');
const assert=require('node:assert/strict');
const {readFileSync}=require('node:fs');
const vm=require('node:vm');
const createI18n=require('../ui/i18n.js');
const H=require('../ui/history.js');
const K=require('../ui/keyboard.js');
const messages=require('../ui/locales.json');
const Themes=require('../ui/themes.js');
const Dropdowns=require('../ui/dropdown.js');

// Small DOM adapter to exercise the actual App event handlers and render flow.
class Element {
  constructor(id=''){
    this.id=id;this.dataset={};this.children=[];this.listeners={};this.attributes={};this.value='all';this.replacements=0;
    this.classList={toggle(){}};this.style={values:{},setProperty(key,value){this.values[key]=value;}};
  }
  append(...children){this.children.push(...children);}
  replaceChildren(...children){this.children=children;this.replacements++;}
  get lastElementChild(){return this.children.at(-1);}
  get firstElementChild(){return this.children[0];}
  get options(){return this.children;}
  setAttribute(name,value){this.attributes[name]=value;}
  addEventListener(event,handler){this.listeners[event]=handler;}
  fire(event='click'){this.listeners[event]?.({target:this,button:0,preventDefault(){}});}
  focus(){if(this.ownerDocument)this.ownerDocument.activeElement=this;}
}
function app(platform='macos',legacyWebKit=false){
  let document;
  const nodes=new Map(),get=id=>{
    if(!nodes.has(id))nodes.set(id,new Element(id));
    const node=nodes.get(id);node.ownerDocument=document;return node;
  };
  const nav=['overview','history','keys','settings'].map(id=>{const button=new Element();button.dataset.page=id;return button;});
  const pages=nav.map(button=>get(button.dataset.page));
  get('status').append(new Element());
  get('keyboard-layout').parentElement=new Element();
  get('keyboard-treemap').clientWidth=640;get('keyboard-treemap').clientHeight=279;
  const root=new Element('root');root.lang='en';
  document={documentElement:root,body:new Element(),getElementById:get,
    querySelector(selector){return get(selector==='header'?'header':selector==='.sidebar'?'sidebar':'main');},
    querySelectorAll(selector){return selector==='nav button'?nav:selector==='.page'?pages:[];},
    createElement(){return new Element();},createElementNS(){return new Element();}};
  const window={ipc:{postMessage(){}},presentationView:{compact:false},clickpulsePlatform:platform,clickpulseMessages:messages,clickpulseLocale:'en'};
  const context=vm.createContext({document,window,setTimeout,clearTimeout,...(legacyWebKit?{}:{ClickPulseThemes:Themes,ClickPulseDropdowns:Dropdowns,ClickPulseI18n:createI18n(messages,'en',platform),ClickPulseHistory:H,ClickPulseKeyboard:K.create(platform)})});
  if(legacyWebKit){
    // Load the real modules in this realm: injected Node functions would keep
    // Node's newer built-ins and conceal a minimum-WebKit startup failure.
    vm.runInContext('Object.hasOwn=undefined;Array.prototype.at=undefined;',context);
    for(const module of ['themes','i18n','history','keyboard','dropdown'])vm.runInContext(readFileSync('ui/'+module+'.js','utf8'),context);
  }
  vm.runInContext(readFileSync('ui/app.js','utf8'),context);
  vm.runInContext(readFileSync('ui/presentation.js','utf8'),context);
  return {window,get,root,context,navigate(page){nav.find(button=>button.dataset.page===page).fire();}};
}
function snapshot(){
  const counts={keyboard:100,mouse:20};
  const day={day:'2026-10-07',counts};
  return {now:'2026-10-07T12:00:00-07:00',today:counts,week:counts,lifetime:counts,days:[day],history_days:[day],
    windows:[{seconds:60,counts:{keyboard:2,mouse:1},approximate:false}],keyboard_stats:{physical:[{code:0,count:100}]},
    capture:{listening:true,permission_needed:false,error:null},paused:false,source_mode:false,setup_pending:false,auto_start:true,database_path:'/current/events.sqlite3'};
}
function keyCells(a){
  const visit=node=>node.dataset.key?[node]:node.children.flatMap(visit);
  return visit(a.get('keyboard-layout'));
}
test('the app starts and renders history without Safari 15.4 built-ins',()=>{
  const a=app('macos',true),s=snapshot();
  s.history_days=[{day:'2025-12-31',counts:{keyboard:3,mouse:1}},...s.history_days];
  a.window.receive(s);
  assert.equal(a.get('heatmap').children.length,2);
  assert.equal(a.get('heatmap').children[1].children[0].children[1].textContent,'Jan 1 — Oct 7');
  assert.equal(a.get('active-days').textContent,'2 active days');
  a.navigate('history');
  assert.equal(a.get('history-table').children[0].children[3].textContent,'120');
  a.navigate('keys');a.get('keyboard-view-toggle').fire();
  assert.equal(a.get('keyboard-treemap').children[0].dataset.key,'A');
  a.window.receiveView({theme:'teal',locale:'zh-CN',language_preference:'system'});
  a.navigate('overview');
  assert.equal(a.root.dataset.theme,'teal');
  assert.equal(a.get('active-days').textContent,'2 个记录日');
  assert.equal(vm.runInContext('typeof Object.hasOwn+":"+typeof Array.prototype.at',a.context),'undefined:undefined');
});
test('legacy own-property checks reject inherited themes and interpolation values',()=>{
  const a=app('macos',true);
  assert.equal(vm.runInContext('ClickPulseThemes.get("constructor").name',a.context),'blue');
  assert.equal(vm.runInContext('ClickPulseThemes.get("__proto__").name',a.context),'blue');
  assert.equal(vm.runInContext('ClickPulseI18n.t("common.year",Object.create({year:"inherited"}))',a.context),'{year}');
  assert.equal(vm.runInContext('ClickPulseI18n.t("common.year",Object.assign(Object.create(null),{year:2026,hasOwnProperty:null}))',a.context),'2026');
});
test('integrated headers drag from empty space without intercepting controls or right clicks',()=>{
  for(const platform of ['macos','windows']){
    const a=app(platform),sent=[];
    a.window.ipc.postMessage=message=>sent.push(JSON.parse(message));
    const drag=a.get('header').listeners.mousedown;
    let prevented=0;
    const event=(button,control)=>({button,target:{closest(){return control;}},preventDefault(){prevented++;}});
    drag(event(2,null));drag(event(0,{}));
    assert.equal(sent.length,0);assert.equal(prevented,0);
    drag(event(0,null));
    assert.deepEqual(sent,[{action:'drag'}]);assert.equal(prevented,1);
  }
});
test('Windows controls and close choices send distinct actions with optional persistence',()=>{
  const a=app('windows'),sent=[];a.window.ipc.postMessage=message=>sent.push(JSON.parse(message));
  for(const id of ['window-minimize','window-maximize','window-close','quit'])a.get(id).fire();
  assert.deepEqual(sent.map(message=>message.action),['minimize','maximize','close','quit']);
  sent.length=0;
  a.get('close-overlay').hidden=true;a.window.requestClose();
  assert.equal(a.get('close-overlay').hidden,false);assert.equal(a.get('main').inert,true);
  assert.equal(a.get('close-remember').checked,false);
  a.get('close-hide').fire();assert.equal(a.get('close-overlay').hidden,true);assert.equal(a.get('main').inert,false);
  assert.deepEqual(sent,[{action:'close_decision',close_action:'hide',remember:false}]);
  a.window.requestClose();a.get('close-remember').checked=true;a.get('close-quit').fire();
  assert.deepEqual(sent.at(-1),{action:'close_decision',close_action:'quit',remember:true});
  a.window.requestClose();assert.equal(a.get('close-remember').checked,false);
  const before=sent.length;a.get('close-cancel').fire();assert.equal(sent.length,before);
  a.window.receiveView({close_action:'quit',maximized:true,locale:'en'});
  assert.equal(a.get('close-action-select').value,'quit');assert.equal(a.get('window-maximize').attributes['aria-label'],'Restore window');
  a.window.receiveView({locale:'zh-CN'});assert.equal(a.get('window-maximize').attributes['aria-label'],'还原窗口');
  a.get('close-action-select').value='ask';a.get('close-action-select').fire('change');
  assert.deepEqual(sent.at(-1),{action:'close_preference',close_action:'ask'});
  assert.equal(app('macos').window.requestClose,undefined);
});
test('close prompt traps keyboard focus, cancels on Escape and ignores header control double clicks',()=>{
  const a=app('windows'),sent=[];a.window.ipc.postMessage=message=>sent.push(JSON.parse(message));
  a.get('close-overlay').hidden=true;a.window.requestClose();
  const key=a.get('close-overlay').listeners.keydown;
  let prevented=0;key({key:'Tab',preventDefault(){prevented++;}});
  a.get('close-quit').fire();assert.equal(sent.at(-1).close_action,'quit');
  a.window.requestClose();key({key:'Escape',preventDefault(){prevented++;}});
  assert.equal(a.get('close-overlay').hidden,true);assert.equal(a.get('main').inert,false);assert.equal(prevented,2);
  const doubleClick=a.get('header').listeners.dblclick;
  const before=sent.length;doubleClick({button:0,target:{closest(){return {};}}});assert.equal(sent.length,before);
  doubleClick({button:0,target:{closest(){return null;}}});assert.equal(sent.at(-1).action,'maximize');
});
test('calendar literary aside uses only selected keyboard history and follows range, live counts and language',()=>{
  const a=app(),s=snapshot();
  s.history_days=[{day:'2025-06-01',counts:{keyboard:700000,mouse:5000000}},{day:'2026-10-07',counts:{keyboard:1400000,mouse:9000000}}];
  a.window.receive(s);
  const aside=a.get('calendar-aside');
  assert.equal(aside.hidden,false);assert.match(aside.textContent,/About 3 copies of Pride and Prejudice/);
  assert.match(aside.title,/700,000/);assert.match(aside.title,/not actual written text/);
  const range=a.get('calendar-range-select');range.value='2025';range.fire('change');
  assert.match(aside.textContent,/About 1 copy of Pride and Prejudice/);
  range.value='90';range.fire('change');assert.match(aside.textContent,/About 2 copies/);
  a.window.receiveView({locale:'zh-CN',language_preference:'system'});
  assert.equal(aside.textContent,'这些按键，约够写完 2 本《傲慢与偏见》。');
  a.window.receive({...s,history_days:[s.history_days[0],{day:'2026-10-07',counts:{keyboard:2100000,mouse:9000000}}]});
  assert.equal(aside.textContent,'这些按键，约够写完 3 本《傲慢与偏见》。');
  a.navigate('settings');a.window.receive({...s,history_days:[{day:'2026-10-07',counts:{keyboard:1,mouse:9000000}}]});a.navigate('overview');
  assert.equal(aside.textContent,'故事，正从这些按键开始。');
  a.window.receive({...s,history_days:[{day:'2026-10-07',counts:{keyboard:0,mouse:9000000}}]});
  assert.equal(aside.hidden,true);assert.equal(aside.textContent,'');
});
test('view toggle replaces the total, uses current proportional counts and restores the unchanged keyboard',()=>{
  for(const platform of ['macos','windows']){
    const a=app(platform),s=snapshot();
    s.keyboard_stats={physical:[{code:0,count:80},{code:1,count:20},{code:999,count:10},{code:115,count:2}]};
    a.window.receive(s);a.navigate('keys');
    const layout=a.get('keyboard-layout'),order=keyCells(a).filter(cell=>cell.dataset.key!=='home').map(cell=>cell.dataset.key);
    a.get('keyboard-view-toggle').fire();
    const blocks=a.get('keyboard-treemap');
    assert.equal(layout.parentElement.hidden,true);assert.equal(blocks.hidden,false);
    assert.equal(a.get('keyboard-view-toggle').attributes['aria-pressed'],'true');
    assert.equal(a.get('keyboard-view-label').textContent,'Keyboard layout');
    assert.equal(blocks.children.some(cell=>cell.dataset.key==='other'),false);
    assert.equal(blocks.children.some(cell=>cell.dataset.key==='home'),platform==='windows');
    const area=id=>{const cell=blocks.children.find(cell=>cell.dataset.key===id);return parseFloat(cell.style.width)*parseFloat(cell.style.height);};
    assert.ok(Math.abs(area('A')/area('S')-4)<1e-9);
    const first=blocks.children.find(cell=>cell.dataset.key==='A'),before=first.style.values['--key-bg'];
    a.window.receive({...s,keyboard_stats:{physical:[{code:0,count:100},{code:1,count:20}]}});
    assert.match(first.attributes['aria-label'],/100/);assert.ok(Math.abs(area('A')/area('S')-5)<1e-9);
    a.window.receiveView({theme:'teal',locale:'zh-CN',language_preference:'system'});
    assert.notEqual(first.style.values['--key-bg'],before);
    assert.equal(a.get('keyboard-view-label').textContent,'键盘布局');
    a.navigate('overview');a.navigate('keys');assert.equal(blocks.hidden,false);
    a.get('keyboard-view-toggle').fire();
    assert.equal(layout.parentElement.hidden,false);assert.equal(blocks.hidden,true);
    assert.equal(a.get('keyboard-view-label').textContent,'色块图');
    assert.deepEqual(keyCells(a).filter(cell=>cell.dataset.key!=='home').map(cell=>cell.dataset.key),order);
    assert.equal(keyCells(a).find(cell=>cell.dataset.key==='A').lastElementChild.textContent,'100');
  }
  assert.doesNotMatch(readFileSync('ui/index.html','utf8'),/keyboard-total/);
});
test('an empty treemap recovers when new counts arrive and follows locale changes',()=>{
  const a=app(),s=snapshot();s.keyboard_stats={physical:[]};
  a.window.receive(s);a.navigate('keys');a.get('keyboard-view-toggle').fire();
  assert.equal(a.get('keyboard-treemap').firstElementChild.textContent,messages.en['common.empty']);
  a.window.receive({...s,keyboard_stats:{physical:[{code:49,count:1}]}});
  assert.equal(a.get('keyboard-treemap').children.length,1);
  assert.equal(a.get('keyboard-treemap').firstElementChild.dataset.key,'space');
});
test('both keyboard footers share visible used keys across hidden counts, live additions, locale and navigation',()=>{
  for(const platform of ['macos','windows']){
    const a=app(platform),s=snapshot(),count=platform==='macos'?0:2;
    const next=physical=>({...s,keyboard_stats:{physical}});
    const hidden=[{code:115,count:1000},{code:86,count:7},{code:999,count:3}];
    a.window.receive(next(hidden));a.navigate('keys');
    assert.equal(a.get('keyboard-coverage').textContent,`${count} keys used`);
    a.get('keyboard-view-toggle').fire();
    assert.equal(a.get('keyboard-coverage').textContent,`${count} keys used`);
    assert.equal(a.get('keyboard-treemap').children.filter(key=>key.dataset.key).length,count);
    a.window.receive(next([...hidden,{code:0,count:5}]));
    assert.equal(a.get('keyboard-coverage').textContent,`${count+1} keys used`);
    a.get('keyboard-view-toggle').fire();
    assert.equal(a.get('keyboard-coverage').textContent,`${count+1} keys used`);
    a.window.receiveView({theme:'teal',locale:'zh-CN',language_preference:'system'});
    assert.equal(a.get('keyboard-coverage').textContent,`使用过 ${count+1} 个按键`);
    a.navigate('overview');
    a.window.receive(next([...hidden,{code:0,count:6},{code:49,count:1}]));
    a.navigate('keys');a.get('keyboard-view-toggle').fire();
    assert.equal(a.get('keyboard-coverage').textContent,`使用过 ${count+2} 个按键`);
    a.get('keyboard-view-toggle').fire();
    assert.equal(a.get('keyboard-coverage').textContent,`使用过 ${count+2} 个按键`);
    assert.equal(keyCells(a).find(key=>key.dataset.key==='A').lastElementChild.textContent,'6');
    a.window.receive(next([]));
    assert.equal(a.get('keyboard-coverage').textContent,'使用过 0 个按键');
    a.get('keyboard-view-toggle').fire();
    assert.equal(a.get('keyboard-coverage').textContent,'使用过 0 个按键');
  }
});
test('Windows settings report native capture without offering macOS permission controls',()=>{
  const a=app('windows');a.window.receive(snapshot());a.navigate('settings');
  assert.equal(a.get('settings-permission').hidden,true);
  assert.equal(a.get('permission-description').textContent,messages.en['settings.windows_capture']);
  assert.equal(a.get('autostart-row').hidden,false);
  a.window.receive({...snapshot(),capture:{listening:false,error:'failure'}});
  assert.equal(a.get('permission-description').textContent,messages.en['status.unavailable']);
});
test('refresh only draws the selected page and skips unchanged charts',()=>{
  const a=app(),s=snapshot();a.window.receive(s);
  assert.equal(a.get('trend-chart').replacements,1);
  assert.equal(a.get('history-chart').replacements,0);
  assert.equal(a.get('keyboard-layout').replacements,0);
  a.window.receive({...s,now:'2026-10-07T12:00:02-07:00'});
  assert.equal(a.get('trend-chart').replacements,1);
  assert.equal(a.get('donut').replacements,1);
  assert.equal(a.get('windows').replacements,1);
  a.navigate('settings');
  const changed={...s,today:{keyboard:111,mouse:22},lifetime:{keyboard:111,mouse:22},history_days:[{day:'2026-10-07',counts:{keyboard:111,mouse:22}}]};
  a.window.receive(changed);
  assert.equal(a.get('trend-chart').replacements,1);
  assert.equal(a.get('db-path').textContent,'/current/events.sqlite3');
  a.navigate('history');
  assert.equal(a.get('history-table').children[0].children[3].textContent,'133');
  assert.equal(a.get('history-chart').replacements,1);
});
test('a page visited after hidden updates receives the latest keyboard counts',()=>{
  const a=app(),s=snapshot();a.window.receive(s);a.navigate('settings');
  a.window.receive({...s,keyboard_stats:{physical:[{code:0,count:123}]}});
  assert.equal(a.get('keyboard-layout').replacements,0);
  a.navigate('keys');
  assert.equal(keyCells(a).find(cell=>cell.dataset.key==='A').lastElementChild.textContent,'123');
  a.window.receive({...s,keyboard_stats:{physical:[{code:0,count:124}]}});
  assert.equal(keyCells(a).find(cell=>cell.dataset.key==='A').lastElementChild.textContent,'124');
  assert.equal(a.get('keyboard-layout').replacements,1);
});
test('extended keys refresh while unresolved history never creates a key in either language',()=>{
  const a=app(),s=snapshot();a.window.receive(s);a.navigate('keys');
  const next={...s,keyboard_stats:{physical:[{code:0,count:100},{code:115,count:4},{code:999,count:7}]}};
  a.window.receive(next);
  const cells=()=>keyCells(a);
  assert.equal(a.get('keyboard-layout').replacements,2);
  assert.equal(cells().find(cell=>cell.dataset.key==='home').lastElementChild.textContent,'4');
  assert.equal(cells().some(cell=>cell.dataset.key==='other'),false);
  assert.equal(cells().find(cell=>cell.dataset.key==='A').lastElementChild.textContent,'100');
  a.window.receiveView({locale:'zh-CN',language_preference:'system'});
  assert.equal(cells().some(cell=>cell.dataset.key==='other'),false);
  assert.equal(a.get('keyboard-layout').replacements,2);
  a.window.receive(s);
  assert.equal(cells().some(cell=>cell.dataset.key==='other'),false);
  assert.equal(a.get('keyboard-layout').replacements,3);
});
test('unknown history appearing or disappearing changes the total without rearranging the keyboard',()=>{
  const a=app(),s=snapshot();a.window.receive(s);a.navigate('keys');
  const layout=a.get('keyboard-layout'),order=()=>layout.children.map(row=>row.children.map(cell=>cell.dataset.key));
  const initial=order();
  a.window.receive({...s,keyboard_stats:{physical:[{code:0,count:100},{code:999,count:1471}]}});
  assert.deepEqual(order(),initial);
  assert.equal(layout.replacements,1);
  assert.equal(keyCells(a).find(cell=>cell.dataset.key==='A').lastElementChild.textContent,'100');
  a.window.receive(s);
  assert.deepEqual(order(),initial);
  assert.equal(layout.replacements,1);
  assert.equal(keyCells(a).find(cell=>cell.dataset.key==='A').lastElementChild.textContent,'100');
});
test('mini mode avoids full-page drawing and expansion catches up',()=>{
  const a=app(),s=snapshot();a.window.receiveView({compact:true});a.window.receive(s);
  assert.equal(a.get('mini-today').textContent,'120');
  assert.equal(a.get('trend-chart').replacements,0);
  assert.equal(a.get('heatmap').replacements,0);
  a.window.receive({...s,today:{keyboard:200,mouse:30}});
  a.window.receiveView({compact:false});
  assert.equal(a.get('today').textContent,'230');
  assert.equal(a.get('trend-chart').replacements,1);
});
test('language changes invalidate visible and deferred chart labels',()=>{
  const a=app(),s=snapshot();a.window.receive(s);a.navigate('settings');
  a.window.receiveView({locale:'zh-CN',language_preference:'system'});
  assert.equal(a.get('language-select').value,'system');
  assert.equal(a.get('trend-chart').replacements,1);
  a.navigate('overview');
  assert.equal(a.get('trend-chart').replacements,2);
  assert.equal(a.get('trend-chart').children[0].attributes['aria-label'],messages['zh-CN']['chart.aria']);
  a.window.receiveView({locale:'en',language_preference:'system'});
  assert.equal(a.get('trend-chart').replacements,3);
});
test('the calendar advances at local midnight even when counters are unchanged',()=>{
  const a=app(),s=snapshot();a.window.receive(s);
  assert.equal(a.get('heatmap').replacements,1);
  a.window.receive({...s,now:'2026-10-08T00:00:00-07:00'});
  assert.equal(a.get('heatmap').replacements,2);
  assert.match(a.get('calendar-range').textContent,/Oct 8, 2026/);
});
test('theme switching recolors current and deferred charts while preserving keyboard counts and rows',()=>{
  const a=app(),s=snapshot();a.window.receive(s);a.navigate('keys');
  const layout=a.get('keyboard-layout'),first=keyCells(a).find(cell=>cell.dataset.key==='A');
  const before=first.style.values['--key-bg'];
  a.window.receiveView({theme:'teal'});
  assert.equal(a.root.dataset.theme,'teal');assert.equal(a.get('theme-select').value,'teal');
  assert.notEqual(first.style.values['--key-bg'],before);assert.equal(layout.replacements,1);
  assert.equal(keyCells(a).find(cell=>cell.dataset.key==='A').lastElementChild.textContent,'100');
  a.navigate('overview');
  assert.equal(a.get('trend-chart').children[0].children.find(node=>node.attributes.fill).attributes.fill,Themes.get('teal').accent);
  a.window.receiveView({theme:'violet'});
  assert.equal(a.get('trend-chart').children[0].children.find(node=>node.attributes.fill).attributes.fill,Themes.get('violet').accent);
  assert.equal(a.get('today').textContent,'120');
});
test('a failed snapshot updates mini status, preserves last counts and clears after recovery',()=>{
  const a=app(),s=snapshot();a.window.receiveView({compact:true});a.window.receive(s);
  a.window.failure(messages.en['error.stats']);
  assert.equal(a.get('mini-status').textContent,messages.en['status.recording_error']);
  assert.equal(a.get('mini-today').textContent,'120');
  a.window.receiveView({compact:false});
  assert.equal(a.get('error-banner').hidden,false);
  a.window.receive({...s,today:{keyboard:102,mouse:20}});
  assert.equal(a.get('error-banner').hidden,true);
  assert.equal(a.get('mini-status').textContent,messages.en['status.recording']);
  assert.equal(a.get('today').textContent,'122');
});
test('grouped arrows retain unique per-key counts through refresh, theme and language changes on both platforms',()=>{
  for(const platform of ['macos','windows']){
    const a=app(platform),s=snapshot();
    s.keyboard_stats={physical:[{code:0,count:55},{code:123,count:11},{code:124,count:22},{code:125,count:33},{code:126,count:44}]};
    a.window.receive(s);a.navigate('keys');
    const layout=a.get('keyboard-layout'),cluster=layout.children[5].children.find(node=>node.className==='keyboard-arrow-cluster');
    assert.deepEqual(cluster.children.map(cell=>cell.dataset.key),['up','left','down','right']);
    assert.equal(new Set(keyCells(a).map(cell=>cell.dataset.key)).size,keyCells(a).length);
    const counts=()=>Object.fromEntries(cluster.children.map(cell=>[cell.dataset.key,cell.lastElementChild.textContent]));
    assert.deepEqual(counts(),{up:'44',left:'11',down:'33',right:'22'});assert.equal(keyCells(a).find(cell=>cell.dataset.key==='A').lastElementChild.textContent,'55');
    a.window.receive({...s,keyboard_stats:{physical:[...s.keyboard_stats.physical,{code:126,count:1}]}});
    a.window.receiveView({theme:'teal',locale:'zh-CN',language_preference:'system'});
    assert.equal(layout.replacements,1);assert.equal(layout.children[5].children.at(-1),cluster);
    assert.deepEqual(counts(),{up:'45',left:'11',down:'33',right:'22'});assert.equal(keyCells(a).find(cell=>cell.dataset.key==='A').lastElementChild.textContent,'55');
    assert.match(cluster.children[0].attributes['aria-label'],/45/);
  }
});

test('trend labels render as unscaled text with grouped integers and localized dates',()=>{
  const a=app(),s=snapshot();
  s.days=H.calendarDays([{day:'2026-10-07',counts:{keyboard:17000,mouse:5100}}],'2026-09-25','2026-10-08');
  a.window.receive(s);
  const chart=a.get('trend-chart'),[plot,yAxis,xAxis]=chart.children;
  assert.equal(plot.attributes.class,'trend-plot');
  assert.deepEqual(yAxis.children.map(label=>label.textContent),['25,000','20,000','15,000','10,000','5,000','0']);
  assert.deepEqual(xAxis.children.map(label=>label.children[0].textContent),['Sep 25','Sep 28','Oct 2','Oct 5','Oct 8']);
  assert.equal(plot.children.some(node=>node.attributes['text-anchor']),false);
  assert.ok(plot.children.filter(node=>node.attributes.fill).every(bar=>Number(bar.attributes.y)>=0));
  a.window.receiveView({locale:'zh-CN',language_preference:'system'});
  assert.equal(chart.children[2].children[0].children[0].textContent,'9月25日');
  assert.match(a.get('calendar-range').textContent,/2026年10月7日/);
});

test('weekly dates begin at the first recorded day and multi-year axes retain the year',()=>{
  const a=app(),s=snapshot();
  s.now='2026-03-08T12:00:00-07:00';
  s.history_days=[{day:'2025-12-05',counts:{keyboard:100,mouse:20}}];
  a.window.receive(s);a.navigate('history');
  const labels=a.get('history-chart').children[2].children;
  assert.equal(labels[0].children[0].textContent,'Dec 5');assert.equal(labels[0].children[1].textContent,'2025');
  assert.equal(labels.at(-1).children[1].textContent,'2026');
  assert.equal(a.get('history-table').children[0].children[0].textContent,'Dec 5, 2025');
  a.window.receive({...s,history_days:[{day:'2024-12-05',counts:{keyboard:100,mouse:20}}]});
  const months=a.get('history-chart').children[2].children;
  assert.equal(months[0].children[0].textContent,'Dec');assert.equal(months[0].children[1].textContent,'2024');
  assert.equal(months.at(-1).children[0].textContent,'Mar');assert.equal(months.at(-1).children[1].textContent,'2026');
});
