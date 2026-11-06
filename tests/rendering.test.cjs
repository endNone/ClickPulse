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
const Sport=require('../ui/sport.js');

// Small DOM adapter to exercise the actual App event handlers and render flow.
class Element {
  constructor(id=''){
    this.id=id;this.dataset={};this.children=[];this.listeners={};this.attributes={};this.value='all';this.checked=false;this.replacements=0;
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
  scrollIntoView(options){this.scrolled=options;}
  focus(){if(this.ownerDocument)this.ownerDocument.activeElement=this;}
}
function app(platform='macos',legacyWebKit=false,reducedMotion=false){
  let document,now=0;
  const intervals=new Map();let timerId=0;
  const nodes=new Map(),get=id=>{
    if(!nodes.has(id))nodes.set(id,new Element(id));
    const node=nodes.get(id);node.ownerDocument=document;return node;
  };
  const nav=['overview','history','keys','settings'].map(id=>{const button=new Element();button.dataset.page=id;return button;});
  const pages=nav.map(button=>get(button.dataset.page));
  const settingsLinks=['personalization-settings','handoff-settings','capture-settings','storage-settings','app-settings'].map(id=>{const b=new Element();b.dataset.settingsSection=id;return b;});
  for(const id of ['handoff-settings','capture-settings'])get(id).hidden=true;
  const ranges=[14,30,90].map(days=>{const button=new Element();button.dataset.range=String(days);return button;});
  get('status').append(new Element());
  get('keyboard-layout').parentElement=new Element();
  get('keyboard-treemap').clientWidth=640;get('keyboard-treemap').clientHeight=279;
  const root=new Element('root');root.lang='en';
  document={documentElement:root,body:new Element(),getElementById:id=>['export','history-export','settings-pause','recording-description','quit','sport-open-mini','refresh-time','calendar-description'].includes(id)?null:get(id),
    querySelector(selector){return get(selector==='header'?'header':selector==='.sidebar'?'sidebar':'main');},
    querySelectorAll(selector){return selector==='nav button[data-page]'?nav:selector==='[data-settings-section]'?settingsLinks:selector==='.page'?pages:selector==='[data-range]'?ranges:[];},
    createElement(){const node=new Element();node.ownerDocument=document;return node;},createElementNS(){const node=new Element();node.ownerDocument=document;return node;}};
  const window={ipc:{postMessage(){}},presentationView:{compact:false},clickpulsePlatform:platform,clickpulseMessages:messages,clickpulseLocale:'en'};
  Object.assign(window,{matchMedia:()=>({matches:reducedMotion,addEventListener(){}}),performance:{now:()=>now},setInterval(handler,ms){const id=++timerId;intervals.set(id,{handler,ms});return id;},clearInterval(id){intervals.delete(id);}});
  const context=vm.createContext({document,window,setTimeout,clearTimeout,...(legacyWebKit?{}:{ClickPulseSport:Sport,ClickPulseThemes:Themes,ClickPulseDropdowns:Dropdowns,ClickPulseI18n:createI18n(messages,'en',platform),ClickPulseHistory:H,ClickPulseKeyboard:K.create(platform)})});
  if(legacyWebKit){
    // Load the real modules in this realm: injected Node functions would keep
    // Node's newer built-ins and conceal a minimum-WebKit startup failure.
    vm.runInContext('Object.hasOwn=undefined;Array.prototype.at=undefined;',context);
    for(const module of ['themes','i18n','history','keyboard','dropdown','sport'])vm.runInContext(readFileSync('ui/'+module+'.js','utf8'),context);
  }
  vm.runInContext(readFileSync('ui/app.js','utf8'),context);
  vm.runInContext(readFileSync('ui/presentation.js','utf8'),context);
  return {window,get,root,context,intervals,settingsLinks,jumpTo(id){settingsLinks.find(b=>b.dataset.settingsSection===id).fire();},setTime(value){now=value;},selectRange(days){ranges.find(button=>Number(button.dataset.range)===days).fire();},navigate(page){nav.find(button=>button.dataset.page===page).fire();}};
}

test('Sport polls only in enabled visible Mini windows and discards historical or resume bursts',()=>{
  const a=app(),sent=[];a.window.ipc.postMessage=message=>sent.push(JSON.parse(message));
  a.window.receive(snapshot());assert.equal(a.intervals.size,0);
  a.window.receiveView({sport_enabled:true,sport_visible:true,compact:false});assert.equal(a.intervals.size,0);
  a.window.receiveView({compact:true});assert.equal(a.intervals.size,1);assert.equal([...a.intervals.values()][0].ms,250);
  assert.equal(sent[sent.length-1].action,'sport_sample');
  a.window.receiveSport({keyboard:0,mouse:50000,available:true});
  for(let i=1;i<=4;i++){a.setTime(i*250);a.window.receiveSport({keyboard:0,mouse:50000+i*3,available:true});}
  assert.equal(a.get('mini-sport-grade').textContent,'SS');assert.equal(a.get('mini-status').hidden,true);
  a.window.receiveView({compact:false});assert.equal(a.intervals.size,0);
  a.setTime(6000);a.window.receiveView({compact:true});a.window.receiveSport({keyboard:0,mouse:90000,available:true});
  assert.equal(a.get('mini-sport-cps').textContent,'0.0');
  a.window.receiveView({visible:false});assert.equal(a.intervals.size,0);
  a.window.receiveView({visible:true});assert.equal(a.intervals.size,1);
  a.window.receiveView({sport_visible:false});assert.equal(a.intervals.size,0);
  assert.equal(a.get('mini-sport').hidden,true);assert.equal(a.get('mini-status').hidden,false);
});

test('reduced-motion Mini still shows SS speed without enabling animation',()=>{
  const a=app('macos',false,true);a.window.receiveView({compact:true,sport_enabled:true,sport_visible:true});
  a.window.receiveSport({keyboard:0,mouse:100,available:true});
  for(let i=1;i<=4;i++){a.setTime(i*250);a.window.receiveSport({keyboard:0,mouse:100+i*3,available:true});}
  assert.equal(a.get('mini-sport-grade').textContent,'SS');
  assert.equal(a.get('mini-widget').dataset.moving,'false');
});

test('Mini Sport controls retain Warp for old preferences and clear energy on pause or error',()=>{
  const a=app(),sent=[];a.window.ipc.postMessage=message=>sent.push(JSON.parse(message));
  a.get('mini-sport').fire();assert.deepEqual(sent.pop(),{action:'sport',enabled:true});
  a.window.receiveView({compact:true,sport_enabled:true,sport_visible:true,sport_skin:'beat'});
  assert.equal(a.get('mini-widget').dataset.sportSkin,'warp');
  a.window.receiveSport({keyboard:0,mouse:0,available:true});a.setTime(250);a.window.receiveSport({keyboard:0,mouse:2,available:true});
  a.window.receiveSport({keyboard:0,mouse:4,available:true,paused:true});assert.equal(a.get('mini-sport-cps').textContent,'0.0');
  a.window.receiveView({locale:'zh-CN',language_preference:'zh-CN'});
  assert.equal(a.get('mini-sport').attributes['aria-label'],messages['zh-CN']['sport.disable']);
  a.get('sport-visible').checked=false;a.get('sport-visible').fire('change');
  assert.deepEqual(sent.pop(),{action:'sport_visible',visible:false});
  a.window.receiveView({sport_visible:false,sport_enabled:false});assert.equal(a.get('mini-sport-readout').hidden,true);
  assert.equal(a.get('mini-widget').dataset.sportSkin,'warp');
  a.window.failure('offline');assert.equal(a.get('mini-sport-cps').textContent,'0.0');
});

test('all eight languages update Mini Sport, access names and deferred settings without resetting its state',()=>{
  for(const platform of ['macos','windows'])for(const locale of ['en','zh-CN','zh-TW','ja','ko','fr','de','es']){
    const a=app(platform);a.window.receive(snapshot());
    a.window.receiveView({compact:true,sport_enabled:true,sport_visible:true,sport_skin:'beat',locale,language_preference:locale});
    a.window.receiveSport({keyboard:100,mouse:20,available:true});a.setTime(250);a.window.receiveSport({keyboard:102,mouse:20,available:true});
    assert.equal(a.root.lang,locale);
    assert.equal(a.get('mini-sport').attributes['aria-label'],messages[locale]['sport.disable']);
    assert.equal(a.get('mini-sport-caption').textContent,'Sport · '+messages[locale]['sport.rank_b']);
    assert.equal(a.get('mini-sport-readout').attributes['aria-label'],createI18n(messages,locale).t('sport.mini_readout',{rank:'B',speed:createI18n(messages,locale).decimal(2,1)}));
    a.window.receiveView({compact:false});a.navigate('settings');
    assert.equal(a.get('mini-widget').dataset.sportSkin,'warp');
    assert.equal(a.get('mini-sport').attributes['aria-pressed'],'true');
    assert.equal(a.get('page-title').textContent,messages[locale]['page.settings']);
  }
});
function snapshot(){
  const counts={keyboard:100,mouse:20};
  const day={day:'2026-10-07',counts};
  return {now:'2026-10-07T12:00:00-07:00',today:counts,week:counts,lifetime:counts,days:[day],history_days:[day],
    windows:[{seconds:60,counts:{keyboard:2,mouse:1},approximate:false}],keyboard_stats:{physical:[{code:0,count:100}]},
    capture:{listening:true,permission_needed:false,error:null},paused:false,source_mode:false,setup_pending:false,auto_start:true,auto_start_available:true,database_mode:'default',database_path:'/current/events.sqlite3'};
}
function keyCells(a){
  const visit=node=>node.dataset.key?[node]:node.children.flatMap(visit);
  return visit(a.get('keyboard-layout'));
}

test('Pause acknowledges immediately while statistics are pending and stale replies cannot undo it',()=>{
  for(const platform of ['macos','windows'])for(const compact of [false,true])for(const source_mode of [false,true]){
    const a=app(platform),s={...snapshot(),source_mode};
    a.window.receive(s);a.window.receiveView({compact,sport_enabled:true,sport_visible:true});
    a.window.receiveSport({keyboard:0,mouse:0,available:true});a.setTime(250);
    a.window.receiveSport({keyboard:10,mouse:0,available:true});
    a.window.receivePaused(true);
    assert.equal(a.get('pause').textContent,messages.en[source_mode?'action.resume_refresh':'action.resume']);
    assert.equal(a.get('mini-pause').attributes['aria-label'],messages.en[source_mode?'action.resume_refresh':'action.resume']);
    assert.equal(a.get('status').lastElementChild.textContent,messages.en['status.paused']);
    assert.equal(a.get('mini-sport-cps').textContent,'0.0');
    a.window.receive(s);a.window.receiveSport({keyboard:20,mouse:0,available:true,paused:false});
    assert.equal(a.get('mini-pause-label').textContent,messages.en['mini.resume']);
    assert.equal(a.get('mini-sport-cps').textContent,'0.0');
    a.window.receivePaused(false);a.window.receive({...s,paused:true});
    assert.equal(a.get('mini-pause-label').textContent,messages.en['mini.pause']);
    assert.equal(a.get('pause').textContent,messages.en[source_mode?'action.pause_refresh':'action.pause']);
  }
  const a=app('windows');a.window.receivePaused(true);
  assert.equal(a.get('pause').textContent,messages.en['action.resume']);
  assert.equal(a.get('mini-pause-label').textContent,messages.en['mini.resume']);
  a.window.receive(snapshot());
  assert.equal(a.get('pause').textContent,messages.en['action.resume']);
});
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
  for(const id of ['window-minimize','window-maximize','window-close'])a.get(id).fire();
  assert.deepEqual(sent.map(message=>message.action),['minimize','maximize','close']);
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
    assert.equal(blocks.children.some(cell=>cell.dataset.key==='home'),false);
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
    const a=app(platform),s=snapshot(),count=0;
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
  assert.equal(a.get('permission-row').hidden,true);assert.equal(a.get('capture-settings').hidden,true);
  assert.equal(a.get('permission-description').textContent,messages.en['settings.windows_capture']);
  assert.equal(a.get('autostart-row').hidden,false);
  a.window.receive({...snapshot(),capture:{listening:false,error:'failure'}});
  assert.equal(a.get('permission-description').textContent,messages.en['status.unavailable']);
});

test('settings distinguish storage modes and only offer login startup for the default database',()=>{
  const a=app('windows');a.navigate('settings');
  for(const mode of ['default','custom','source','demo']){
    const s={...snapshot(),database_mode:mode,auto_start_available:mode==='default',source_mode:mode==='source',demo:mode==='demo'};
    a.window.receive(s);
    assert.equal(a.get('autostart-row').hidden,mode!=='default');
    assert.equal(a.get('db-path').textContent,mode==='demo'?messages.en['demo.storage']:s.database_path);
    assert.equal(a.get('db-description').textContent,messages.en[mode==='default'?'storage.directory_description':'settings.storage_'+mode]);
    assert.equal(a.get('storage-directory').hidden,mode!=='default');
    assert.equal(a.get('import-history-row').hidden,mode!=='default');
    a.window.receiveView({locale:'zh-CN',language_preference:'system'});
    assert.equal(a.get('db-description').textContent,messages['zh-CN'][mode==='default'?'storage.directory_description':'settings.storage_'+mode]);
    a.window.receiveView({locale:'en',language_preference:'system'});
  }
});

test('selected main and Mini statistics keep updating without losing copy selection',()=>{
  for(const platform of ['macos','windows'])for(const id of ['today','mini-today'])for(const backwards of [false,true]){
    const a=app(platform),s={...snapshot(),today:{keyboard:8,mouse:1}};a.window.receive(s);
    const node=a.get(id);let value=node.textContent;
    const text={nodeType:3,get data(){return value;},set data(next){value=next;selection.anchorOffset=selection.focusOffset=0;}};
    Object.defineProperty(node,'textContent',{get(){return text.data;},set(next){text.data=next;}});
    node.firstChild=text;node.childNodes=[text];
    const selection={isCollapsed:false,rangeCount:1,anchorNode:text,focusNode:text,anchorOffset:backwards?1:0,focusOffset:backwards?0:1,
      getRangeAt:()=>({intersectsNode:target=>target===node}),
      setBaseAndExtent(anchor,from,focus,to){this.anchorNode=anchor;this.anchorOffset=from;this.focusNode=focus;this.focusOffset=to;}};
    a.window.getSelection=()=>selection;
    for(const count of [10,12345,7]){
      a.window.receive({...s,today:{keyboard:count-1,mouse:1}});
      const expected=count.toLocaleString('en');
      assert.equal(node.textContent,expected,'selected count must match the latest snapshot');
      assert.equal(selection.anchorNode,text);assert.equal(selection.focusNode,text);
      assert.equal(selection.anchorOffset,backwards?expected.length:0);
      assert.equal(selection.focusOffset,backwards?0:expected.length);
      assert.equal(text.data.slice(Math.min(selection.anchorOffset,selection.focusOffset),Math.max(selection.anchorOffset,selection.focusOffset)),expected,'copy selection covers the current count');
    }
  }
});

test('data processing prevents another mutation while page navigation remains available',()=>{
  const a=app();a.window.receive(snapshot());a.window.receiveDataBusy(true);
  assert.equal(a.get('data-operation-status').hidden,false);
  for(const id of ['storage-directory','import-history','reveal'])assert.equal(a.get(id).disabled,true);
  a.navigate('history');assert.equal(a.get('page-title').textContent,messages.en['page.history']);
  a.navigate('settings');assert.equal(a.get('reveal').disabled,true);
  a.window.receiveDataBusy(false);assert.equal(a.get('data-operation-status').hidden,true);
  for(const id of ['storage-directory','import-history','reveal'])assert.equal(a.get(id).disabled,false);
});

test('history processing stays visible across language switches and blocks overlapping work',()=>{
  const a=app(),s={...snapshot(),source_mode:true,setup_pending:true,database_mode:'source'};a.window.receive(s);
  a.window.receiveDataBusy(true);
  for(const locale of Object.keys(messages)){
    a.window.setInterfaceLanguage(locale,locale);
    assert.equal(a.get('data-operation-message').textContent,messages[locale]['storage.processing']);
    a.navigate('settings');assert.equal(a.get('takeover').disabled,true);
    a.navigate('history');assert.equal(a.get('import-history').disabled,true);
    assert.equal(a.get('data-operation-status').hidden,false);
  }
  a.window.receiveDataBusy(false);
  assert.equal(a.get('data-operation-status').hidden,true);
  assert.equal(a.get('takeover').disabled,false);
  a.window.receive({...s,capture:{listening:false,permission_needed:true,error:null}});
  a.window.receiveDataBusy(true);a.window.receiveDataBusy(false);
  assert.equal(a.get('takeover').disabled,true);
});

test('the Windows heatmap ends at the space row even after extended-key input arrives',()=>{
  const a=app('windows'),s=snapshot();a.navigate('keys');
  for(const physical of [[],[{code:115,count:100},{code:86,count:40},{code:49,count:2}]]){
    a.window.receive({...s,keyboard_stats:{physical}});
    const rows=a.get('keyboard-layout').children.filter(row=>!row.className.includes('supplemental-row'));
    assert.equal(rows.length,6);
    assert.ok(rows[5].children.some(cell=>cell.dataset.key==='space'));
    assert.ok(!rows.flatMap(row=>row.children).some(cell=>['home','num4'].includes(cell.dataset.key)));
    assert.equal(a.get('keyboard-coverage').textContent,physical.length?'1 keys used':'0 keys used');
  }
  assert.match(readFileSync('ui/style.css','utf8'),/\.keyboard-row\.supplemental-row\s*\{display:none\}/);
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

function hourlyFixture(){
  const start=Date.parse('2026-11-02T00:37:15-08:00'),end=start+86400000,hour=3600000;
  const hours=[];
  for(let boundary=Math.floor(start/hour)*hour;boundary<end;boundary+=hour){
    hours.push({start_ms:Math.max(start,boundary),end_ms:Math.min(end,boundary+hour),counts:{keyboard:0,mouse:0},approximate:false});
  }
  hours[0].counts={keyboard:7,mouse:2};hours[24].counts={keyboard:3,mouse:1};
  return {...snapshot(),now:new Date(end).toISOString(),hourly:hours};
}

test('hourly chart retains partial edges, zero hours and keyboard/mouse counts in all eight languages',()=>{
  for(const locale of Object.keys(messages)){
    const a=app(),s=hourlyFixture(),i=createI18n(messages,locale);
    a.window.receiveView({locale,language_preference:locale});a.window.receive(s);
    const chart=a.get('hourly-chart'),plot=chart.children[0];
    assert.equal(plot.children.length,25);assert.equal(a.get('hourly-total').textContent,i.number(13));
    assert.equal(plot.children[1].children[0].style.height,'0%');
    assert.ok(parseFloat(plot.children[0].children[0].style.height)>parseFloat(plot.children[0].children[1].style.height));
    assert.match(plot.children[0].attributes['aria-label'],new RegExp(messages[locale]['common.keyboard']));
    plot.children[0].fire('mouseenter');
    assert.equal(a.get('hourly-detail').textContent,plot.children[0].attributes['aria-label']);
    assert.ok(a.get('hourly-detail').textContent.includes(i.number(7)));
    assert.equal(a.get('hourly-detail').textContent.includes('chart.'),false);
    const first=chart.children[2].children[0].children[0].textContent;
    assert.equal(first,i.date(s.hourly[0].start_ms,{hour:'2-digit',minute:'2-digit',hourCycle:'h23'}));
    assert.equal(chart.children[3].hidden,true);
    s.hourly.forEach(h=>h.counts={keyboard:0,mouse:0});a.window.receive(s);
    assert.equal(chart.children[3].hidden,false);assert.equal(chart.children[3].textContent,messages[locale]['common.empty']);
  }
});

test('hourly live refresh preserves focused bars, updates details and marks approximate imported data',()=>{
  const a=app(),s=hourlyFixture();a.window.receive(s);
  const chart=a.get('hourly-chart'),plot=chart.children[0],bar=plot.children[24],replacements=chart.replacements;
  bar.focus();bar.fire('focus');
  s.hourly[24].counts.mouse=6;s.hourly[24].end_ms+=2000;s.hourly[0].start_ms+=2000;s.hourly[0].approximate=true;
  a.window.receive(s);
  assert.equal(chart.replacements,replacements);assert.equal(chart.children[0],plot);
  assert.equal(a.context.document.activeElement,bar);
  assert.equal(a.get('hourly-detail').textContent,bar.attributes['aria-label']);
  assert.match(a.get('hourly-total').textContent,/^≈ /);
  assert.match(plot.children[0].attributes['aria-label'],/≈ /);
  bar.listeners.keydown({key:'ArrowLeft',preventDefault(){}});
  assert.equal(a.context.document.activeElement,plot.children[23]);
  bar.listeners.keydown({key:'Home',preventDefault(){}});
  assert.equal(a.context.document.activeElement,plot.children[0]);
  a.window.receiveView({compact:true});s.hourly[0].counts.keyboard=50;a.window.receive(s);
  assert.equal(chart.replacements,replacements);
  a.window.receiveView({compact:false});
  assert.match(plot.children[0].attributes['aria-label'],/50/);
});

test('hourly local times distinguish repeated DST hours and span the spring clock jump',()=>{
  const previous=process.env.TZ;
  try{
    process.env.TZ='America/Los_Angeles';
    for(const [start,repeat] of [['2026-11-01T00:00:00-07:00',true],['2026-03-08T00:00:00-08:00',false]]){
      const a=app(),s=snapshot(),base=Date.parse(start);
      s.hourly=Array.from({length:24},(_,index)=>({start_ms:base+index*3600000,end_ms:base+(index+1)*3600000,counts:{keyboard:1,mouse:0},approximate:false}));
      a.window.receive(s);
      const bars=a.get('hourly-chart').children[0].children;
      assert.equal(bars.length,24);
      if(repeat){
        assert.match(bars[1].attributes['aria-label'],/01:00/);assert.match(bars[2].attributes['aria-label'],/01:00/);
        assert.notEqual(bars[1].attributes['aria-label'],bars[2].attributes['aria-label']);
        assert.match(bars[1].attributes['aria-label'],/PDT/);assert.match(bars[2].attributes['aria-label'],/PST/);
      }else{
        assert.match(bars[1].attributes['aria-label'],/01:00.*03:00/);
        assert.match(bars[2].attributes['aria-label'],/03:00.*04:00/);
      }
    }
  }finally{if(previous===undefined)delete process.env.TZ;else process.env.TZ=previous;}
});


test('Sport is hidden and inactive by default until its control is explicitly enabled',()=>{
  const a=app();a.window.receive(snapshot());
  assert.equal(a.window.presentationView.sport_enabled,false);
  assert.equal(a.window.presentationView.sport_visible,false);
  assert.equal(a.get('sport-visible').checked,false);
  a.window.receiveView({compact:true});assert.equal(a.get('mini-sport').hidden,true);assert.equal(a.intervals.size,0);
  a.window.receiveView({sport_visible:true});assert.equal(a.get('mini-sport').hidden,false);assert.equal(a.intervals.size,0);
  a.window.receiveView({sport_enabled:true});assert.equal(a.intervals.size,1);
});

test('the thirty-day trend range uses thirty dates and survives refresh and language changes',()=>{
  const a=app(),s=snapshot();
  s.days=Array.from({length:90},(_,index)=>({day:H.addDays('2026-07-10',index),counts:{keyboard:index+1,mouse:1}}));
  a.window.receive(s);a.selectRange(30);
  const bars=()=>a.get('trend-chart').children[0].children.filter(node=>node.attributes.fill);
  assert.equal(bars().length,60);
  assert.ok(bars()[0].children[0].textContent.includes('61'));
  s.days[89].counts.keyboard=999;a.window.receive(s);assert.equal(bars().length,60);
  a.window.receiveView({locale:'zh-CN',language_preference:'zh-CN'});assert.equal(bars().length,60);
  assert.ok(bars()[59].children[0].textContent.includes('999'));
  a.selectRange(14);assert.equal(bars().length,28);
  a.selectRange(90);assert.equal(bars().length,180);
});


test('Settings section links expand only on Settings and jump without navigating away',()=>{
  for(const platform of ['macos','windows']){
    const a=app(platform);a.window.receive(snapshot());a.navigate('settings');
    assert.equal(a.get('settings-subnav').hidden,false);
    assert.equal(a.get('settings-nav').attributes['aria-expanded'],'true');
    assert.equal(a.settingsLinks.find(b=>b.dataset.settingsSection==='capture-settings').hidden,platform==='windows');
    assert.equal(a.settingsLinks.find(b=>b.dataset.settingsSection==='handoff-settings').hidden,true);
    a.jumpTo('storage-settings');
    assert.equal(a.get('page-title').textContent,messages.en['page.settings']);
    assert.equal(a.get('storage-settings').scrolled.block,'start');
    assert.equal(a.settingsLinks.find(b=>b.dataset.settingsSection==='storage-settings').attributes['aria-current'],'location');
    a.window.receiveView({locale:'ja',language_preference:'ja'});a.window.receive(snapshot());
    assert.equal(a.settingsLinks.find(b=>b.dataset.settingsSection==='storage-settings').attributes['aria-current'],'location');
    a.navigate('overview');assert.equal(a.get('settings-subnav').hidden,true);
    assert.equal(a.get('settings-nav').attributes['aria-expanded'],'false');
  }
});

test('Settings links track conditional groups and respect reduced motion',()=>{
  const a=app('macos',false,true);const s=snapshot();s.setup_pending=true;
  a.window.receive(s);a.navigate('settings');
  assert.equal(a.settingsLinks.find(b=>b.dataset.settingsSection==='handoff-settings').hidden,false);
  a.jumpTo('handoff-settings');assert.equal(a.get('handoff-settings').scrolled.behavior,'auto');
  s.setup_pending=false;s.source_mode=true;a.window.receive(s);
  assert.equal(a.settingsLinks.find(b=>b.dataset.settingsSection==='handoff-settings').hidden,true);
  assert.equal(a.settingsLinks.find(b=>b.dataset.settingsSection==='capture-settings').hidden,true);
  assert.equal(a.settingsLinks[0].attributes['aria-current'],'location');
});

test('time zone choices persist across locale changes and stale-zone snapshots are ignored',()=>{
  const a=app(),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
  const s=snapshot();s.timezone='system';s.now='2026-11-04T00:30:00Z';a.window.receive(s);
  a.window.receiveView({timezone:'Asia/Shanghai',timezones:['UTC','Asia/Shanghai','not/a-zone']});
  assert.deepEqual(a.get('timezone-select').children.map(o=>o.value),['system','UTC','Asia/Shanghai']);
  const option=a.get('timezone-select').children[1];
  a.window.receiveView({pinned:false,timezones:['UTC','Asia/Shanghai','not/a-zone']});
  assert.equal(a.get('timezone-select').children[1],option);
  const zoned={...s,timezone:'Asia/Shanghai',now:'2026-11-04T08:30:00+08:00',today:{keyboard:17,mouse:3}};
  a.window.receive(zoned);assert.equal(a.get('today').textContent,'20');
  a.window.receive({...s,today:{keyboard:9999,mouse:0}});assert.equal(a.get('today').textContent,'20');
  a.window.receiveView({locale:'fr',language_preference:'fr'});
  assert.equal(a.get('timezone-select').value,'Asia/Shanghai');
  a.get('timezone-select').value='system';a.get('timezone-select').fire('change');
  assert.deepEqual(sent.pop(),{action:'timezone',timezone:'system'});
  a.window.receiveView({timezone:'system'});a.window.receive(s);
  assert.equal(a.get('timezone-select').value,'system');
});

test('updates require an explicit user click for both checking and installing',()=>{
  for(const platform of ['macos','windows']){
    const a=app(platform),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
    a.window.receiveUpdateState({status:'idle',current:'0.2.0'});assert.equal(sent.length,0);
    a.get('check-updates').fire();assert.deepEqual(sent.pop(),{action:'check_updates'});
    a.window.receiveUpdateState({status:'checking'});a.get('check-updates').fire();assert.equal(sent.length,0);
    a.window.receiveUpdateState({status:'available',version:'0.2.1',can_install:true});assert.equal(sent.length,0);
    assert.equal(a.get('check-updates').textContent,messages.en['update.install']);
    assert.equal(a.get('update-details').hidden,false);
    a.get('check-updates').fire();assert.deepEqual(sent.pop(),{action:'install_update'});
  }
});
test('updater progress and data operations exclude concurrent destructive actions',()=>{
  const a=app();a.window.receive(snapshot());
  for(const status of ['downloading','verifying','preparing','installing']){
    a.window.receiveUpdateState({status,current:'0.2.0',received:50,total:100});
    assert.equal(a.get('check-updates').disabled,true);
    assert.equal(a.get('storage-directory').disabled,true);assert.equal(a.get('import-history').disabled,true);
    assert.equal(a.get('check-updates').attributes['aria-busy'],'true');
  }
  a.window.receiveUpdateState({status:'available',version:'0.2.1',can_install:true});a.window.receiveDataBusy(true);
  assert.equal(a.get('check-updates').disabled,true);
  const sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));a.get('check-updates').fire();assert.equal(sent.length,0);
  a.window.receiveDataBusy(false);assert.equal(a.get('check-updates').disabled,false);
});
test('unsupported installations open release downloads and errors can be retried',()=>{
  const a=app(),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
  a.window.receiveUpdateState({status:'available',version:'0.2.1',can_install:false});
  assert.equal(a.get('check-updates').textContent,messages.en['update.download']);
  a.get('check-updates').fire();assert.deepEqual(sent.pop(),{action:'update_release'});
  a.window.receiveUpdateState({status:'error'});a.get('check-updates').fire();assert.deepEqual(sent.pop(),{action:'check_updates'});
  a.window.receiveUpdateState({status:'current'});assert.equal(a.get('update-status').textContent,messages.en['update.current']);
  assert.equal(a.get('update-details').hidden,true);
});
test('all updater states follow all eight languages without restarting a transfer',()=>{
  for(const platform of ['macos','windows'])for(const locale of Object.keys(messages)){
    const a=app(platform),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
    a.window.receiveUpdateState({status:'downloading',current:'0.2.0',version:'0.2.1',received:51,total:100});
    a.window.receiveView({locale,language_preference:locale});
    assert.equal(a.get('update-status').textContent,createI18n(messages,locale).t('update.downloading',{percent:51}));
    assert.equal(a.get('check-updates').textContent,messages[locale]['update.updating']);
    assert.equal(a.get('check-updates').disabled,true);assert.equal(sent.length,0);
  }
});
