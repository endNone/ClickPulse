'use strict';
function createClickPulseDropdowns(document,window){
  const controllers=new Map();let opened=null;
  const closeAll=()=>opened?.close();
  for(const select of document.querySelectorAll('select')){
    const wrapper=document.createElement('div'),trigger=document.createElement('button'),label=document.createElement('span'),caret=document.createElementNS('http://www.w3.org/2000/svg','svg'),path=document.createElementNS('http://www.w3.org/2000/svg','path'),menu=document.createElement('div');
    wrapper.className='dropdown';wrapper.dataset.select=select.id;
    trigger.type='button';trigger.id=select.id+'-trigger';trigger.className='button dropdown-trigger';trigger.setAttribute('role','combobox');trigger.setAttribute('aria-haspopup','listbox');trigger.setAttribute('aria-expanded','false');trigger.setAttribute('aria-controls',select.id+'-listbox');
    label.className='dropdown-label';caret.setAttribute('viewBox','0 0 16 16');caret.setAttribute('aria-hidden','true');caret.classList.add('dropdown-caret');path.setAttribute('d','m4 6 4 4 4-4');caret.append(path);trigger.append(label,caret);wrapper.append(trigger);
    menu.id=select.id+'-listbox';menu.className='dropdown-menu';menu.hidden=true;menu.setAttribute('role','listbox');
    select.parentNode.insertBefore(wrapper,select);select.hidden=true;document.body.append(menu);
    let options=[],items=[],signature='',active=-1,buffer='',typedAt=0;
    const current=()=>Math.max(0,options.findIndex(option=>option.value===select.value));
    function mark(){
      items.forEach((item,i)=>{item.classList.toggle('active',i===active);item.classList.toggle('selected',options[i].value===select.value);item.setAttribute('aria-selected',String(i===active));});
      if(opened===controller&&items[active]){trigger.setAttribute('aria-activedescendant',items[active].id);items[active].scrollIntoView({block:'nearest'});}
    }
    function sync(){
      const rows=Array.from(select.options).filter(option=>!option.disabled&&!option.hidden);
      const selected=rows.find(option=>option.value===select.value)||rows[0];
      label.replaceChildren();
      if(selected?.dataset.swatch){const dot=document.createElement('i');dot.className='theme-swatch';dot.style.backgroundColor=selected.dataset.swatch;label.append(dot);}
      const text=document.createElement('span');text.textContent=selected?.textContent||'';label.append(text);
      trigger.disabled=select.disabled||!rows.length;trigger.setAttribute('aria-label',select.getAttribute('aria-label')||text.textContent);menu.setAttribute('aria-label',select.getAttribute('aria-label')||text.textContent);
      const next=JSON.stringify(rows.map(option=>[option.value,option.textContent,option.dataset.swatch]));
      if(next!==signature){
        const activeValue=options[active]?.value;signature=next;options=rows;
        items=options.map((option,i)=>{
          const item=document.createElement('div'),caption=document.createElement('span'),check=document.createElementNS('http://www.w3.org/2000/svg','svg'),stroke=document.createElementNS('http://www.w3.org/2000/svg','path');
          item.id=menu.id+'-'+i;item.className='dropdown-option';item.setAttribute('role','option');item.dataset.value=option.value;
          if(option.dataset.swatch){const dot=document.createElement('i');dot.className='theme-swatch';dot.style.backgroundColor=option.dataset.swatch;item.append(dot);}
          caption.textContent=option.textContent;check.setAttribute('viewBox','0 0 16 16');check.setAttribute('aria-hidden','true');stroke.setAttribute('d','m3 8 3 3 7-7');check.append(stroke);item.append(caption,check);
          item.addEventListener('pointermove',()=>{active=i;mark();});
          item.addEventListener('pointerdown',event=>event.preventDefault());
          item.addEventListener('click',()=>{active=i;commit();trigger.focus();});return item;
        });
        menu.replaceChildren(...items);active=Math.max(0,options.findIndex(option=>option.value===activeValue));
        if(!options.some(option=>option.value===activeValue))active=current();
      }
      if(opened===controller&&!rows.length)close();
      mark();
    }
    function position(){
      const rect=trigger.getBoundingClientRect(),margin=8,width=Math.min(Math.max(rect.width,156),window.innerWidth-margin*2),height=Math.min(menu.scrollHeight,244),below=window.innerHeight-rect.bottom-6-margin,above=rect.top-6-margin;
      const flip=below<height&&above>below,available=Math.max(36,flip?above:below),shown=Math.min(height,available);
      menu.style.width=width+'px';menu.style.maxHeight=available+'px';menu.style.left=Math.max(margin,Math.min(rect.left,window.innerWidth-width-margin))+'px';menu.style.top=(flip?rect.top-shown-6:rect.bottom+6)+'px';
    }
    function open(){
      if(trigger.disabled)return;closeAll();trigger.focus();sync();opened=controller;active=current();menu.hidden=false;trigger.setAttribute('aria-expanded','true');position();mark();
    }
    function close(){menu.hidden=true;trigger.setAttribute('aria-expanded','false');trigger.removeAttribute('aria-activedescendant');if(opened===controller)opened=null;buffer='';}
    function commit(){
      const value=options[active]?.value;close();
      if(value!==undefined&&select.value!==value){select.value=value;select.dispatchEvent(new window.Event('change',{bubbles:true}));}
      sync();
    }
    function keydown(event){
      if(event.isComposing)return;
      const wasOpen=opened===controller,key=event.key;
      if(key==='Tab'){if(wasOpen)commit();return;}
      if(key==='Escape'){if(wasOpen){event.preventDefault();close();}return;}
      if(key==='Enter'||key===' '){event.preventDefault();wasOpen?commit():open();return;}
      if(['ArrowDown','ArrowUp','Home','End','PageDown','PageUp'].includes(key)){
        event.preventDefault();if(!wasOpen)open();
        if(wasOpen)active=Math.min(options.length-1,Math.max(0,active+(key==='ArrowDown'?1:key==='ArrowUp'?-1:key==='PageDown'?10:key==='PageUp'?-10:0)));
        if(key==='Home')active=0;if(key==='End')active=options.length-1;
        if(wasOpen&&event.altKey&&key==='ArrowUp')commit();else mark();return;
      }
      if(key.length===1&&!event.ctrlKey&&!event.metaKey&&!event.altKey){
        event.preventDefault();if(!wasOpen)open();const now=Date.now();buffer=now-typedAt<700?buffer+key:key;typedAt=now;
        const repeated=[...buffer].every(c=>c===buffer[0]),query=(repeated?key:buffer).toLocaleLowerCase(),start=repeated?active+1:active;
        for(let n=0;n<options.length;n++){const i=(start+n)%options.length;if(options[i].textContent.toLocaleLowerCase().startsWith(query)){active=i;mark();break;}}
      }
    }
    const controller={select,wrapper,trigger,menu,sync,open,close};controllers.set(select.id,controller);
    trigger.addEventListener('click',()=>opened===controller?close():open());trigger.addEventListener('keydown',keydown);select.addEventListener('change',sync);sync();
  }
  if(controllers.size){
    document.addEventListener('pointerdown',event=>{if(opened&&!opened.wrapper.contains(event.target)&&!opened.menu.contains(event.target))closeAll();});
    document.addEventListener('focusin',event=>{if(opened&&!opened.wrapper.contains(event.target)&&!opened.menu.contains(event.target))closeAll();});
    document.addEventListener('scroll',event=>{if(opened&&!opened.menu.contains(event.target))closeAll();},true);
    window.addEventListener('resize',closeAll);window.addEventListener('blur',closeAll);
  }
  return {sync:id=>controllers.get(typeof id==='string'?id:id.id)?.sync(),syncAll:()=>controllers.forEach(controller=>controller.sync()),closeAll,controllers};
}
const ClickPulseDropdowns={create:createClickPulseDropdowns};
if(typeof module!=='undefined')module.exports=ClickPulseDropdowns;
