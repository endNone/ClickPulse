'use strict';
function createClickPulseDropdowns(document,window,t){
  const controllers=new Map();let opened=null;
  const closeAll=()=>opened?.close();
  const normalizeSearch=text=>text.toLowerCase().normalize('NFD').replace(/[\u0300-\u036f]/g,'').replace(/[/_]+/g,' ');
  for(const select of document.querySelectorAll('select')){
    const wrapper=document.createElement('div'),trigger=document.createElement('button'),label=document.createElement('span'),caret=document.createElementNS('http://www.w3.org/2000/svg','svg'),path=document.createElementNS('http://www.w3.org/2000/svg','path'),menu=document.createElement('div');
    wrapper.className='dropdown';wrapper.dataset.select=select.id;
    trigger.type='button';trigger.id=select.id+'-trigger';trigger.className='button dropdown-trigger';trigger.setAttribute('role','combobox');trigger.setAttribute('aria-haspopup','listbox');trigger.setAttribute('aria-expanded','false');trigger.setAttribute('aria-controls',select.id+'-listbox');
    label.className='dropdown-label';caret.setAttribute('viewBox','0 0 16 16');caret.setAttribute('aria-hidden','true');caret.classList.add('dropdown-caret');path.setAttribute('d','m4 6 4 4 4-4');caret.append(path);trigger.append(label,caret);wrapper.append(trigger);
    menu.className='dropdown-menu';if(select.closest('.settings-card'))menu.classList.add('settings-menu');menu.hidden=true;
    const searchable=select.id==='timezone-select';
    const list=searchable?document.createElement('div'):menu;
    list.id=select.id+'-listbox';list.setAttribute('role','listbox');
    const search=searchable?document.createElement('input'):null;
    const empty=searchable?document.createElement('div'):null;
    if(search){
      menu.classList.add('searchable-menu');list.className='dropdown-results';
      search.type='search';search.className='dropdown-search';search.autocomplete='off';search.spellcheck=false;
      search.setAttribute('role','combobox');search.setAttribute('aria-autocomplete','list');
      search.setAttribute('aria-controls',list.id);search.setAttribute('aria-expanded','false');
      empty.className='dropdown-empty';empty.setAttribute('role','status');empty.hidden=true;
      menu.append(search,list,empty);
    }
    select.parentNode.insertBefore(wrapper,select);select.hidden=true;document.body.append(menu);
    let options=[],items=[],signature='',active=-1,buffer='',typedAt=0;
    const current=()=>Math.max(0,options.findIndex(option=>option.value===select.value));
    function mark(scroll=true){
      items.forEach((item,i)=>{item.classList.toggle('active',i===active);item.classList.toggle('selected',options[i].value===select.value);item.setAttribute('aria-selected',String(i===active));});
      const owner=search||trigger;
      if(opened===controller&&items[active]){owner.setAttribute('aria-activedescendant',items[active].id);if(scroll)items[active].scrollIntoView({block:'nearest'});}
      else owner.removeAttribute('aria-activedescendant');
    }
    function sync(){
      const available=Array.from(select.options).filter(option=>!option.disabled);
      const visible=available.filter(option=>!option.hidden);
      const selected=available.find(option=>option.value===select.value)||visible[0];
      const terms=search?normalizeSearch(search.value).trim().split(/\s+/).filter(Boolean):[];
      const rows=terms.length?visible.filter(option=>{
        const name=normalizeSearch(option.textContent+' '+option.value);
        return terms.every(term=>name.includes(term));
      }):visible;
      label.replaceChildren();
      if(selected?.dataset.swatch){const dot=document.createElement('i');dot.className='theme-swatch';dot.style.backgroundColor=selected.dataset.swatch;label.append(dot);}
      const text=document.createElement('span');text.textContent=selected?.textContent||'';label.append(text);trigger.title=text.textContent;
      trigger.disabled=select.disabled||!visible.length;trigger.setAttribute('aria-label',select.getAttribute('aria-label')||text.textContent);list.setAttribute('aria-label',select.getAttribute('aria-label')||text.textContent);
      if(search){
        search.placeholder=t('settings.timezone_search');search.setAttribute('aria-label',search.placeholder);
        empty.hidden=rows.length>0;
        const message=empty.hidden?'':t('settings.timezone_no_matches');
        if(empty.textContent!==message)empty.textContent=message;
      }
      const next=JSON.stringify(rows.map(option=>[option.value,option.textContent,option.dataset.swatch]));
      const changed=next!==signature;
      if(changed){
        const activeValue=options[active]?.value;signature=next;options=rows;
        items=options.map((option,i)=>{
          const item=document.createElement('div'),caption=document.createElement('span'),check=document.createElementNS('http://www.w3.org/2000/svg','svg'),stroke=document.createElementNS('http://www.w3.org/2000/svg','path');
          item.id=list.id+'-'+i;item.className='dropdown-option';item.setAttribute('role','option');item.dataset.value=option.value;
          if(option.dataset.swatch){const dot=document.createElement('i');dot.className='theme-swatch';dot.style.backgroundColor=option.dataset.swatch;item.append(dot);}
          caption.textContent=option.textContent;check.setAttribute('viewBox','0 0 16 16');check.setAttribute('aria-hidden','true');stroke.setAttribute('d','m3 8 3 3 7-7');check.append(stroke);item.append(caption,check);
          item.addEventListener('pointermove',()=>{if(active!==i){active=i;mark(false);}});
          item.addEventListener('pointerdown',event=>event.preventDefault());
          item.addEventListener('click',()=>{active=i;commit();trigger.focus();});return item;
        });
        list.replaceChildren(...items);active=Math.max(0,options.findIndex(option=>option.value===activeValue));
        if(!options.some(option=>option.value===activeValue))active=current();
      }
      if(opened===controller&&trigger.disabled)close();
      mark(changed);
      if(changed&&opened===controller&&!search)position();
    }
    function position(){
      const rect=trigger.getBoundingClientRect(),margin=8,gap=6,viewportWidth=Math.max(0,window.innerWidth-margin*2);
      // Measure at the final width, without an old height cap. The outer height
      // includes borders; using scrollHeight would clip even a two-option menu.
      menu.style.width='max-content';menu.style.minWidth=Math.min(Math.max(rect.width,search?224:156),viewportWidth)+'px';
      menu.style.maxWidth=Math.min(Math.max(rect.width,360),viewportWidth)+'px';menu.style.maxHeight='none';
      const natural=menu.getBoundingClientRect(),width=Math.ceil(natural.width),height=Math.min(Math.ceil(natural.height),244);
      const below=window.innerHeight-rect.bottom-gap-margin,above=rect.top-gap-margin,flip=below<height&&above>below;
      const available=Math.max(0,flip?above:below),shown=Math.min(height,available);
      const left=rect.left+rect.width-width,top=flip?rect.top-shown-gap:rect.bottom+gap;
      menu.style.width=width+'px';menu.style.maxHeight=shown+'px';
      menu.style.left=Math.max(margin,Math.min(left,window.innerWidth-width-margin))+'px';
      menu.dataset.placement=flip?'above':'below';
      if(search&&flip){
        menu.style.top='auto';
        menu.style.bottom=Math.max(margin,window.innerHeight-rect.top+gap)+'px';
      }else{
        menu.style.bottom='auto';
        menu.style.top=Math.max(margin,Math.min(top,window.innerHeight-shown-margin))+'px';
      }
      if(search)menu.style.maxHeight=Math.min(244,available)+'px';
    }
    function open(){
      if(trigger.disabled)return;closeAll();trigger.focus();if(search)search.value='';sync();opened=controller;active=current();menu.hidden=false;trigger.setAttribute('aria-expanded','true');position();
      if(search){search.setAttribute('aria-expanded','true');search.focus({preventScroll:true});}mark();
    }
    function close(){menu.hidden=true;trigger.setAttribute('aria-expanded','false');(search||trigger).removeAttribute('aria-activedescendant');if(search)search.setAttribute('aria-expanded','false');if(opened===controller)opened=null;buffer='';}
    function commit(){
      const value=options[active]?.value;close();
      if(value!==undefined&&select.value!==value){select.value=value;select.dispatchEvent(new window.Event('change',{bubbles:true}));}
      sync();
    }
    function keydown(event){
      if(event.isComposing||event.keyCode===229)return;
      const wasOpen=opened===controller,key=event.key;
      if(search&&event.target===search){
        if(key==='Tab'||key==='Escape'){
          if(key==='Escape')event.preventDefault();
          close();trigger.focus();return;
        }
        if(key==='Enter'){
          event.preventDefault();
          if(options[active]){commit();trigger.focus();}return;
        }
        if(!['ArrowDown','ArrowUp','PageDown','PageUp'].includes(key))return;
      }
      if(key==='Tab'){if(wasOpen)commit();return;}
      if(key==='Escape'){if(wasOpen){event.preventDefault();close();}return;}
      if(wasOpen&&event.altKey&&key==='ArrowUp'){event.preventDefault();commit();trigger.focus();return;}
      if(key==='Enter'||(key===' '&&!(wasOpen&&buffer&&Date.now()-typedAt<700))){event.preventDefault();wasOpen?commit():open();return;}
      if(['ArrowDown','ArrowUp','Home','End','PageDown','PageUp'].includes(key)){
        event.preventDefault();if(!wasOpen)open();
        if(wasOpen)active=Math.min(options.length-1,Math.max(0,active+(key==='ArrowDown'?1:key==='ArrowUp'?-1:key==='PageDown'?10:key==='PageUp'?-10:0)));
        if(key==='Home')active=0;if(key==='End')active=options.length-1;
        if(wasOpen&&event.altKey&&key==='ArrowUp')commit();else mark();return;
      }
      if(key.length===1&&!event.ctrlKey&&!event.metaKey&&!event.altKey){
        if(search){
          event.preventDefault();open();search.value=key;sync();return;
        }
        event.preventDefault();if(!wasOpen)open();const now=Date.now();buffer=now-typedAt<700?buffer+key:key;typedAt=now;
        const repeated=[...buffer].every(c=>c===buffer[0]),query=(repeated?key:buffer).toLocaleLowerCase(),start=repeated?active+1:active;
        let match=-1;
        for(const mode of ['startsWith','includes']){
          for(let n=0;n<options.length;n++){const i=(start+n)%options.length;if(options[i].textContent.toLocaleLowerCase()[mode](query)){match=i;break;}}
          if(match>=0)break;
        }
        if(match>=0){active=match;mark();}
      }
    }
    const controller={select,wrapper,trigger,menu,sync,open,close};controllers.set(select.id,controller);
    if(search){search.addEventListener('input',sync);search.addEventListener('keydown',keydown);}
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
