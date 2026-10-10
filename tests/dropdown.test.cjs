const test=require('node:test');
const assert=require('node:assert/strict');
const D=require('../ui/dropdown.js');

class Element{
  constructor(tag,document){
    this.tag=tag;this.document=document;this.children=[];this.dataset={};this.attributes={};this.listeners={};this.style={};this.hidden=false;this.disabled=false;this.value='';this.classes=new Set();
    this.classList={add:(name)=>this.classes.add(name),toggle:(name,on)=>on?this.classes.add(name):this.classes.delete(name)};
    this.rect={left:30,top:50,bottom:82,width:108,height:32};
  }
  set textContent(value){this.text=value;this.children=[];}
  get textContent(){return this.text||this.children.map(node=>node.textContent).join('');}
  append(...nodes){for(const node of nodes){if(node.parentNode)node.parentNode.children=node.parentNode.children.filter(child=>child!==node);node.parentNode=this;this.children.push(node);}}
  insertBefore(node,anchor){node.parentNode=this;this.children.splice(this.children.indexOf(anchor),0,node);}
  replaceChildren(...nodes){this.children=[];this.text='';this.append(...nodes);}
  get options(){return this.children;}
  get scrollHeight(){return 12+this.children.length*32;}
  setAttribute(name,value){this.attributes[name]=String(value);}
  getAttribute(name){return this.attributes[name]||null;}
  removeAttribute(name){delete this.attributes[name];}
  addEventListener(name,fn){(this.listeners[name]||=[]).push(fn);}
  dispatchEvent(event){event.target??=this;for(const fn of this.listeners[event.type]||[])fn(event);return !event.defaultPrevented;}
  contains(node){return this===node||this.children.some(child=>child.contains(node));}
  focus(){this.document.activeElement=this;this.document.fire('focusin',{target:this});}
  getBoundingClientRect(){return this.rect;}
  scrollIntoView(){this.scrolled=true;}
  fire(type,values={}){const event={type,target:this,preventDefault(){this.defaultPrevented=true;},...values};this.dispatchEvent(event);return event;}
}
function fixture(){
  const listeners={},document={addEventListener(name,fn){(listeners[name]||=[]).push(fn);},fire(name,event){for(const fn of listeners[name]||[])fn(event);},createElement(tag){return new Element(tag,this);},createElementNS(_,tag){return this.createElement(tag);}};
  document.body=document.createElement('body');
  const add=(id,rows,value)=>{const select=document.createElement('select');select.id=id;select.value=value;select.setAttribute('aria-label',id);for(const[v,label]of rows){const option=document.createElement('option');option.value=v;option.textContent=label;select.append(option);}document.body.append(select);return select;};
  const history=add('history-range-select',[['all','All history'],['2026','2026']],'all');
  const themes=add('theme-select',[['blue','Blue'],['teal','Teal'],['violet','Violet'],['graphite','Graphite']],'blue');
  document.querySelectorAll=()=>[history,themes];
  const winListeners={},window={innerWidth:800,innerHeight:500,Event:class{constructor(type,options){this.type=type;Object.assign(this,options);}},addEventListener(name,fn){(winListeners[name]||=[]).push(fn);},fire(name){for(const fn of winListeners[name]||[])fn();}};
  const dropdowns=D.create(document,window);
  return {document,window,history,themes,dropdowns,h:dropdowns.controllers.get(history.id),t:dropdowns.controllers.get(themes.id)};
}
test('menus render inside the App with combobox/listbox semantics and no visible native select',()=>{
  const {history,h,document}=fixture();
  assert.equal(history.hidden,true);assert.equal(h.trigger.getAttribute('role'),'combobox');
  assert.equal(h.menu.getAttribute('role'),'listbox');assert.equal(h.menu.parentNode,document.body);
  h.trigger.fire('click');assert.equal(h.menu.hidden,false);assert.equal(h.trigger.getAttribute('aria-expanded'),'true');
  assert.equal(document.activeElement,h.trigger);
  assert.equal(h.menu.children[0].getAttribute('role'),'option');
});
test('arrows preview options; Escape cancels and Enter commits exactly one change',()=>{
  const {history,h}=fixture();let changes=0;history.addEventListener('change',()=>changes++);
  h.trigger.fire('keydown',{key:'ArrowDown'});h.trigger.fire('keydown',{key:'ArrowDown'});
  assert.equal(history.value,'all');assert.equal(h.trigger.getAttribute('aria-activedescendant'),h.menu.children[1].id);
  h.trigger.fire('keydown',{key:'Escape'});assert.equal(history.value,'all');assert.equal(changes,0);
  h.trigger.fire('keydown',{key:'Enter'});h.trigger.fire('keydown',{key:'End'});h.trigger.fire('keydown',{key:'Enter'});
  assert.equal(history.value,'2026');assert.equal(changes,1);assert.equal(h.menu.hidden,true);
});
test('Tab commits the active option while preserving normal focus navigation',()=>{
  const {themes,t}=fixture();t.trigger.fire('keydown',{key:'Enter'});t.trigger.fire('keydown',{key:'End'});
  const event=t.trigger.fire('keydown',{key:'Tab'});assert.equal(themes.value,'graphite');assert.equal(event.defaultPrevented,undefined);assert.equal(t.menu.hidden,true);
});
test('outside clicks, page scrolling and resize close menus; only one menu opens at once',()=>{
  const {document,window,h,t}=fixture();h.open();t.open();assert.equal(h.menu.hidden,true);
  document.fire('scroll',{target:t.menu});assert.equal(t.menu.hidden,false);
  document.fire('pointerdown',{target:document.body});assert.equal(t.menu.hidden,true);
  h.open();document.fire('scroll',{target:document.body});assert.equal(h.menu.hidden,true);
  h.open();window.fire('resize');assert.equal(h.menu.hidden,true);
});
test('option/locale refresh retains the selected year and updates local labels',()=>{
  const {history,h,document,dropdowns}=fixture();history.value='2026';dropdowns.sync(history);
  const years=[['all','全部历史'],['2026','2026 年'],['2027','2027 年']].map(([value,label])=>{const option=document.createElement('option');option.value=value;option.textContent=label;return option;});
  history.replaceChildren(...years);history.setAttribute('aria-label','历史范围');dropdowns.sync(history);
  assert.equal(h.trigger.children[0].textContent,'2026 年');assert.equal(h.trigger.getAttribute('aria-label'),'历史范围');
  h.open();assert.equal(h.menu.children.length,3);assert.equal(h.menu.children[1].classes.has('selected'),true);
});
test('typing selects matching options while IME composition is ignored',()=>{
  const {themes,t}=fixture();t.trigger.fire('keydown',{key:'v',isComposing:true});assert.equal(t.menu.hidden,true);
  t.trigger.fire('keydown',{key:'v'});t.trigger.fire('keydown',{key:'Enter'});assert.equal(themes.value,'violet');
});
test('menus flip above controls near the bottom and stay inside the viewport',()=>{
  const {h,window}=fixture();h.trigger.rect={left:750,top:465,bottom:497,width:108};h.open();
  assert.ok(parseFloat(h.menu.style.top)<465);assert.ok(parseFloat(h.menu.style.left)+parseFloat(h.menu.style.width)<=window.innerWidth-8);
  assert.equal(h.menu.children[0].scrolled,true);
});

test('either arrow opens at the saved selection before previewing adjacent options',()=>{
  for(const key of ['ArrowUp','ArrowDown']){
    const {themes,t,dropdowns}=fixture();themes.value='violet';dropdowns.sync(themes);
    let changes=0;themes.addEventListener('change',()=>changes++);
    t.trigger.fire('keydown',{key});
    assert.equal(t.trigger.getAttribute('aria-activedescendant'),t.menu.children[2].id);
    assert.equal(themes.value,'violet');assert.equal(changes,0);
    t.trigger.fire('keydown',{key:'Enter'});
    assert.equal(themes.value,'violet');assert.equal(changes,0);
    t.trigger.fire('keydown',{key});t.trigger.fire('keydown',{key});
    const index=key==='ArrowUp'?1:3;
    assert.equal(t.trigger.getAttribute('aria-activedescendant'),t.menu.children[index].id);
    t.trigger.fire('keydown',{key:'Escape'});
    assert.equal(themes.value,'violet');assert.equal(changes,0);
    t.trigger.fire('keydown',{key});t.trigger.fire('keydown',{key});t.trigger.fire('keydown',{key:'Enter'});
    assert.equal(themes.value,key==='ArrowUp'?'teal':'graphite');assert.equal(changes,1);
  }
});
