const test=require('node:test'),assert=require('node:assert/strict');
const Picker=require('../ui/date-picker.js'),i18n=require('../ui/i18n.js'),messages=require('../ui/locales.json');

function harness({value='2026-11-18',max='2026-11-18',locale='en',zone='UTC'}={}){
  Picker.closeAll();let document;
  class Element{
    constructor(){this.children=[];this.handlers={};this.attributes={};this.dataset={};this.style={};this.hidden=false;this.disabled=false;this.tabIndex=-1;this.classList={toggle(){}};}
    append(...nodes){this.children.push(...nodes);}
    replaceChildren(...nodes){this.children=nodes;}
    setAttribute(k,v){this.attributes[k]=v;}
    addEventListener(k,f){(this.handlers[k]??=[]).push(f);}
    dispatchEvent(event){event.target??=this;for(const fn of this.handlers[event.type]||[])fn(event);}
    contains(node){return this===node||this.children.some(child=>child.contains(node));}
    querySelector(){return this.children.flatMap(child=>[child,...child.descendants()]).find(child=>child.tabIndex===0);}
    descendants(){return this.children.flatMap(child=>[child,...child.descendants()]);}
    focus(){document.activeElement=this;}
    scrollIntoView(){}
    getBoundingClientRect(){return this.id==='share-date-trigger'?{left:260,top:120,bottom:160,width:240}:{height:352};}
    fire(type,extra={}){if(type==='click'&&this.disabled)return;this.dispatchEvent({type,preventDefault(){},stopPropagation(){},...extra});}
  }
  const nodes=new Map();const get=id=>{if(!nodes.has(id)){const el=new Element();el.id=id;nodes.set(id,el);}return nodes.get(id);};
  document={body:new Element(),activeElement:null,createElement:()=>new Element(),createElementNS:()=>new Element(),getElementById:get,handlers:{},addEventListener(k,f){(this.handlers[k]??=[]).push(f);},fire(type,target){for(const f of this.handlers[type]||[])f({type,target});}};
  const window={innerWidth:900,innerHeight:680,handlers:{},addEventListener(k,f){this.handlers[k]=f;},Event:class {constructor(type){this.type=type;}}};
  const input=get('share-date');input.value=value;input.max=max;
  const I=i18n(messages,locale);I.setTimeZone(zone);
  const picker=Picker.create({document,window,I,input}),all=()=>picker.panel.descendants();
  let changed=0;input.addEventListener('change',()=>changed++);
  return {picker,input,I,document,window,get,changed:()=>changed,day:key=>all().find(el=>el.dataset.day===key),previous:()=>all().find(el=>el.attributes['aria-label']===I.t('date.previous_month')),next:()=>all().find(el=>el.attributes['aria-label']===I.t('date.next_month')),today:()=>all().find(el=>el.className==='date-picker-today'),key:(day,key,extra={})=>picker.panel.fire('keydown',{target:day,key,...extra})};
}

test('calendar uses date-only arithmetic through leap years and month boundaries',()=>{
  for(const day of ['2024-02-29','0001-01-01','9999-12-31'])assert(Picker.valid(day));
  for(const day of ['2026-02-29','2026-02-30','2026-13-01','2026-1-02','0000-12-31','x'])assert(!Picker.valid(day));
  assert.equal(Picker.shiftMonth('2024-01-31',1),'2024-02-29');
  assert.equal(Picker.shiftMonth('2026-01-31',1),'2026-02-28');
  assert.equal(Picker.shiftMonth('2026-12-31',1),'2027-01-31');
  assert.equal(Picker.shiftMonth('2026-03-31',-1),'2026-02-28');
  assert.equal(Picker.shiftMonth('0001-01-01',-1),'0001-01-01');
  assert.equal(Picker.shiftMonth('9999-12-31',1),'9999-12-31');
  const days=Picker.monthDays('2026-11');assert.equal(days.length,42);assert.equal(days[0],'2026-10-26');assert.equal(days[6],'2026-11-01');assert.equal(days[41],'2026-12-06');
});

test('picker selects past dates, keeps future days disabled and restores focus',()=>{
  const a=harness();a.picker.open();assert.equal(a.next().disabled,true);
  assert.equal(a.document.activeElement.dataset.day,'2026-11-18');
  a.day('2026-11-19').fire('click');assert.equal(a.changed(),0);assert.equal(a.input.value,'2026-11-18');
  a.day('2026-11-09').fire('click');assert.equal(a.changed(),1);assert.equal(a.input.value,'2026-11-09');assert.equal(a.picker.panel.hidden,true);assert.equal(a.document.activeElement,a.picker.trigger);
  assert.equal(a.get('share-date-value').textContent,'Nov 9, 2026');
});

test('Today is the supplied reporting-zone date, not the computer UTC date',()=>{
  const a=harness({value:'2026-11-16',max:'2026-11-19',zone:'America/Los_Angeles'});
  a.picker.open();a.today().fire('click');assert.equal(a.input.value,'2026-11-19');assert.equal(a.changed(),1);
  assert.equal(a.get('share-date-value').textContent,'Nov 19, 2026');
});

test('live sync preserves a browsed month and changing locale does not change selection',()=>{
  const a=harness();a.picker.open();a.previous().fire('click');
  assert(a.day('2026-10-18'));a.picker.sync();assert(a.day('2026-10-18'));
  a.input.max='2026-11-19';a.picker.sync();assert(a.day('2026-10-18'));
  a.I.setLocale('zh-CN');a.picker.sync();assert(a.day('2026-10-18'));assert.equal(a.input.value,'2026-11-18');assert.equal(a.changed(),0);
  assert.equal(a.get('share-date-value').textContent,'2026年11月18日');
});

test('keyboard browsing moves focus without committing; Escape preserves the selected day',()=>{
  const a=harness();a.picker.open();a.key(a.day('2026-11-18'),'ArrowLeft');assert.equal(a.document.activeElement.dataset.day,'2026-11-17');
  a.key(a.day('2026-11-17'),'Home');assert.equal(a.document.activeElement.dataset.day,'2026-11-16');
  a.key(a.day('2026-11-16'),'End');assert.equal(a.document.activeElement.dataset.day,'2026-11-18');
  a.key(a.day('2026-11-18'),'PageUp');assert.equal(a.document.activeElement.dataset.day,'2026-10-18');
  a.key(a.day('2026-10-18'),'PageUp',{shiftKey:true});assert.equal(a.document.activeElement.dataset.day,'2025-10-18');
  a.key(a.day('2025-10-18'),'Escape');assert.equal(a.changed(),0);assert.equal(a.input.value,'2026-11-18');assert.equal(a.picker.panel.hidden,true);assert.equal(a.document.activeElement,a.picker.trigger);
});

test('popover stays inside a small viewport and closes on outside focus or window changes',()=>{
  const a=harness();a.window.innerWidth=280;a.window.innerHeight=400;a.picker.open();
  assert.equal(a.picker.panel.style.left,'8px');assert.equal(a.picker.panel.style.width,'264px');assert.equal(a.picker.panel.style.top,'40px');
  a.document.fire('pointerdown',a.day('2026-11-18'));assert.equal(a.picker.panel.hidden,false);
  a.document.fire('focusin',a.document.body);assert.equal(a.picker.panel.hidden,true);
  a.picker.open();a.window.handlers.blur();assert.equal(a.picker.panel.hidden,true);
  a.picker.open();Picker.closeAll();assert.equal(a.picker.panel.hidden,true);
});

test('new day and locale changes use all eight complete catalogs',()=>{
  for(const locale of Object.keys(messages)){
    const a=harness({locale});a.picker.open();
    assert.notEqual(a.today().textContent,'date.today');assert(a.previous());assert(a.next());
    a.input.max='2026-12-01';a.picker.sync();assert.equal(a.next().disabled,false);
    a.next().fire('click');assert.equal(a.next().disabled,true);assert.equal(a.input.value,'2026-11-18');
  }
});
