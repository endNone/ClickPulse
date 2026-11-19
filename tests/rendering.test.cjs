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
  closest(){return null;}
  contains(node){return this===node||this.children.some(child=>child.contains?.(node));}
}
function app(platform='macos',legacyWebKit=false,reducedMotion=false,random=()=>0){
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
  get('calendar-range-select').value='365';
  get('keyboard-layout').parentElement=new Element();
  get('keyboard-treemap').clientWidth=640;get('keyboard-treemap').clientHeight=279;
  const root=new Element('root');root.lang='en';
  document={listeners:{},addEventListener(event,handler){const previous=this.listeners[event];this.listeners[event]=previous?event=>{previous(event);handler(event);}:handler;},documentElement:root,body:new Element(),getElementById:id=>['page-description','date','status','today-scope','legacy-note','export','history-export','settings-pause','recording-description','quit','sport-open-mini','refresh-time','calendar-description'].includes(id)?null:get(id),
    querySelector(selector){return get(selector==='header'?'header':selector==='.sidebar'?'sidebar':'main');},
    querySelectorAll(selector){return selector==='nav button[data-page]'?nav:selector==='[data-settings-section]'?settingsLinks:selector==='.page'?pages:selector==='[data-range]'?ranges:[];},
    createElement(){const node=new Element();node.ownerDocument=document;return node;},createElementNS(){const node=new Element();node.ownerDocument=document;return node;}};
  const window={ipc:{postMessage(){}},presentationView:{compact:false},clickpulsePlatform:platform,clickpulseMessages:messages,clickpulseLocale:'en'};
  Object.assign(window,{matchMedia:()=>({matches:reducedMotion,addEventListener(){}}),performance:{now:()=>now},setInterval(handler,ms){const id=++timerId;intervals.set(id,{handler,ms});return id;},clearInterval(id){intervals.delete(id);}});
  const context=vm.createContext({Element,document,window,setTimeout,clearTimeout,createClickPulseI18n:createI18n,...(legacyWebKit?{}:{ClickPulseSport:Sport,ClickPulseThemes:Themes,ClickPulseDropdowns:Dropdowns,ClickPulseI18n:createI18n(messages,'en',platform),ClickPulseHistory:H,ClickPulseKeyboard:K.create(platform)})});
  context.literaryRandom=random;
  vm.runInContext('Math.random=literaryRandom;',context);
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

test('Mini-only startup accepts counts without history and preserves the full snapshot across expansion',()=>{
  const a=app();a.window.receiveView({compact:true});
  const mini={timezone:'system',today:{keyboard:12,mouse:3},capture:{listening:true},paused:false,source_mode:false,demo:false};
  a.window.receiveMini(mini);
  assert.equal(a.get('mini-today').textContent,'15');
  a.window.receivePaused(true);assert.equal(a.get('mini-pause-label').textContent,'Resume');
  a.window.receiveView({compact:false});
  const full=snapshot();a.window.receive(full);
  const today=a.get('today').textContent;
  a.window.receiveView({compact:true});a.window.receiveMini({...mini,today:{keyboard:100,mouse:2}});
  assert.equal(a.get('mini-today').textContent,'102');
  a.window.receiveView({compact:false});
  assert.equal(a.get('today').textContent,today);
  a.window.receiveMini({...mini,today:{keyboard:999,mouse:2}});
  assert.equal(a.get('today').textContent,today);
});

test('Mini ignores stale timezone results and needs no full snapshot to show errors',()=>{
  const a=app();a.window.receiveView({compact:true,timezone:'Asia/Shanghai'});
  const mini={timezone:'Asia/Shanghai',today:{keyboard:7,mouse:1},capture:{listening:false,error:'capture failed'},paused:false};
  a.window.receiveMini(mini);assert.equal(a.get('mini-today').textContent,'8');
  a.window.receiveMini({...mini,timezone:'system',today:{keyboard:999,mouse:0}});
  assert.equal(a.get('mini-today').textContent,'8');
  assert.equal(a.get('mini-status').textContent,messages.en['status.recording_error']);
});

test('counter period switches Overview and Mini together without changing calendar history',()=>{
  for(const platform of ['macos','windows','linux']){
    const a=app(platform),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
    const full=snapshot();full.windows.push({seconds:86400,counts:{keyboard:250,mouse:50},approximate:true});
    full.timezone='America/Los_Angeles';
    a.window.receiveView({today_period:'calendar_day',timezone:full.timezone});a.window.receive(full);
    assert.equal(a.get('timezone-row').hidden,false);
    assert.equal(a.get('today').textContent,'120');
    a.get('today-period-select').value='rolling_24h';a.get('today-period-select').fire('change');
    assert.deepEqual(sent.at(-1),{action:'today_period',period:'rolling_24h'});
    a.window.receiveView({today_period:'rolling_24h'});
    assert.equal(a.get('timezone-row').hidden,true);
    assert.equal(a.window.presentationView.timezone,'America/Los_Angeles');
    assert.equal(a.get('today-label').textContent,messages.en['metric.last24']);
    assert.equal(a.get('today').textContent,'≈ 300');
    assert.equal(a.get('week').textContent,'120');assert.equal(full.today.keyboard,100);
    a.window.receiveView({compact:true});assert.equal(a.get('mini-today').textContent,'≈ 300');
    a.window.receiveMini({timezone:full.timezone,today_period:'calendar_day',today:{keyboard:999,mouse:0},capture:{listening:true}});
    assert.equal(a.get('mini-today').textContent,'≈ 300');
    a.window.receiveMini({timezone:full.timezone,today_period:'rolling_24h',today:{keyboard:260,mouse:50},approximate:true,capture:{listening:true}});
    assert.equal(a.get('mini-today').textContent,'≈ 310');
    assert.equal(a.get('mini-keyboard').textContent,'≈ 260');
    a.window.receiveView({locale:'zh-CN',compact:false});
    assert.equal(a.get('today-label').textContent,'最近 24 小时');
    assert.equal(a.get('today-period-select').value,'rolling_24h');
    a.window.receiveView({today_period:'calendar_day'});
    assert.equal(a.get('timezone-row').hidden,false);
    assert.equal(a.get('timezone-select').value,'America/Los_Angeles');
    assert.equal(a.get('today').textContent,'120');assert.equal(a.get('today-label').textContent,'今天');
    assert.equal(a.get('mini-today').textContent,'120');
  }
});

test('cold Mini uses the restored counter period and refuses stale or missing-period results',()=>{
  const a=app();a.window.receiveView({compact:true,today_period:'rolling_24h'});
  assert.equal(a.get('mini-period-label').textContent,'Last 24 hours');
  assert.equal(a.get('mini-today').textContent,'—');
  const mini={timezone:'system',today:{keyboard:11,mouse:2},capture:{listening:true}};
  a.window.receiveMini(mini);assert.equal(a.get('mini-today').textContent,'—');
  a.window.receiveMini({...mini,today_period:'rolling_24h'});assert.equal(a.get('mini-today').textContent,'13');
  a.window.receiveView({today_period:'calendar_day'});assert.equal(a.get('mini-today').textContent,'—');
  a.window.receiveMini({...mini,today_period:'calendar_day'});assert.equal(a.get('mini-today').textContent,'13');
});

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
  const walk=node=>node.dataset?.key?[node]:node.children.flatMap(walk);
  return walk(a.get('keyboard-layout'));
}

test('Heatmap navigation preserves manual sidebar state instead of expanding automatically',()=>{
  for(const platform of ['macos','windows','linux']){
    const a=app(platform),sent=[];a.window.ipc.postMessage=body=>sent.push(JSON.parse(body));a.window.receive(snapshot());
    const collapsed=()=>a.context.document.body.dataset.sidebarCollapsed;
    assert.equal(collapsed(),'false');a.navigate('keys');assert.equal(collapsed(),'false');
    assert.equal(a.get('sidebar-toggle').attributes['aria-expanded'],'true');
    a.window.receive(snapshot());assert.equal(collapsed(),'false');
    a.get('sidebar-toggle').fire();assert.equal(collapsed(),'true');
    a.window.receive(snapshot());assert.equal(collapsed(),'true');
    const state=sent.filter(m=>m.action==='session_state').at(-1).state;
    const b=app(platform);b.window.restoreSessionState(state);b.window.receive(snapshot());
    assert.equal(b.context.document.body.dataset.sidebarCollapsed,'true');
    a.get('sidebar-toggle').fire();a.navigate('settings');assert.equal(collapsed(),'false');
    a.get('sidebar-toggle').fire();assert.equal(a.get('settings-subnav').hidden,true);
    a.get('sidebar-toggle').fire();assert.equal(a.get('settings-subnav').hidden,false);
    a.navigate('keys');assert.equal(collapsed(),'false');
    a.window.receiveView({compact:true});a.window.receiveView({compact:false});assert.equal(collapsed(),'false');
    a.get('sidebar-toggle').fire();
    for(const locale of Object.keys(messages)){
      a.window.setInterfaceLanguage(locale,locale);
      assert.equal(a.get('sidebar-toggle').attributes['aria-label'],messages[locale]['nav.expand']);
      assert.equal(a.get('toolbar-title').textContent,messages[locale]['page.heatmap']);
      const keys=a.context.document.querySelectorAll('nav button[data-page]').find(n=>n.dataset.page==='keys');
      assert.equal(keys.attributes['aria-label'],messages[locale]['page.heatmap']);
      assert.equal(keys.title,messages[locale]['page.heatmap']);
    }
    a.navigate('history');assert.equal(collapsed(),'true','navigation keeps manual choice');
    delete state.sidebar_collapsed;b.window.restoreSessionState(state);b.window.receive(snapshot());
    assert.equal(b.context.document.body.dataset.sidebarCollapsed,'false','old state defaults to expanded on every page');
    assert(!sent.some(m=>['maximize','compact'].includes(m.action)),'navigation never requests a window mode change');
  }
});

test('Heatmap is one page and retains the keyboard view across navigation and recreation',()=>{
  for(const platform of ['macos','windows','linux']){
    const a=app(platform),sent=[];a.window.ipc.postMessage=body=>sent.push(JSON.parse(body));a.window.receive(snapshot());
    a.navigate('keys');assert.equal(a.get('toolbar-title').textContent,'Heatmap');
    assert.equal(a.get('keys').hidden,false);
    a.get('keyboard-view-toggle').fire();assert.equal(a.get('keyboard-treemap').hidden,false);
    a.navigate('overview');assert.equal(a.get('keys').hidden,true);
    const state=sent.filter(m=>m.action==='session_state').at(-1).state;
    assert.equal(state.heatmap_page,undefined,'the removed device choice is no longer saved');
    const b=app(platform);b.window.restoreSessionState(state);b.window.receive(snapshot());b.navigate('keys');
    assert.equal(b.get('keys').hidden,false);assert.equal(b.get('keyboard-treemap').hidden,false);
    a.navigate('keys');assert.equal(a.get('keyboard-treemap').hidden,false);
    assert.equal(a.context.document.body.dataset.sidebarCollapsed,'false');
  }
});

test('legacy mouse sessions reopen the combined Heatmap with translated headings',()=>{
  for(const platform of ['macos','windows','linux']){
    const a=app(platform),sent=[];a.window.ipc.postMessage=body=>sent.push(JSON.parse(body));
    a.window.restoreSessionState({page:'mouse',heatmap_page:'mouse',settings_section:'personalization-settings',trend_days:14,calendar_range:'365',history_range:'all',history_page:0,keyboard_view:'layout',scroll_top:0});a.window.receive(snapshot());
    assert.equal(a.get('keys').hidden,false);
    assert.equal(a.context.document.body.dataset.sidebarCollapsed,'false');
    assert.equal(sent.filter(m=>m.action==='session_state').at(-1).state.page,'keys');
    for(const locale of Object.keys(messages)){
      a.window.setInterfaceLanguage(locale,locale);
      assert.equal(a.get('toolbar-title').textContent,messages[locale]['page.heatmap']);
      const entry=a.context.document.querySelectorAll('nav button[data-page]').find(b=>b.dataset.page==='keys');
      assert.equal(entry.attributes['aria-label'],messages[locale]['page.heatmap']);
    }
  }
});

test('sidebar shortcut toggles once, saves its state and leaves editing and Mini alone',()=>{
  for(const platform of ['macos','windows','linux']){
    const a=app(platform),sent=[];a.window.ipc.postMessage=body=>sent.push(JSON.parse(body));a.window.receive(snapshot());
    const mac=platform==='macos',collapsed=()=>a.context.document.body.dataset.sidebarCollapsed;
    const key=(overrides={})=>{
      const event={key:'b',code:'KeyB',metaKey:mac,ctrlKey:!mac,target:a.context.document.body,
        preventDefault(){this.defaultPrevented=true;},...overrides};
      a.context.document.listeners.keydown(event);return event;
    };
    assert.equal(key().defaultPrevented,true);assert.equal(collapsed(),'true');
    assert.equal(sent.filter(m=>m.action==='session_state').at(-1).state.sidebar_collapsed,true);
    assert.equal(a.get('sidebar-toggle').attributes['aria-keyshortcuts'],mac?'Meta+B':'Control+B');
    assert.match(a.get('sidebar-toggle').title,mac?/⌘B/:/Ctrl\+B/);
    key({repeat:true});assert.equal(collapsed(),'true');
    key({key:'и'});assert.equal(collapsed(),'false');
    for(const overrides of [{metaKey:!mac,ctrlKey:mac},{shiftKey:true},{altKey:true},{isComposing:true},{defaultPrevented:true},{key:'c',code:'KeyC'}]){
      key(overrides);assert.equal(collapsed(),'false');
    }
    const editable=new Element();editable.isContentEditable=true;
    assert.equal(key({target:editable}).defaultPrevented,undefined);
    for(const name of ['input','textarea','select']){
      const input=new Element(name);input.closest=selector=>selector.split(',').includes(name)?input:null;
      assert.equal(key({target:input}).defaultPrevented,undefined);
    }
    a.window.receiveView({compact:true});key();assert.equal(collapsed(),'false');
    a.window.receiveView({compact:false,visible:false});key();assert.equal(collapsed(),'false');
    a.window.receiveView({visible:true});a.get('close-overlay').hidden=false;
    key();assert.equal(collapsed(),'false');
    a.get('close-overlay').hidden=true;key();assert.equal(collapsed(),'true');
  }
});

test('native fullscreen layout survives partial view updates and restores after exit or Mini',()=>{
  const a=app(),fullscreen=()=>a.context.document.body.dataset.windowFullscreen;
  a.window.receiveView({maximized:true,fullscreen:false});assert.equal(fullscreen(),'false');
  a.window.receiveView({fullscreen:true});assert.equal(fullscreen(),'true');
  a.window.receiveView({pinned:false});assert.equal(fullscreen(),'true');
  a.get('sidebar-toggle').fire();assert.equal(fullscreen(),'true');
  a.window.receiveView({compact:true});assert.equal(fullscreen(),'false');
  a.window.receiveView({compact:false,fullscreen:false});assert.equal(fullscreen(),'false');
  a.window.receiveView({fullscreen:true});a.window.receiveView({fullscreen:false});assert.equal(fullscreen(),'false');
});

test('Pause acknowledges immediately while statistics are pending and stale replies cannot undo it',()=>{
  for(const platform of ['macos','windows'])for(const compact of [false,true])for(const source_mode of [false,true]){
    const a=app(platform),s={...snapshot(),source_mode};
    a.window.receive(s);a.window.receiveView({compact,sport_enabled:true,sport_visible:true});
    a.window.receiveSport({keyboard:0,mouse:0,available:true});a.setTime(250);
    a.window.receiveSport({keyboard:10,mouse:0,available:true});
    a.window.receivePaused(true);
    assert.equal(a.get('pause').textContent,messages.en[source_mode?'action.resume_refresh':'action.resume']);
    assert.equal(a.get('mini-pause').attributes['aria-label'],messages.en[source_mode?'action.resume_refresh':'action.resume']);
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
  a.get('calendar-range-select').value='all';
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
  for(const platform of ['macos','windows','linux']){
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
for(const platform of ['windows','linux'])test(`${platform} controls and close choices send distinct actions with optional persistence`,()=>{
  const a=app(platform),sent=[];a.window.ipc.postMessage=message=>sent.push(JSON.parse(message));
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
for(const platform of ['windows','linux'])test(`${platform} close prompt traps keyboard focus, cancels on Escape and ignores header control double clicks`,()=>{
  const a=app(platform),sent=[];a.window.ipc.postMessage=message=>sent.push(JSON.parse(message));
  a.get('close-overlay').hidden=true;a.window.requestClose();
  const key=a.context.document.listeners.keydown;
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
  a.get('calendar-range-select').value='all';a.window.receive(s);
  const aside=a.get('calendar-aside');
  assert.equal(aside.hidden,false);assert.match(aside.textContent,/About 3 copies of Pride and Prejudice/);
  assert.equal(aside.title,messages.en['calendar.literary_basis']);
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
test('all100 independent book/phrase combinations use Chinese or English without changing the UI locale',()=>{
  for(let index=0;index<H.books.length;index++)for(let style=0;style<H.phraseCount;style++){
    let draws=0;
    const a=app('macos',false,false,()=>++draws%2?(index+.5)/H.books.length:(style+.5)/H.phraseCount),s=snapshot();
    const reference=H.books[index];
    s.history_days=[{day:'2026-10-07',counts:{keyboard:reference.characters,mouse:9999999}}];
    a.window.receive(s);
    for(const locale of Object.keys(messages)){
      a.window.receiveView({locale,language_preference:locale});
      const phrase=a.get('calendar-aside').textContent,copyLocale=locale.startsWith('zh')?locale:'en';
      assert.ok(phrase.includes(messages[copyLocale]['calendar.book_'+reference.id]),locale+': '+reference.id);
      assert.equal(phrase,messages[copyLocale]['calendar.literary_'+style+'_one'].replace('{count}','1').replace('{book}',messages[copyLocale]['calendar.book_'+reference.id]));
      assert.equal(a.get('calendar-aside').attributes.lang,copyLocale);
      assert.equal(a.get('page-title').textContent,messages[locale]['page.overview']);
      assert.doesNotMatch(phrase,/\{[^}]+\}/);
      a.window.receive(s);assert.equal(a.get('calendar-aside').textContent,phrase);
    }
    assert.equal(draws,2);
    a.navigate('overview');assert.equal(draws,2);
    const previous=a.get('calendar-aside').textContent;
    a.navigate('keys');a.navigate('overview');assert.equal(draws,4);
    assert.notEqual(vm.runInContext('calendarBook',a.context),index);
    assert.notEqual(vm.runInContext('calendarPhrase',a.context),style);
    assert.notEqual(a.get('calendar-aside').textContent,previous);
    const next=H.books[vm.runInContext('calendarBook',a.context)];
    assert.ok(a.get('calendar-aside').textContent.includes(messages.en['calendar.book_'+next.id]));
  }
});
test('English literary fallback also uses English decimal punctuation and the opening sentence',()=>{
  const a=app(),s=snapshot();
  s.history_days=[{day:'2026-10-07',counts:{keyboard:1750000,mouse:0}}];a.window.receive(s);
  for(const locale of ['ja','ko','fr','de','es']){
    a.window.receiveView({locale});
    assert.equal(a.get('calendar-aside').textContent,'About 2.5 copies of Pride and Prejudice, one keystroke at a time.');
    assert.equal(a.get('calendar-aside').title,messages.en['calendar.literary_basis']);
    assert.equal(a.get('active-days').textContent,messages[locale]['calendar.recorded'].replace('{count}','1'));
  }
  a.window.receive({...s,history_days:[{day:'2026-10-07',counts:{keyboard:1,mouse:0}}]});
  assert.equal(a.get('calendar-aside').textContent,messages.en['calendar.literary_opening']);
});
test('view toggle replaces the total, uses current proportional counts and restores the unchanged keyboard',()=>{
  for(const platform of ['macos','windows']){
    const a=app(platform),s=snapshot();
    s.keyboard_stats={physical:[{code:0,count:80},{code:1,count:20},{code:999,count:10},{code:115,count:2}]};
    a.window.receiveView({keyboard_layout:'compact'});a.window.receive(s);a.navigate('keys');
    const layout=a.get('keyboard-layout'),order=keyCells(a).filter(cell=>cell.dataset.key!=='home').map(cell=>cell.dataset.key);
    a.get('keyboard-view-toggle').fire();
    const blocks=a.get('keyboard-treemap');
    assert.equal(layout.parentElement.hidden,true);assert.equal(blocks.hidden,false);
    assert.equal(a.get('keyboard-view-toggle').attributes['aria-pressed'],'true');
    assert.equal(a.get('keyboard-view-label').textContent,'Keyboard layout');
    assert.equal(blocks.children.some(cell=>cell.dataset.key==='other'),false);
    assert.equal(blocks.children.some(cell=>cell.dataset.key==='home'),true);
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
test('keyboard and treemap keep extended keys and show only unresolved counts in the footer',()=>{
  for(const platform of ['macos','windows','linux']){
    const a=app(platform),s=snapshot(),physical=[{code:115,count:1000},{code:86,count:7},{code:999,count:3}];
    a.window.receive({...s,keyboard_stats:{physical}});a.navigate('keys');
    assert.equal(a.get('keyboard-other').hidden,false);
    assert.equal(a.get('keyboard-other').textContent,'Other keys: 3');
    a.get('keyboard-view-toggle').fire();
    assert.equal(a.get('keyboard-treemap').children.filter(key=>key.dataset.key).length,2);
    a.window.receive({...s,keyboard_stats:{physical:[...physical,{code:0,count:5}]}});
    assert.equal(a.get('keyboard-treemap').children.filter(key=>key.dataset.key).length,3);
    a.window.receiveView({locale:'zh-CN'});
    assert.equal(a.get('keyboard-other').textContent,'其他按键：3 次');
    a.get('keyboard-view-toggle').fire();
    assert.equal(keyCells(a).find(key=>key.dataset.key==='A').lastElementChild.textContent,'5');
    a.window.receive({...s,keyboard_stats:{physical:[]}});
    assert.equal(a.get('keyboard-other').hidden,true);
    a.get('keyboard-view-toggle').fire();
    assert.equal(a.get('keyboard-other').textContent,'');
  }
});
test('Mac full layout stops at F19 while additional keys stay available in treemap',()=>{
  const a=app(),s=snapshot();a.window.receiveView({keyboard_layout:'full_size'});
  a.window.receive({...s,keyboard_stats:{physical:[]}});a.navigate('keys');
  for(let i=13;i<=19;i++)assert.equal(keyCells(a).find(k=>k.dataset.key==='f'+i).lastElementChild.textContent,'0');
  a.window.receive({...s,keyboard_stats:{physical:[{code:105,count:10},{code:80,count:20},{code:90,count:5}]}});
  for(const layout of ['compact','full_size']){
    a.window.receiveView({keyboard_layout:layout});
    const cells=keyCells(a);assert.equal(new Set(cells.map(k=>k.dataset.key)).size,cells.length);
    if(layout==='compact'){
      assert.equal(a.get('keyboard-layout').children.length,6);
      assert.ok(!cells.some(k=>['f13','f19','f20'].includes(k.dataset.key)));
    }else{
      assert.equal(cells.find(k=>k.dataset.key==='f13').lastElementChild.textContent,'10');
      assert.equal(cells.find(k=>k.dataset.key==='f19').lastElementChild.textContent,'20');
      assert.ok(!cells.some(k=>k.dataset.key==='f20'));
    }
  }
  a.get('keyboard-view-toggle').fire();
  assert.deepEqual(a.get('keyboard-treemap').children.filter(k=>k.dataset.key).map(k=>k.dataset.key).sort(),['f13','f19','f20']);
  a.get('keyboard-view-toggle').fire();
  assert.equal(keyCells(a).filter(k=>k.dataset.key==='f13').length,1);
});
test('recorded extra keys never append a bottom row and retain their treemap counts',()=>{
  for(const platform of ['macos','windows','linux']){
    const a=app(platform),s=snapshot();
    const physical=platform==='macos'?[{code:81,count:1},{code:104,count:3}]:[{code:72,count:1},{code:10,count:3}];
    const extra=platform==='macos'?'jis-kana':'volume-up';
    a.window.receive({...s,keyboard_stats:{physical}});a.navigate('keys');
    for(const layout of ['compact','full_size']){
      a.window.receiveView({keyboard_layout:layout});
      assert.ok(!keyCells(a).some(k=>k.dataset.key===extra));
      assert.ok(!a.get('keyboard-layout').children.some(row=>row.className.includes('supplemental-row')));
      if(platform==='macos'){
        const equals=keyCells(a).filter(k=>k.dataset.key==='numequals');
        assert.equal(equals.length,layout==='full_size'?1:0);
        if(equals.length){assert.equal(equals[0].style.gridColumn,'2 / span 1');assert.equal(equals[0].style.gridRow,'1 / span 1');assert.equal(equals[0].lastElementChild.textContent,'1');}
      }
    }
    a.get('keyboard-view-toggle').fire();
    const blocks=a.get('keyboard-treemap').children.filter(k=>k.dataset.key);
    assert.equal(blocks.length,2);
    assert.ok(blocks.some(k=>k.dataset.key===extra));
    assert.equal(a.get('keyboard-other').hidden,true);
  }
});
test('Windows settings report native capture without offering macOS permission controls',()=>{
  const a=app('windows');a.window.receive(snapshot());a.navigate('settings');
  assert.equal(a.get('permission-row').hidden,true);assert.equal(a.get('capture-settings').hidden,false);
  assert.equal(a.get('permission-description').textContent,messages.en['settings.windows_capture']);
  assert.equal(a.get('autostart-row').hidden,false);
  a.window.receive({...snapshot(),capture:{listening:false,error:'failure'}});
  assert.equal(a.get('permission-description').textContent,messages.en['status.unavailable']);
});
test('software-input filtering defaults off and is available only for supported capture backends',()=>{
  for(const platform of ['macos','windows','linux']){
    const a=app(platform),sent=[];a.window.ipc.postMessage=body=>sent.push(JSON.parse(body));
    a.window.receive(snapshot());a.navigate('settings');
    assert.equal(a.get('ignore-software-row').hidden,platform==='linux');
    assert.equal(a.get('ignore-software-input').checked,false);
    a.window.receiveView({ignore_software_input:true});
    assert.equal(a.get('ignore-software-input').checked,true);
    a.get('ignore-software-input').checked=false;a.get('ignore-software-input').fire('change');
    assert.deepEqual(sent.at(-1),{action:'ignore_software_input',enabled:false});
  }
});

test('startup switch shows current state and toggles only when a different state is chosen',()=>{
  const a=app('windows'),sent=[];a.window.ipc.postMessage=body=>sent.push(JSON.parse(body));
  a.window.receive(snapshot());a.navigate('settings');
  const control=a.get('autostart');sent.length=0;
  assert.equal(control.checked,true);control.fire('change');assert.equal(sent.length,0);
  control.checked=false;control.fire('change');assert.deepEqual(sent.pop(),{action:'autostart'});
  a.window.receive({...snapshot(),auto_start:false});assert.equal(control.checked,false);
  control.fire('change');assert.equal(sent.length,0);
  control.checked=true;control.fire('change');assert.deepEqual(sent.pop(),{action:'autostart'});
  a.window.receive({...snapshot(),auto_start:false,auto_start_available:false});
  control.checked=true;control.fire('change');assert.equal(sent.length,0);
});

test('macOS access guidance explains stale grants and names a completed update',()=>{
  const a=app(),s=snapshot(),denied={...s,capture:{listening:false,permission_needed:true,error:null}};
  a.window.receive(denied);
  assert.equal(a.get('permission-banner').hidden,false);
  assert.equal(a.get('permission-title').hidden,false);assert.equal(a.get('permission-updated-title').hidden,true);
  a.navigate('settings');assert.equal(a.get('permission-reset-settings').hidden,false);
  a.window.receiveUpdateNotice({success:false,version:'0.3.0'});
  assert.equal(a.get('permission-updated-title').hidden,true);
  // The notice can arrive after the first snapshot; the banner updates at once.
  a.window.receiveUpdateNotice({success:true,version:'0.3.0'});
  assert.equal(a.get('toast').textContent,messages.en['update.installed'].replace('{version}','0.3.0'));
  assert.equal(a.get('permission-title').hidden,true);assert.equal(a.get('permission-updated-title').hidden,false);
  a.window.receive(s);
  assert.equal(a.get('permission-banner').hidden,true);assert.equal(a.get('permission-reset-settings').hidden,true);
  const w=app('windows');w.navigate('settings');w.window.receive(denied);
  assert.equal(w.get('permission-reset-settings').hidden,true);
});

test('settings distinguish storage modes and only offer login startup for the default database',()=>{
  const a=app('windows');a.navigate('settings');
  for(const mode of ['default','custom','source','demo']){
    const s={...snapshot(),database_mode:mode,auto_start_available:mode==='default',source_mode:mode==='source',demo:mode==='demo'};
    a.window.receive(s);
    assert.equal(a.get('autostart-row').hidden,mode!=='default');
    assert.equal(a.get('db-path').textContent,mode==='demo'?messages.en['demo.storage']:s.database_path);
    assert.equal(a.get('db-description').hidden,mode==='default');
    assert.equal(a.get('db-description').textContent,mode==='default'?'':messages.en['settings.storage_'+mode]);
    assert.equal(a.get('storage-directory').hidden,mode!=='default');
    assert.equal(a.get('import-history-row').hidden,mode!=='default');
    a.window.receiveView({locale:'zh-CN',language_preference:'system'});
    assert.equal(a.get('db-description').textContent,mode==='default'?'':messages['zh-CN']['settings.storage_'+mode]);
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

test('mode changes clear stale selections while same-mode refreshes retain copying',()=>{
  for(const platform of ['macos','windows','linux']){
    const a=app(platform);let clears=0;
    a.window.getSelection=()=>({removeAllRanges(){clears++;}});
    a.window.receiveView({compact:false});assert.equal(clears,0);
    a.window.receiveView({compact:true});assert.equal(clears,1);
    a.window.receiveView({compact:true,pinned:false});assert.equal(clears,1);
    a.window.receiveView({visible:false});assert.equal(clears,1);
    a.window.receiveView({compact:false,visible:true});assert.equal(clears,2);
    a.window.receiveView({compact:false});assert.equal(clears,2);
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

test('the Windows heatmap automatically adds navigation and a number pad when used',()=>{
  const a=app('windows'),s=snapshot();a.navigate('keys');
  a.window.receive({...s,keyboard_stats:{physical:[]}});
  assert.equal(a.get('keyboard-layout').children.length,6);
  a.window.receive({...s,keyboard_stats:{physical:[{code:115,count:100},{code:86,count:40},{code:49,count:2},{code:999,count:3}]}});
  assert.equal(a.get('keyboard-layout').children[0].className,'keyboard-main');
  assert.ok(keyCells(a).some(cell=>cell.dataset.key==='home'));
  assert.ok(keyCells(a).some(cell=>cell.dataset.key==='num4'));
  assert.equal(a.get('keyboard-other').hidden,false);
  assert.equal(a.get('keyboard-other').textContent,'Other keys: 3');
  a.window.receiveView({keyboard_layout:'compact'});
  assert.equal(a.get('keyboard-layout').children[0].className,'keyboard-row function-row');
  assert.equal(keyCells(a).filter(cell=>cell.dataset.key==='num4').length,0);
  assert.equal(a.get('keyboard-layout').children.length,6);
});
test('header layout choices use saved preferences and stay in sync with Settings and restored views',()=>{
  for(const platform of ['macos','windows','linux']){
    const a=app(platform),s=snapshot(),sent=[];
    s.keyboard_stats={physical:[{code:0,count:25}]};
    a.window.ipc.postMessage=body=>sent.push(JSON.parse(body));
    a.window.receive(s);a.navigate('keys');
    assert.equal(a.get('keyboard-compact').attributes['aria-pressed'],'true');
    for(const [button,layout] of [['keyboard-full','full_size'],['keyboard-compact','compact']]){
      a.get(button).fire();
      assert.deepEqual(sent.at(-1),{action:'keyboard_layout',layout});
      a.window.receiveView({keyboard_layout:layout});
      assert.equal(a.get('keyboard-layout-select').value,layout);
      assert.equal(a.get(button).attributes['aria-pressed'],'true');
      assert.equal(keyCells(a).some(key=>key.dataset.key==='num0'),layout==='full_size');
      a.window.receiveView({compact:true});a.window.receiveView({compact:false,locale:'zh-CN'});
      a.navigate('overview');a.navigate('keys');
      assert.equal(a.get(button).attributes['aria-pressed'],'true');
      a.get('keyboard-view-toggle').fire();assert.equal(a.get('keyboard-layout-mode').hidden,true);
      a.get('keyboard-view-toggle').fire();assert.equal(a.get('keyboard-layout-mode').hidden,false);
      const restored=app(platform);restored.window.receiveView({keyboard_layout:layout});
      restored.window.receive(s);restored.navigate('keys');
      assert.equal(restored.get(button).attributes['aria-pressed'],'true');
    }
    a.get('keyboard-layout-select').value='full_size';a.get('keyboard-layout-select').fire('change');
    assert.deepEqual(sent.at(-1),{action:'keyboard_layout',layout:'full_size'});
    a.window.receiveView({keyboard_layout:'full_size'});
    assert.equal(a.get('keyboard-full').attributes['aria-pressed'],'true');
  }
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

test('activity mix compares lifetime keyboard and mouse totals regardless of button detail',()=>{
  const a=app(),s=snapshot();
  const legend=()=>a.get('composition-legend').children.map(node=>[node.children[1].children[1],node.children[0].textContent]);
  s.mouse_buttons={left:14,right:2,middle:2,side:1,other:1};
  a.window.receive(s);
  assert.deepEqual(legend(),[['Keyboard','100'],['Mouse','20']]);
  const svg=a.get('donut').children[0];
  assert.ok(svg.children.every(node=>node.attributes.cx==='85')); // circles, no center text
  assert.equal(svg.children.filter(node=>node.attributes['stroke-dasharray']).length,2);
  assert.match(svg.attributes['aria-label'],/Mouse: 20 \(16\.7%\)/);
  // Older histories can have fewer per-button counts than total clicks.
  a.window.receive({...s,mouse_buttons:{left:5,right:2}});
  assert.equal(a.get('donut').replacements,1);
  assert.deepEqual(legend()[1],['Mouse','20']);
  for(const locale of Object.keys(messages)){
    a.window.receiveView({locale});
    assert.deepEqual(legend().map(([label])=>label),[messages[locale]['common.keyboard'],messages[locale]['common.mouse']]);
  }
  a.window.receive({...s,lifetime:{keyboard:0,mouse:0}});
  assert.equal(a.get('donut').children[0].children.length,1);
  assert.deepEqual(legend().map(([,count])=>count),['0','0']);
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
test('past-year calendar pads missing dates in one cross-year grid and retains access to older history',()=>{
  const a=app(),s=snapshot(),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
  s.history_days=[{day:'2023-03-01',counts:{keyboard:700000,mouse:9000}},...s.history_days];
  a.window.receive(s);
  const cells=()=>a.get('heatmap').children.flatMap(section=>section.children[2].children[1].children).filter(cell=>cell.dataset.day);
  assert.equal(a.get('calendar-range-select').value,'365');assert.equal(a.get('heatmap').children.length,1);
  assert.equal(cells().length,365);assert.equal(cells()[0].dataset.day,'2025-10-08');
  assert.equal(cells().at(-1).dataset.day,'2026-10-07');
  assert.ok(cells().slice(0,-1).every(cell=>cell.className==='heat-cell level-0'));
  assert.match(cells().at(-1).className,/today/);assert.equal(a.get('active-days').textContent,'1 active days');
  assert.match(a.get('calendar-range').textContent,/120 events/);
  a.window.captureSessionState();const saved=sent.at(-1).state;
  const b=app();b.window.restoreSessionState(saved);b.window.receive(s);
  assert.equal(b.get('calendar-range-select').value,'365');assert.equal(b.get('heatmap').children.length,1);
  const select=a.get('calendar-range-select');select.value='2023';select.fire('change');
  assert.match(a.get('calendar-range').textContent,/709,000 events/);
  select.value='all';select.fire('change');
  assert.match(a.get('calendar-range').textContent,/709,120 events/);
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
test('storage health reaches full and mini views without a readable snapshot',()=>{
  const a=app(),s=snapshot();a.window.receiveView({compact:true});a.window.receive(s);
  a.window.receiveCapture({listening:true,error:'locked',error_code:'error.storage_retrying'});
  assert.equal(a.get('mini-status').textContent,messages.en['status.recording_error']);
  assert.equal(a.get('mini-today').textContent,'120');
  a.window.receiveView({compact:false});
  assert.equal(a.get('error-banner').textContent,messages.en['error.storage_retrying']);
  a.window.receiveCapture({listening:true,error:null});
  assert.equal(a.get('error-banner').hidden,true);
  assert.equal(a.get('mini-status').textContent,messages.en['status.recording']);
});
test('grouped arrows retain unique per-key counts through refresh, theme and language changes on both platforms',()=>{
  for(const platform of ['macos','windows']){
    const a=app(platform),s=snapshot();
    a.window.receiveView({keyboard_layout:'compact'});
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
  const a=app();a.window.receiveView({});a.window.receive(snapshot());
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
    assert.equal(a.settingsLinks.find(b=>b.dataset.settingsSection==='capture-settings').hidden,false);
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
  for(const [locale,label] of [['de','Systemzeitzone'],['es','Zona horaria del sistema']]){
    a.window.receiveView({locale,language_preference:locale});
    assert.equal(a.get('timezone-select').children[0].textContent,label);
    assert.equal(a.get('timezone-select').value,'system');
    assert.notEqual(label,messages[locale]['language.system']);
  }
});

test('manual checks remain available and an offered update never installs without a click',()=>{
  for(const platform of ['macos','windows']){
    const a=app(platform),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
    a.window.receiveUpdateState({status:'idle',current:'0.2.0'});assert.equal(sent.length,0);
    a.get('check-updates').fire();assert.deepEqual(sent.pop(),{action:'check_updates'});
    a.window.receiveUpdateState({status:'checking'});a.get('check-updates').fire();assert.equal(sent.length,0);
    a.window.receiveUpdateState({status:'available',version:'0.2.1',can_install:true});assert.equal(sent.length,0);
    assert.equal(a.get('check-updates').textContent,messages.en['update.download']);
    assert.equal(a.get('update-details').hidden,false);
    a.get('check-updates').fire();assert.deepEqual(sent.pop(),{action:'install_update'});
  }
});
test('Scoop-managed updates keep checks and localized guidance without sending install requests',()=>{
  for(const locale of Object.keys(messages)){
    const a=app('windows'),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
    a.window.receiveView({locale,language_preference:locale});
    a.window.receiveUpdateState({status:'available',current:'0.2.1',version:'9.9.9',can_install:true,update_command:'scoop update clickpulse'});
    assert.equal(a.get('check-updates').textContent,messages[locale]['update.check']);
    assert.equal(a.get('update-managed').hidden,false);
    assert.ok(a.get('update-managed').textContent.includes('scoop update clickpulse'));
    assert.equal(a.get('automatic-updates').disabled,false);
    a.get('check-updates').fire();assert.deepEqual(sent.pop(),{action:'check_updates'});
    a.get('update-details').fire();assert.deepEqual(sent.pop(),{action:'update_release'});
    a.window.receiveUpdateAvailable('9.9.9');assert.equal(a.get('toast').textContent,messages[locale]['update.found_managed']);
    assert.ok(!a.get('toast').textContent.includes('scoop update clickpulse'),'the update command is shown only in Settings');
    assert.equal(sent.length,0);
    a.window.receiveUpdateState({status:'current'});assert.equal(a.get('update-managed').hidden,false);
    a.window.receiveUpdateState({status:'available',can_install:true,update_command:null});
    assert.equal(a.get('update-managed').hidden,true);
    a.get('check-updates').fire();assert.deepEqual(sent.pop(),{action:'install_update'});
  }
});
test('downloaded updates await confirmation in every language and never open a browser',()=>{
  for(const platform of ['macos','windows','linux'])for(const locale of Object.keys(messages)){
    const a=app(platform),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
    a.window.receiveView({locale,language_preference:locale});
    a.window.receiveUpdateState({status:'downloading',version:'0.3.1',can_install:true});
    a.window.receiveUpdateState({status:'ready',version:'0.3.1',can_install:true});
    assert.equal(sent.length,0);
    assert.equal(a.get('check-updates').textContent,messages[locale]['update.install']);
    assert.equal(a.get('check-updates').disabled,false);
    assert.equal(a.get('update-status').textContent,createI18n(messages,locale).t('update.ready',{version:'0.3.1'}));
    assert.equal(a.get('update-install-help').hidden,true);
    a.get('check-updates').fire();assert.deepEqual(sent.pop(),{action:'install_update'});
  }
});
test('WinGet guidance replaces Scoop branding without offering an in-app installer',()=>{
  for(const locale of Object.keys(messages)){
    const a=app('windows'),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
    a.window.receiveView({locale,language_preference:locale});
    const command='winget upgrade --id endNone.ClickPulse --exact';
    a.window.receiveUpdateState({status:'available',version:'0.3.1',can_install:true,update_command:command});
    assert.ok(a.get('update-managed').textContent.includes('WinGet'));
    assert.ok(a.get('update-managed').textContent.includes(command));
    assert.ok(!a.get('update-managed').textContent.includes('Scoop'));
    a.get('check-updates').fire();assert.deepEqual(sent.pop(),{action:'check_updates'});
  }
});
test('background failure keeps version copy and only explicit error states show failures',()=>{
  for(const locale of Object.keys(messages)){
    const a=app();a.window.receiveView({locale,language_preference:locale});
    a.window.receiveUpdateState({status:'idle',current:'0.2.0'});
    assert.equal(a.get('update-status').textContent,createI18n(messages,locale).t('update.version',{version:'0.2.0'}));
    assert.equal(a.get('update-details').hidden,true);
    assert.equal(a.get('check-updates').disabled,false);
    a.window.receiveUpdateState({status:'error'});
    assert.equal(a.get('update-status').textContent,messages[locale]['update.failed']);
  }
});
test('new-version reminders are localized and never initiate a check, download or install',()=>{
  for(const platform of ['macos','windows'])for(const locale of Object.keys(messages)){
    const a=app(platform),sent=[];let timeout,expire;
    a.context.setTimeout=(fn,ms)=>{timeout=ms;expire=fn;return 1;};a.context.clearTimeout=()=>{};
    a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
    a.window.receiveView({locale,language_preference:locale});
    a.window.receiveUpdateAvailable('0.2.1');
    assert.equal(a.get('toast').textContent,createI18n(messages,locale).t('update.found',{version:'0.2.1'}));
    assert.equal(a.get('toast').hidden,false);assert.equal(timeout,8000);
    assert.equal(sent.length,0);expire();assert.equal(a.get('toast').hidden,true);
  }
});
test('daily update preference and last attempt survive UI refresh and language changes',()=>{
  for(const platform of ['macos','windows'])for(const locale of Object.keys(messages)){
    const a=app(platform),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
    a.window.receiveView({locale,auto_check_updates:false,last_update_check:1794310000});
    assert.equal(a.get('automatic-updates').checked,false);
    assert.equal(a.get('update-last-checked').hidden,false);
    assert.ok(a.get('update-last-checked').textContent.startsWith(messages[locale]['update.last_checked'].split('{time}')[0]));
    a.get('automatic-updates').checked=true;a.get('automatic-updates').fire('change');
    assert.deepEqual(sent.pop(),{action:'automatic_updates',enabled:true});
    a.window.receiveView({auto_check_updates:true});a.window.receive(snapshot());
    assert.equal(a.get('automatic-updates').checked,true);assert.ok(sent.every(message=>message.action==='session_state'));sent.length=0;
    a.window.receiveUpdateState({status:'checking'});assert.equal(a.get('automatic-updates').disabled,false);
    a.window.receiveUpdateState({status:'downloading'});assert.equal(a.get('automatic-updates').disabled,false);
    a.get('automatic-updates').checked=false;a.get('automatic-updates').fire('change');
    assert.deepEqual(sent.pop(),{action:'automatic_updates',enabled:false});
    for(const last_update_check of [null,undefined,'invalid',Number.MAX_VALUE]){
      a.window.receiveView({last_update_check});assert.equal(a.get('update-last-checked').hidden,true);
    }
  }
});
test('demo, read-only and independent sessions cannot change the installed app daily check',()=>{
  for(const locale of Object.keys(messages)){
    const a=app('macos'),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
    a.window.receiveView({locale,auto_check_updates:true,last_update_check:1794310000});
    assert.equal(a.get('update-automatic-unavailable').hidden,true);
    a.window.receiveUpdateState({status:'current',current:'0.2.0',automatic:false});
    const toggle=a.get('automatic-updates');
    assert.equal(toggle.checked,false);assert.equal(toggle.disabled,true);
    assert.equal(a.get('update-automatic-unavailable').hidden,false);
    assert.equal(a.get('update-last-checked').hidden,true);
    toggle.checked=true;toggle.fire('change');assert.equal(sent.length,0);
    // Manual checks stay available in these sessions.
    a.get('check-updates').fire();assert.deepEqual(sent.pop(),{action:'check_updates'});
    assert.ok(messages[locale]['update.automatic_unavailable']);
  }
});
test('background downloads leave data controls available until installation starts',()=>{
  const a=app();a.window.receive(snapshot());
  for(const status of ['downloading','verifying','preparing','installing']){
    a.window.receiveUpdateState({status,current:'0.2.0',received:50,total:100});
    assert.equal(a.get('check-updates').disabled,true);
    const installing=['preparing','installing'].includes(status);
    assert.equal(a.get('storage-directory').disabled,installing);assert.equal(a.get('import-history').disabled,installing);
    assert.equal(a.get('reveal').disabled,installing);
    assert.equal(a.get('automatic-updates').disabled,installing);
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
  assert.equal(a.get('update-install-help').hidden,false);
  assert.equal(a.get('update-install-help').textContent,messages.en['update.install_help']);
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

test('Linux permission guidance and close decisions remain usable without macOS settings',()=>{
  const a=app('linux'),s=snapshot(),sent=[];
  a.window.ipc.postMessage=body=>sent.push(JSON.parse(body));
  a.window.receive({...s,capture:{listening:false,permission_needed:true,error:'denied',error_code:'error.linux_permission'}});
  a.navigate('settings');
  assert.equal(a.get('capture-settings').hidden,false);
  assert.equal(a.get('permission-reset').hidden,true);
  assert.equal(a.get('permission-reset-settings').hidden,true);
  assert.match(a.get('permission-description').textContent,/usermod -aG input/);
  assert.equal(a.get('error-banner').textContent,messages.en['error.linux_permission']);
  a.get('settings-permission').fire();assert.equal(sent.at(-1).action,'permission');
  a.get('close-overlay').hidden=true;
  a.window.requestClose();assert.equal(a.get('close-overlay').hidden,false);
  a.get('close-quit').fire();assert.deepEqual(sent.at(-1),{action:'close_decision',close_action:'quit',remember:false});
  a.window.receive(s);assert.equal(a.get('permission-banner').hidden,true);
  assert.equal(a.get('permission-description').textContent,messages.en['settings.authorized.linux']);
});

test('recreated views restore navigation and filters after fresh data arrives',()=>{
  const a=app(),sent=[];a.window.ipc.postMessage=body=>sent.push(JSON.parse(body));
  a.window.receive(snapshot());a.navigate('settings');a.jumpTo('app-settings');
  a.get('main').scrollTop=240;a.window.captureSessionState();
  const saved=sent.filter(message=>message.action==='session_state').at(-1).state;
  assert.equal(saved.page,'settings');assert.equal(saved.settings_section,'app-settings');
  assert.equal(saved.scroll_top,240);
  const b=app(),restored=[];b.window.ipc.postMessage=body=>restored.push(JSON.parse(body));
  b.window.restoreSessionState(saved);
  b.window.receiveView({compact:true});b.window.captureSessionState();
  assert.equal(restored.length,0,'Mini must not overwrite the pending full-window state');
  b.window.receiveView({compact:false});b.window.receive(snapshot());
  assert.equal(b.get('settings').hidden,false);assert.equal(b.get('overview').hidden,true);
  assert.equal(b.get('main').scrollTop,240);
  assert.equal(b.settingsLinks.find(link=>link.dataset.settingsSection==='app-settings').attributes['aria-current'],'location');
  b.window.captureSessionState();
  assert.equal(restored.at(-1).state.settings_section,'app-settings');
});

test('recreation restores history, trend and keyboard choices and clamps removed year filters',()=>{
  for(const page of ['overview','history','keys']){
    const a=app(),sent=[];a.window.ipc.postMessage=body=>sent.push(JSON.parse(body));
    a.window.restoreSessionState({page,settings_section:'personalization-settings',trend_days:90,calendar_range:'90',history_range:'1000',history_page:100,keyboard_view:'treemap',scroll_top:120});
    a.window.receive(snapshot());
    assert.equal(a.get(page).hidden,false);
    assert.equal(a.get('history-range-select').value,'all');
    a.window.captureSessionState();
    assert.equal(sent.at(-1).state.trend_days,90);assert.equal(sent.at(-1).state.keyboard_view,'treemap');
    assert.equal(sent.at(-1).state.calendar_range,'90');
    if(page==='history')assert.ok(sent.at(-1).state.history_page<100);
    const count=sent.length;a.window.receiveView({visible:false});a.get('main').scrollTop=0;a.window.captureSessionState();
    assert.equal(sent.length,count,'Hidden layout changes must not overwrite scroll state');
  }
});

test('cold-view scrolling waits for fonts instead of saving a clamped position',async()=>{
  const a=app('windows'),sent=[];let fontsReady,top=0,max=420;
  a.context.document.fonts={ready:new Promise(resolve=>{fontsReady=resolve;})};
  Object.defineProperty(a.get('main'),'scrollTop',{get:()=>top,set:value=>{top=Math.min(value,max);}});
  a.window.ipc.postMessage=body=>sent.push(JSON.parse(body));
  a.window.restoreSessionState({page:'settings',settings_section:'app-settings',trend_days:90,calendar_range:'90',history_range:'all',history_page:0,keyboard_view:'layout',scroll_top:434});
  a.window.receive(snapshot());a.window.captureSessionState();
  assert.equal(top,420);assert.equal(sent.length,0,'do not replace the saved position with the fallback-font clamp');
  max=434;fontsReady();await Promise.resolve();
  assert.equal(top,434);
  assert.equal(sent.at(-1).state.scroll_top,434);
  assert.equal(a.get('main').style.values['overflow-anchor'],'auto');
});

test('late fonts cannot pull the user back after navigating away',async()=>{
  const a=app('windows');let fontsReady;
  a.context.document.fonts={ready:new Promise(resolve=>{fontsReady=resolve;})};
  a.window.restoreSessionState({page:'settings',settings_section:'app-settings',trend_days:14,calendar_range:'all',history_range:'all',history_page:0,keyboard_view:'layout',scroll_top:434});
  a.window.receive(snapshot());a.navigate('history');a.get('main').scrollTop=80;
  fontsReady();await Promise.resolve();
  assert.equal(a.get('main').scrollTop,80);
  assert.equal(a.get('history').hidden,false);
});

test('font completion while hidden preserves the pending full-window scroll',async()=>{
  const a=app('windows');let fontsReady;
  a.context.document.fonts={ready:new Promise(resolve=>{fontsReady=resolve;})};
  a.window.restoreSessionState({page:'settings',settings_section:'app-settings',trend_days:14,calendar_range:'all',history_range:'all',history_page:0,keyboard_view:'layout',scroll_top:434});
  a.window.receive(snapshot());a.window.receiveView({compact:true});a.get('main').scrollTop=0;
  fontsReady();await Promise.resolve();
  assert.equal(a.get('main').scrollTop,0);
  a.window.receiveView({compact:false});a.window.receive(snapshot());
  assert.equal(a.get('main').scrollTop,434);
});

test('local brand images are permitted without allowing remote image sources',()=>{
  const html=readFileSync('ui/index.html','utf8');
  const policy=html.match(/http-equiv="Content-Security-Policy" content="([^"]+)"/)[1];
  const imageSources=policy.split(';').map(part=>part.trim().split(/\s+/)).find(parts=>parts[0]==='img-src').slice(1);
  assert.ok(imageSources.includes("'self'"),'protocol-served brand images must be allowed');
  assert.ok(imageSources.includes('data:'),'offline documentation previews still embed images');
  assert.ok(!imageSources.some(source=>['*','http:','https:'].includes(source)),'remote images stay blocked');
});


test('uninstall is available only for owned Windows data and never during an update',()=>{
  const a=app('windows'),s=snapshot(),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
  a.window.receive(s);a.navigate('settings');
  assert.equal(a.get('uninstall-row').hidden,false);
  a.get('uninstall').fire();assert.equal(sent.at(-1).action,'uninstall');
  const before=sent.length;
  a.window.receiveUpdateState({status:'downloading',automatic:true});a.get('uninstall').fire();
  assert.equal(a.get('uninstall').disabled,true);
  assert.equal(sent.length,before);
  a.window.receiveUpdateState({status:'idle'});assert.equal(a.get('uninstall').disabled,false);
  a.window.receiveDataBusy(true);a.get('uninstall').fire();assert.equal(sent.length,before);
  assert.equal(a.get('uninstall').disabled,true);a.window.receiveDataBusy(false);
  a.window.receiveUpdateState({status:'checking'});a.get('uninstall').fire();assert.equal(sent.length,before);
  assert.equal(a.get('uninstall').disabled,true);a.window.receiveUpdateState({status:'idle'});
  a.window.receive({...s,demo:true});assert.equal(a.get('uninstall-row').hidden,true);
  for(const mode of ['custom','source']){
    a.window.receive({...s,database_mode:mode});assert.equal(a.get('uninstall-row').hidden,true);
    a.get('uninstall').fire();assert.equal(sent.length,before);
  }
  for(const platform of ['macos','linux']){
    const b=app(platform);b.window.receive(s);b.navigate('settings');assert.equal(b.get('uninstall-row').hidden,true);
  }
});

test('close confirmation closes the date popup and releases inert state on Hide or Mini',()=>{
 for(const platform of ['windows','linux'])for(const next of [{visible:false},{compact:true}]){
  const a=app(platform);let closed=0;a.context.ClickPulseDatePicker={closeAll(){closed++;}};
  a.get('close-overlay').hidden=true;a.window.requestClose();
  assert.equal(closed,1);assert.equal(a.get('close-overlay').hidden,false);assert.equal(a.get('main').inert,true);
  a.window.receiveView(next);
  assert.equal(a.get('close-overlay').hidden,true);assert.equal(a.get('main').inert,false);assert.equal(a.get('sidebar').inert,false);assert.equal(a.get('mini-widget').inert,false);
 }
});
test('changing language retains paused and maximized accessible labels immediately',()=>{
 const a=app('windows');a.window.receive(snapshot());a.window.receivePaused(true);a.window.receiveView({maximized:true});
 for(const locale of ['zh-CN','fr','en']){
  a.window.setInterfaceLanguage(locale,locale);
  assert.equal(a.get('mini-pause').attributes['aria-label'],messages[locale]['action.resume']);
  assert.equal(a.get('window-maximize').attributes['aria-label'],messages[locale]['window.restore']);
 }
});
test('hourly details clear on keyboard focus leaving the chart, not when moving between bars',()=>{
 const a=app();a.window.receive(hourlyFixture());const plot=a.get('hourly-chart').children[0],slot=plot.children[0];
 slot.fire('focus');const before=a.get('hourly-detail').textContent;assert(before);
 plot.listeners.focusout({relatedTarget:plot.children[1]});assert.equal(a.get('hourly-detail').textContent,before);
 plot.listeners.focusout({relatedTarget:a.get('pause')});assert.equal(a.get('hourly-detail').textContent,'');
});

test('live input updates the existing charts and only one calendar tooltip',()=>{
  const a=app(),s=snapshot();a.window.receive(s);
  const countTooltips=()=>a.get('heatmap').children.flatMap(section=>section.children[2].children[1].children).filter(cell=>cell.dataset.day);
  const cells=countTooltips(),oldTitles=cells.map(cell=>cell.title),trend=a.get('trend-chart').children[0],donut=a.get('donut').children[0],windows=a.get('windows').children[0];
  let tooltipWrites=0;
  for(const cell of cells){let title=cell.title;Object.defineProperty(cell,'title',{get:()=>title,set:value=>{tooltipWrites++;title=value;}});}
  const change=rows=>rows.map((row,index)=>index===rows.length-1?{...row,counts:{...row.counts,keyboard:row.counts.keyboard+1}}:row);
  a.window.receive({...s,history_days:change(s.history_days),days:change(s.days),lifetime:{...s.lifetime,keyboard:s.lifetime.keyboard+1},windows:s.windows.map(w=>({...w,counts:{...w.counts,keyboard:w.counts.keyboard+1}}))});
  assert.equal(a.get('trend-chart').children[0],trend);assert.equal(a.get('donut').children[0],donut);assert.equal(a.get('windows').children[0],windows);
  assert.equal(tooltipWrites,1);
  assert.equal(cells.filter((cell,index)=>cell.title!==oldTitles[index]).length,1);
});

test('calendar rescaling updates all colors while unchanged day descriptions stay intact',()=>{
  const a=app(),s=snapshot();s.history_days.unshift({day:'2026-10-06',counts:{keyboard:90,mouse:10}});a.window.receive(s);
  const cells=()=>a.get('heatmap').children.flatMap(section=>section.children[2].children[1].children).filter(cell=>cell.dataset.day);
  const prior=cells().map(cell=>[cell,cell.title,cell.className]);
  const rows=s.history_days.map((row,index)=>index===s.history_days.length-1?{...row,counts:{keyboard:100000,mouse:0}}:row);
  a.window.receive({...s,history_days:rows});
  assert.ok(prior.some(([cell,,color])=>cell.className!==color));
  for(const [cell,title] of prior)if(cell.dataset.day!==rows.at(-1).day)assert.equal(cell.title,title);
  a.window.receiveView({locale:'de'});assert.ok(cells().some(cell=>cell.title.includes('100.000')));
});

test('only the visible platform page title is exposed to screen readers',()=>{
 for(const platform of ['macos','windows','linux']){
  const a=app(platform);
  assert.equal(a.get('toolbar-title').attributes['aria-hidden'],String(platform!=='macos'));
  assert.equal(a.get('page-title').attributes['aria-hidden'],String(platform==='macos'));
 }
});


test('sidebar toggle and page navigation close the date popup',()=>{
 const a=app();let closed=0;a.context.ClickPulseDatePicker={closeAll(){closed++;}};
 a.get('sidebar-toggle').fire();assert.equal(closed,1);
 a.navigate('settings');assert.equal(closed,2);
});

test('Mini pin labels follow current pin state when the language changes',()=>{
 const a=app();
 for(const pinned of [false,true]){
  a.window.receiveView({pinned});
  for(const locale of ['zh-CN','zh-TW','fr','en']){
   a.window.setInterfaceLanguage(locale,locale);
   const label=messages[locale][pinned?'mini.unpin':'mini.pin'];
   assert.equal(a.get('mini-pin').title,label);assert.equal(a.get('mini-pin').attributes['aria-label'],label);
  }
 }
});

test('hour rollover keeps the selected hour and ignores removed plot events',()=>{
 const a=app(),s=hourlyFixture();a.window.receive(s);
 const old=a.get('hourly-chart').children[0],selected=old.children[10];selected.fire('mouseenter');
 const hour=selected.dataset.hour;
 for(const h of s.hourly){h.start_ms+=3600000;h.end_ms+=3600000;}
 s.now=new Date(Date.parse(s.now)+3600000).toISOString();a.window.receive(s);
 const plot=a.get('hourly-chart').children[0],current=plot.children.find(node=>node.dataset.hour===hour);
 assert.notEqual(plot,old);assert(current);assert.equal(a.get('hourly-detail').textContent,current.attributes['aria-label']);
 const detail=a.get('hourly-detail').textContent;
 old.listeners.mouseleave();old.listeners.focusout({relatedTarget:a.get('pause')});old.children[0].fire('focus');
 assert.equal(a.get('hourly-detail').textContent,detail);
 plot.listeners.mouseleave();assert.equal(a.get('hourly-detail').textContent,'');
});


test('cold heading is visible and early navigation outranks queued restoration',()=>{
  for(const platform of ['macos','windows','linux']){
    const a=app(platform),sent=[];a.window.ipc.postMessage=m=>sent.push(JSON.parse(m));
    assert.equal(a.get('toolbar-title').textContent,messages.en['page.overview']);
    a.window.restoreSessionState({page:'history',settings_section:'app-settings',trend_days:14,calendar_range:'365',history_range:'all',history_page:0,keyboard_view:'layout',sidebar_collapsed:false,scroll_top:400});
    a.navigate('keys');
    assert.equal(sent.at(-1).state.page,'keys');
    a.get('sidebar-toggle').fire();
    a.window.receive(snapshot());
    const saved=sent.filter(m=>m.action==='session_state').at(-1).state;
    assert.equal(saved.page,'keys');assert.equal(saved.sidebar_collapsed,true);
    assert.equal(a.get('toolbar-title').textContent,messages.en['page.heatmap']);
  }
});

test('language and timezone arriving together leave the new zone selected',()=>{
  const a=app();a.window.receive(snapshot());
  a.window.receiveView({locale:'zh-CN',language_preference:'zh-CN',timezone:'UTC',timezones:['UTC','Asia/Shanghai']});
  assert.equal(a.get('timezone-select').value,'UTC');
});
