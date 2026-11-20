'use strict';
const ClickPulseDatePicker=(()=>{
  const minimum='0001-01-01',dayMs=86400000;
  let opened=null;
  const date=day=>new Date(day+'T12:00:00Z');
  const key=value=>value.toISOString().slice(0,10);
  function valid(day){
    return typeof day==='string'&&/^\d{4}-\d{2}-\d{2}$/.test(day)&&day>=minimum&&Number.isFinite(date(day).getTime())&&key(date(day))===day;
  }
  function shiftMonth(day,offset){
    const value=date(day),wanted=value.getUTCDate();
    value.setUTCDate(1);value.setUTCMonth(value.getUTCMonth()+offset);
    if(value.getUTCFullYear()<1)return minimum;
    if(value.getUTCFullYear()>9999)return '9999-12-31';
    const end=new Date(value);end.setUTCMonth(end.getUTCMonth()+1);end.setUTCDate(0);
    value.setUTCDate(Math.min(wanted,end.getUTCDate()));return key(value);
  }
  function shiftDay(day,offset){
    return key(new Date(Math.max(date(minimum).getTime(),Math.min(date('9999-12-31').getTime(),date(day).getTime()+offset*dayMs))));
  }
  function monthDays(month){
    const first=date(month+'-01'),start=first.getTime()-((first.getUTCDay()+6)%7)*dayMs;
    return Array.from({length:42},(_,i)=>{
      const value=new Date(start+i*dayMs),year=value.getUTCFullYear();
      return year<1||year>9999?null:key(value);
    });
  }
  function create({document,window,I,input}){
    const trigger=document.getElementById(input.id+'-trigger'),caption=document.getElementById(input.id+'-value');
    const panel=document.createElement('div'),header=document.createElement('div'),monthLabel=document.createElement('strong'),arrows=document.createElement('div'),grid=document.createElement('div'),footer=document.createElement('div'),todayButton=document.createElement('button');
    panel.id=input.id+'-calendar';panel.className='date-picker-panel';panel.hidden=true;
    panel.setAttribute('role','dialog');panel.setAttribute('aria-labelledby',input.id+'-label');
    header.className='date-picker-header';monthLabel.id=input.id+'-month';monthLabel.setAttribute('aria-live','polite');
    arrows.className='date-picker-arrows';grid.className='date-picker-grid';grid.setAttribute('role','grid');grid.setAttribute('aria-labelledby',monthLabel.id);
    footer.className='date-picker-footer';todayButton.type='button';todayButton.className='date-picker-today';
    function arrow(direction){
      const button=document.createElement('button'),icon=document.createElementNS('http://www.w3.org/2000/svg','svg'),path=document.createElementNS('http://www.w3.org/2000/svg','path');
      button.type='button';button.className='date-picker-arrow';icon.setAttribute('viewBox','0 0 16 16');icon.setAttribute('aria-hidden','true');path.setAttribute('d',direction<0?'m10 4-4 4 4 4':'m6 4 4 4-4 4');icon.append(path);button.append(icon);
      button.addEventListener('click',()=>{focused=clamp(shiftMonth(focused,direction));month=focused.slice(0,7);render();position();});return button;
    }
    const previous=arrow(-1),next=arrow(1);arrows.append(previous,next);header.append(monthLabel,arrows);footer.append(todayButton);panel.append(header,grid,footer);document.body.append(panel);
    trigger.setAttribute('aria-controls',panel.id);trigger.setAttribute('aria-expanded','false');
    let month='',focused='',signature='';
    const bound=()=>valid(input.max)?input.max:'';
    const lower=()=>valid(input.min)?input.min:minimum;
    const today=()=>input.dataset.today||bound();
    const clamp=day=>day<lower()?lower():day>bound()?bound():day;
    const isOpen=()=>opened===controller;
    function position(){
      const rect=trigger.getBoundingClientRect(),margin=8,gap=8,width=Math.min(296,window.innerWidth-margin*2);
      panel.style.width=width+'px';panel.style.maxHeight=Math.max(40,window.innerHeight-margin*2)+'px';
      const height=panel.getBoundingClientRect().height,below=window.innerHeight-rect.bottom-gap-margin,above=rect.top-gap-margin;
      const top=below>=height?rect.bottom+gap:above>=height?rect.top-height-gap:Math.max(margin,Math.min(rect.bottom+gap,window.innerHeight-height-margin));
      panel.style.left=Math.max(margin,Math.min(rect.left,window.innerWidth-width-margin))+'px';panel.style.top=top+'px';
    }
    function render(focus=false){
      monthLabel.textContent=I.day(month+'-01',{year:'numeric',month:'long'});
      previous.setAttribute('aria-label',I.t('date.previous_month'));next.setAttribute('aria-label',I.t('date.next_month'));
      previous.disabled=month<=lower().slice(0,7);todayButton.disabled=today()>bound()||today()<lower();next.disabled=month>=bound().slice(0,7);todayButton.textContent=I.t('date.today');
      const weekdays=document.createElement('div');weekdays.className='date-picker-week';weekdays.setAttribute('role','row');
      for(const name of I.weekdays()){
        const el=document.createElement('span');el.className='date-picker-weekday';el.setAttribute('role','columnheader');el.textContent=name;weekdays.append(el);
      }
      const cells=monthDays(month),rows=[weekdays];let focusButton;
      for(let row=0;row<6;row++){
        const week=document.createElement('div');week.className='date-picker-week';week.setAttribute('role','row');
        for(const day of cells.slice(row*7,row*7+7)){
          const cell=document.createElement('div');cell.setAttribute('role','gridcell');week.append(cell);if(!day)continue;
          const button=document.createElement('button'),selected=day===input.value;
          button.type='button';button.className='date-picker-day';button.dataset.day=day;button.textContent=I.number(Number(day.slice(-2)));
          button.disabled=day>bound()||day<lower();button.tabIndex=day===focused&&!button.disabled?0:-1;
          button.classList.toggle('outside-month',day.slice(0,7)!==month);button.classList.toggle('selected',selected);
          cell.setAttribute('aria-selected',String(selected));cell.setAttribute('aria-disabled',String(button.disabled));button.setAttribute('aria-label',I.day(day,{weekday:'long',year:'numeric',month:'long',day:'numeric'}));
          if(day===today())button.setAttribute('aria-current','date');
          button.addEventListener('click',()=>choose(day));cell.append(button);if(day===focused)focusButton=button;
        }
        rows.push(week);
      }
      grid.replaceChildren(...rows);if(focus){focusButton?.focus({preventScroll:true});focusButton?.scrollIntoView({block:'nearest'});}
    }
    function sync(){
      const max=bound(),selected=valid(input.value)?input.value:max;
      caption.textContent=selected?I.day(selected,{year:'numeric',month:input.dataset.numericDate==='true'?'numeric':'short',day:'numeric'}):I.t('common.loading');
      trigger.disabled=!max||input.disabled;
      const current=JSON.stringify([selected,lower(),max,today(),I.locale]);
      if(current===signature)return;signature=current;
      if(isOpen()){
        if(trigger.disabled){close();return;}
        // Live snapshots must not reset the month the user is browsing.
        focused=clamp(focused||selected);month=focused.slice(0,7);
        render(panel.contains(document.activeElement)&&document.activeElement?.dataset.day!==undefined);position();
      }
    }
    function open(){
      sync();if(trigger.disabled)return;
      if(opened)opened.close();opened=controller;focused=clamp(valid(input.value)?input.value:bound());month=focused.slice(0,7);
      panel.hidden=false;trigger.setAttribute('aria-expanded','true');render();position();
      panel.querySelector('[tabindex="0"]')?.focus({preventScroll:true});
    }
    function close(restore=false){
      panel.hidden=true;trigger.setAttribute('aria-expanded','false');if(isOpen())opened=null;if(restore)trigger.focus({preventScroll:true});
    }
    function choose(day){
      if(!valid(day)||day>bound()||day<lower())return;
      input.value=day;close(true);input.dispatchEvent(new window.Event('change',{bubbles:true}));sync();
    }
    const controller={sync,open,close,panel,trigger};
    trigger.addEventListener('click',()=>isOpen()?close():open());
    trigger.addEventListener('keydown',event=>{if(['ArrowDown','ArrowUp'].includes(event.key)){event.preventDefault();open();}});
    todayButton.addEventListener('click',()=>choose(today()));
    panel.addEventListener('keydown',event=>{
      if(event.isComposing)return;
      if(event.key==='Escape'){event.preventDefault();event.stopPropagation();close(true);return;}
      if(event.key==='Tab'){
        const buttons=Array.from(panel.querySelectorAll('button')).filter(button=>!button.disabled&&button.tabIndex!==-1);
        if(buttons.length){event.preventDefault();const index=buttons.indexOf(document.activeElement);buttons[(index+(event.shiftKey?-1:1)+buttons.length)%buttons.length].focus();}
        return;
      }
      const day=event.target.dataset.day;if(!day)return;
      let target;
      const step={ArrowLeft:-1,ArrowRight:1,ArrowUp:-7,ArrowDown:7}[event.key];
      if(step)target=shiftDay(day,step);
      else if(event.key==='Home'||event.key==='End'){
        const weekday=(date(day).getUTCDay()+6)%7;target=shiftDay(day,event.key==='Home'?-weekday:6-weekday);
      }else if(event.key==='PageUp'||event.key==='PageDown')target=shiftMonth(day,(event.key==='PageUp'?-1:1)*(event.shiftKey?12:1));
      if(target){event.preventDefault();focused=clamp(target);month=focused.slice(0,7);render(true);position();}
    });
    for(const type of ['pointerdown','focusin'])document.addEventListener(type,event=>{if(isOpen()&&!panel.contains(event.target)&&!trigger.contains(event.target))close();});
    document.addEventListener('scroll',event=>{if(isOpen()&&!panel.contains(event.target))close();},true);
    window.addEventListener('resize',()=>close());window.addEventListener('blur',()=>close());
    input.addEventListener('change',sync);sync();return controller;
  }
  return {create,valid,shiftMonth,monthDays,closeAll:()=>opened?.close()};
})();
if(typeof module!=='undefined')module.exports=ClickPulseDatePicker;
