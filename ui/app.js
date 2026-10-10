'use strict';
const $=id=>document.getElementById(id);
const I=ClickPulseI18n;
const T=ClickPulseThemes;
let currentTheme=T.get(window.clickpulseTheme).name;
T.apply(document.documentElement,currentTheme);
const t=(key,values={})=>I.t(key,values);
const n=value=>I.number(value);
I.apply(document);
$('theme-select').value=currentTheme;
for(const option of $('theme-select').options)option.dataset.swatch=T.get(option.value).accent;
const Dropdowns=ClickPulseDropdowns.create(document,window);
const total=c=>(c?.keyboard||0)+(c?.mouse||0);
const send=(action,values={})=>window.ipc?.postMessage(JSON.stringify({action,...values}));
let data=null,page='overview',range=14,toastTimer;
let calendarRange='all',historyRange='all',historyPage=0,calendarLayout='',historySignature='';
let calendarCells=new Map();
const H=ClickPulseHistory;
const titles={overview:'page.overview',history:'page.history',keys:'page.keys',settings:'page.settings'};
function updateHeading(){
  $('page-title').textContent=t(titles[page]);$('page-description').textContent=t(titles[page]+'.description');
}
window.setInterfaceLanguage=(locale,preference)=>{
  $('language-select').value=preference||'system';
  Dropdowns.sync('language-select');
  if(I.locale===locale)return;
  I.setLocale(locale);I.apply(document);updateHeading();
  Dropdowns.syncAll();
  calendarLayout='';historySignature='';
  for(const id of ['calendar-range-select','history-range-select'])delete $(id).dataset.options;
};
$('language-select').addEventListener('change',event=>send('language',{language:event.target.value}));
$('theme-select').addEventListener('change',event=>send('theme',{theme:event.target.value}));
const K=ClickPulseKeyboard;
const onWindows=window.clickpulsePlatform==='windows';
const keyboardCells=new Map();
let keyboardLayoutSignature='';
let keyboardView='layout',keyboardDistribution=null;
const keyboardBlocks=new Map();
function updateKeyboardView(){
  const blocks=keyboardView==='treemap';
  $('keyboard-layout').parentElement.hidden=blocks;
  $('keyboard-treemap').hidden=!blocks;
  $('keyboard-view-toggle').setAttribute('aria-pressed',String(blocks));
  $('keyboard-blocks-icon').hidden=blocks;$('keyboard-layout-icon').hidden=!blocks;
  for(const [id,key]of [['keyboard-view-label',blocks?'keys.show_layout':'keys.show_treemap'],['keyboard-description',blocks?'keys.treemap_description':'keys.description']]){
    $(id).dataset.i18n=key;$(id).textContent=t(key);
  }
}
$('keyboard-view-toggle').addEventListener('click',()=>{
  keyboardView=keyboardView==='layout'?'treemap':'layout';updateKeyboardView();
  if(data)renderKeyboard();
});
if(typeof ResizeObserver==='function')new ResizeObserver(()=>{
  if(page==='keys'&&keyboardView==='treemap'&&!window.presentationView?.compact&&keyboardDistribution)renderKeyboardTreemap(keyboardDistribution);
}).observe($('keyboard-treemap'));
const renderSignatures=new Map();
window.setInterfaceTheme=name=>{
  const next=T.get(name).name;$('theme-select').value=next;Dropdowns.sync('theme-select');
  if(next===currentTheme)return;
  currentTheme=next;T.apply(document.documentElement,currentTheme);
  renderSignatures.clear();historySignature='';
};
function renderChanged(name,values,render){
  const signature=JSON.stringify([I.locale,currentTheme,values]);
  if(renderSignatures.get(name)===signature)return;
  render();renderSignatures.set(name,signature);
}
document.querySelectorAll('nav button').forEach(button=>button.addEventListener('click',()=>{
  Dropdowns.closeAll();
  page=button.dataset.page;
  document.querySelectorAll('nav button').forEach(b=>b.classList.toggle('selected',b===button));
  document.querySelectorAll('.page').forEach(el=>el.hidden=el.id!==page);
  updateHeading();
  document.querySelector('main').scrollTop=0;
  if(data)window.receive(data);
}));
document.querySelectorAll('[data-range]').forEach(button=>button.addEventListener('click',()=>{
  range=Number(button.dataset.range);document.querySelectorAll('[data-range]').forEach(b=>b.classList.toggle('selected',b===button));if(data)renderTrend($('trend-chart'),data.days.slice(-range));
}));
for(const id of ['pause','settings-pause'])$(id).addEventListener('click',()=>send('pause'));
for(const id of ['export','history-export'])$(id).addEventListener('click',()=>send('export'));
for(const id of ['permission','settings-permission'])$(id).addEventListener('click',()=>send('permission'));
$('reveal').addEventListener('click',()=>send('reveal'));$('quit').addEventListener('click',()=>send('quit'));
$('calendar-range-select').addEventListener('change',event=>{calendarRange=event.target.value;if(data)renderCalendar()});
$('history-range-select').addEventListener('change',event=>{historyRange=event.target.value;historyPage=0;if(data)renderHistory()});
$('history-prev').addEventListener('click',()=>{historyPage--;if(data)renderHistory()});
$('history-next').addEventListener('click',()=>{historyPage++;if(data)renderHistory()});
window.toast=message=>{$('toast').textContent=message;$('toast').hidden=false;clearTimeout(toastTimer);toastTimer=setTimeout(()=>$('toast').hidden=true,4500)};
window.failure=message=>{
  $('error-banner').hidden=false;$('error-banner').textContent=message;$('status').className='status error';$('status').lastElementChild.textContent=t('status.unavailable');
  if(data){
    data={...data,capture:{...data.capture,listening:false,error:message,error_code:'error.stats'}};
    window.presentationStats?.(data);
  }else{
    $('mini-status').textContent=t('status.unavailable');$('mini-status').classList.toggle('error',true);
  }
};
window.receive=snapshot=>{
  data=snapshot;
  window.presentationStats?.(data);
  if(window.presentationView?.compact)return;
  const now=new Date(data.now);$('date').textContent=I.date(now,{year:'numeric',month:'long',day:'numeric',weekday:'long'});
  $('refresh-time').textContent=t('common.updated',{time:I.date(now,{hour:'2-digit',minute:'2-digit',second:'2-digit',hour12:false})});
  const demo=!!data.demo;$('demo-badge').hidden=!demo;
  let state=demo?'':data.capture.error?'error':data.paused?'paused':data.source_mode||data.capture.listening?'':'error';
  $('status').className='status '+state;$('status').lastElementChild.textContent=t(demo?'status.demo':data.capture.error?(data.source_mode?'status.read_error':'status.recording_error'):data.paused?'status.paused':data.source_mode?'status.updating':data.capture.listening?'status.recording':'status.permission');
  $('permission-banner').hidden=!data.capture.permission_needed;
  $('error-banner').hidden=!data.capture.error;
  if(data.capture.error)$('error-banner').textContent=t(data.capture.error_code||'error.recording');
  $('pause').textContent=t(data.source_mode?(data.paused?'action.resume_refresh':'action.pause_refresh'):(data.paused?'action.resume':'action.pause'));
  if(page==='settings')renderSettings();
  if(page==='overview'){
    for(const key of ['today','week','lifetime']){$(key).textContent=n(total(data[key]));$(key+'-detail').textContent=`${t('common.keyboard')} ${n(data[key].keyboard)} / ${t('common.mouse')} ${n(data[key].mouse)}`;}
    $('legacy-note').textContent=t(data.source_mode?'metric.readonly':data.legacy_imported?'metric.imported':'metric.all');
    renderChanged('windows',data.windows,()=>$('windows').replaceChildren(...data.windows.map(w=>{
      const node=document.createElement('div');node.className='window';const count=document.createElement('strong'),label=document.createElement('span');count.textContent=(w.approximate?'≈ ':'')+n(total(w.counts));label.textContent=t('window.'+w.seconds);node.append(count,label);return node;
    })));
    renderChanged('trend',[range,data.days.slice(-range)],()=>renderTrend($('trend-chart'),data.days.slice(-range)));
    renderChanged('donut',data.lifetime,()=>renderDonut(data.lifetime));
    updateRangeOptions($('calendar-range-select'),true);calendarRange=$('calendar-range-select').value;
    renderChanged('calendar',[data.now.split('T')[0],calendarRange,data.history_days],renderCalendar);
  }else if(page==='history'){
    updateRangeOptions($('history-range-select'),false);historyRange=$('history-range-select').value;
    renderHistory();
  }else if(page==='keys'){
    renderChanged('keyboard',data.keyboard_stats||data.top_keys,renderKeyboard);
  }
};
function renderSettings(){
  $('settings-pause').textContent=t(data.source_mode?(data.paused?'action.resume_refresh':'action.pause_refresh'):(data.paused?'action.resume':'action.pause'));
  $('recording-description').textContent=t(data.source_mode?'settings.source_description':'settings.capture_description');
  $('permission-description').textContent=t(onWindows?(data.capture.listening?'settings.windows_capture':'status.unavailable'):data.source_mode && !data.setup_pending?'settings.source_permission':data.capture.listening?'settings.authorized':'settings.permission_description');
  $('settings-permission').hidden=onWindows || (!!data.source_mode && !data.setup_pending);
  $('capture-settings').hidden=!!data.source_mode && !data.setup_pending;
  $('handoff-settings').hidden=!data.setup_pending;
  $('takeover').disabled=!data.capture.listening || data.capture.permission_needed;
  $('handoff-description').textContent=t(data.capture.listening?'settings.takeover_ready':'settings.takeover_wait');
  $('autostart-row').hidden=data.auto_start_available!==true;
  $('autostart').textContent=t(data.auto_start?'action.disable':'action.enable');
  $('import-description').textContent=t(data.source_mode?'settings.history_source':data.legacy_imported?'settings.history_imported':'settings.history_native');
  $('app-description').textContent=t(data.source_mode?'settings.source_app_description':'settings.app_description');
  $('db-path').textContent=data.demo?t('demo.storage'):data.database_path;
  const storageMode=data.demo?'demo':data.database_mode||'custom';
  const storageDescriptions={default:'settings.storage_default',custom:'settings.storage_custom',source:'settings.storage_source',demo:'settings.storage_demo'};
  $('db-description').textContent=t(storageDescriptions[storageMode]||storageDescriptions.custom);
}
const svgNode=(tag,attrs={})=>{const node=document.createElementNS('http://www.w3.org/2000/svg',tag);for(const[key,value]of Object.entries(attrs))node.setAttribute(key,String(value));return node};
const svgText=(text,x,y,attrs={})=>{const el=svgNode('text',{x,y,...attrs});el.textContent=text;return el};
function renderTrend(container,days){
  const svg=svgNode('svg',{viewBox:'0 0 600 132',preserveAspectRatio:'none',class:'trend-plot',role:'img','aria-label':t('chart.aria')});
  const {ceiling,ticks}=H.countScale(Math.max(0,...days.map(d=>total(d.counts))));
  const yAxis=document.createElement('div'),xAxis=document.createElement('div');
  yAxis.className='trend-y-axis';xAxis.className='trend-x-axis';
  container.style.setProperty('--trend-gutter',Math.max(56,...ticks.map(value=>n(value).length*6+12))+'px');
  for(const value of ticks){
    const fraction=1-value/ceiling,y=fraction*132;
    svg.append(svgNode('line',{x1:0,x2:600,y1:y,y2:y,stroke:'var(--line)','stroke-width':1,'vector-effect':'non-scaling-stroke'}));
    const label=document.createElement('span');label.textContent=n(value);label.style.top=fraction*100+'%';yAxis.append(label);
  }
  const width=600/Math.max(1,days.length),barWidth=Math.min(26,width*.6),palette=T.get(currentTheme);
  days.forEach((d,i)=>{const x=i*width+(width-barWidth)/2,kh=d.counts.keyboard/ceiling*132,mh=d.counts.mouse/ceiling*132;
    const period=d.start?I.day(d.start)+' — '+I.day(d.end):I.day(d.day);
    for(const[color,y,height]of [[palette.accent,132-kh,kh],[palette.mouse,132-kh-mh,mh]]){const bar=svgNode('rect',{x,y,width:barWidth,height,rx:1.4,fill:color});const title=svgNode('title');title.textContent=t('chart.tooltip',{period,keyboard:n(d.counts.keyboard),mouse:n(d.counts.mouse)});bar.append(title);svg.append(bar)}
  });
  const multipleYears=new Set(days.map(d=>(d.start||d.day).slice(0,4))).size>1;
  for(const i of H.trendLabelIndices(days.length)){
    const d=days[i],date=d.start||d.day,monthly=d.grouping==='month',label=document.createElement('span');
    label.className='trend-date'+(days.length>1&&i===0?' first':days.length>1&&i===days.length-1?' last':'');
    label.style.left=(i+.5)/days.length*100+'%';
    const primary=document.createElement('span');primary.textContent=I.day(date,monthly?{month:'short'}:{month:'short',day:'numeric'});label.append(primary);
    if(monthly||multipleYears){const year=document.createElement('small');year.textContent=I.day(date,{year:'numeric'});label.append(year)}
    xAxis.append(label);
  }
  container.replaceChildren(svg,yAxis,xAxis);
  if(!days.some(d=>total(d.counts)>0)){const label=document.createElement('div');label.className='empty-label';label.textContent=t('common.empty');container.append(label)}
}
function renderDonut(counts){
  const all=total(counts),ratio=all?counts.keyboard/all:0,circ=2*Math.PI*51,svg=svgNode('svg',{viewBox:'0 0 170 170',role:'img','aria-label':t('chart.mix_aria',{keyboard:I.percent(ratio,0),mouse:I.percent(1-ratio,0)})});
  const palette=T.get(currentTheme);
  svg.append(svgNode('circle',{cx:85,cy:85,r:51,fill:'none',stroke:all?palette.mouse:'#e9eff9','stroke-width':13}));
  if(all)svg.append(svgNode('circle',{cx:85,cy:85,r:51,fill:'none',stroke:palette.accent,'stroke-width':13,'stroke-dasharray':`${ratio*circ} ${circ}` ,transform:'rotate(-90 85 85)'}));
  svg.append(svgText(all?I.percent(ratio,0):'—',85,85,{'text-anchor':'middle',style:'font-size:26px;fill:var(--accent-strong);letter-spacing:-1px'}),svgText(t('chart.keyboard_share'),85,106,{'text-anchor':'middle',style:'font-size:10px;fill:#7c94b5'}));
  $('donut').replaceChildren(svg);$('composition-legend').replaceChildren(...[[t('common.keyboard'),counts.keyboard,'keyboard-dot'],[t('common.mouse'),counts.mouse,'mouse-dot']].map(([label,value,cls])=>{const node=document.createElement('div'),count=document.createElement('strong'),dot=document.createElement('i');count.textContent=n(value);dot.className=cls;dot.style.marginLeft='0';node.append(count,dot,' '+label);return node}));
}
function updateRangeOptions(select,includeRecent){
  const options=[['all',t('common.all_history')],...H.years(data.history_days).map(year=>[year,t('common.year',{year})])];
  if(includeRecent)options.push(['90',t('common.recent90')]);
  const signature=I.locale+':'+options.map(option=>option[0]).join(',');
  if(select.dataset.options===signature)return;
  const selected=select.value;
  select.replaceChildren(...options.map(([value,label])=>{const option=document.createElement('option');option.value=value;option.textContent=label;return option}));
  select.value=options.some(option=>option[0]===selected)?selected:'all';select.dataset.options=signature;
  Dropdowns.sync(select);
}
function renderCalendar(){
  const today=data.now.split('T')[0],bounds=H.bounds(data.history_days,today,calendarRange);
  const days=H.calendarDays(data.history_days,bounds.start,bounds.end);
  const sections=H.sections(days),layout=I.locale+':'+calendarRange+':'+bounds.start+':'+bounds.end;
  if(calendarLayout!==layout){
    calendarCells=new Map();
    $('heatmap').replaceChildren(...sections.map(section=>{
      const block=document.createElement('div');block.className='calendar-year';
      const label=document.createElement('div');label.className='calendar-year-label';
      const year=document.createElement('strong');year.textContent=t('common.year',{year:section.year});
      const period=document.createElement('span');period.textContent=I.day(section.days[0].day,{month:'short',day:'numeric'})+' — '+I.day(section.days[section.days.length-1].day,{month:'short',day:'numeric'});label.append(year,period);
      const months=document.createElement('div');months.className='calendar-months';
      const body=document.createElement('div');body.className='calendar-body';
      const weekdays=document.createElement('div');weekdays.className='calendar-weekdays';
      for(const text of I.weekdays().map((day,index)=>index%2===0?day:'')){const el=document.createElement('span');el.textContent=text;weekdays.append(el)}
      const grid=document.createElement('div');grid.className='heatmap';
      const first=section.days[0].day,padding=H.weekday(first);
      for(let i=0;i<padding;i++){const cell=document.createElement('div');cell.className='heat-cell blank';grid.append(cell)}
      let previousMonth='';
      for(const d of section.days){
        const month=d.day.slice(5,7);
        if(month!==previousMonth){const el=document.createElement('span');el.textContent=I.month(month);el.style.left=Math.floor((H.difference(first,d.day)+padding)/7)*15+'px';months.append(el);previousMonth=month}
        const cell=document.createElement('div');cell.dataset.day=d.day;grid.append(cell);calendarCells.set(d.day,cell);
      }
      body.append(weekdays,grid);block.append(label,months,body);return block;
    }));
    calendarLayout=layout;
  }
  const max=Math.max(1,...days.map(d=>total(d.counts)));
  for(const d of days){const cell=calendarCells.get(d.day),value=total(d.counts),level=value?Math.max(1,Math.ceil(value/max*4)):0;cell.className='heat-cell level-'+level+(d.day===today?' today':'');cell.title=t('calendar.tooltip',{day:d.day,count:n(value),keyboard:n(d.counts.keyboard),mouse:n(d.counts.mouse)});}
  $('calendar-description').textContent=t('calendar.caption',{range:calendarRange==='all'?t('common.all_history'):calendarRange==='90'?t('common.recent90'):t('common.year',{year:calendarRange})});
  $('active-days').textContent=t('calendar.recorded',{count:n(days.filter(d=>total(d.counts)>0).length)});
  const comparison=H.bookComparison(days.reduce((sum,d)=>sum+d.counts.keyboard,0)),aside=$('calendar-aside');
  aside.hidden=!comparison;
  if(comparison){
    const book=t('calendar.book_title');
    aside.textContent=comparison.opening?t('calendar.literary_opening'):t(comparison.copies===1?'calendar.literary_one':'calendar.literary',{count:n(comparison.copies),book});
    aside.title=t('calendar.literary_basis',{book,characters:n(comparison.characters)});
  }else{aside.textContent='';aside.title='';}
  $('calendar-range').textContent=t('calendar.period',{start:I.day(bounds.start),end:I.day(bounds.end),count:n(days.reduce((sum,d)=>sum+total(d.counts),0))});
}
function renderHistory(){
  const today=data.now.split('T')[0],records=H.recordedInRange(data.history_days,today,historyRange),bounds=H.bounds(data.history_days,today,historyRange);
  const pagination=H.paginate(records,historyPage);historyPage=pagination.page;
  const signature=I.locale+':'+currentTheme+':'+historyRange+':'+historyPage+':'+today+':'+JSON.stringify(records);
  if(signature===historySignature)return;
  historySignature=signature;
  const days=H.calendarDays(records,bounds.start,bounds.end),trend=H.trend(days);
  const granularity=t(days.length>370?'history.monthly':days.length>90?'history.weekly':'history.daily');
  $('history-range').textContent=t('history.range',{start:I.day(bounds.start),end:I.day(bounds.end),grouping:granularity});
  const keyboard=records.reduce((sum,d)=>sum+d.counts.keyboard,0),mouse=records.reduce((sum,d)=>sum+d.counts.mouse,0);
  $('history-summary').textContent=t('history.summary',{days:n(pagination.total),count:n(keyboard+mouse),keyboard:n(keyboard),mouse:n(mouse)});
  renderTrend($('history-chart'),trend);
  $('history-table').replaceChildren(...pagination.rows.map(d=>{
    const tr=document.createElement('tr');for(const value of [I.day(d.day),n(d.counts.keyboard),n(d.counts.mouse),n(total(d.counts))]){const td=document.createElement('td');td.textContent=value;tr.append(td)}return tr;
  }));
  $('history-prev').disabled=historyPage===0;$('history-next').disabled=historyPage+1>=pagination.pages;
  $('history-page-info').textContent=pagination.total?t('history.pagination',{start:n(historyPage*30+1),end:n(Math.min((historyPage+1)*30,pagination.total)),count:n(pagination.total),page:n(historyPage+1),pages:n(pagination.pages)}):t('common.empty_history');
}
function renderKeyboard(){
  const stats=data.keyboard_stats||{physical:data.top_keys};
  const view=K.distribution(stats);
  keyboardDistribution=view;updateKeyboardView();
  $('keyboard-coverage').textContent=t('keys.coverage',{count:n(view.visibleRanked.length)});
  if(keyboardView==='treemap'){
    renderKeyboardTreemap(view);return;
  }
  const max=Math.max(1,...view.visibleRanked.map(k=>k.count));
  const signature=JSON.stringify(view.rows.map(row=>row.map(key=>key.id)));
  if(keyboardLayoutSignature!==signature){
    keyboardCells.clear();keyboardLayoutSignature=signature;
    $('keyboard-layout').replaceChildren(...view.rows.map((row,index)=>{
      const arrowIds=['up','left','down','right'],arrows=row.filter(key=>arrowIds.includes(key.id));
      const node=document.createElement('div');node.className='keyboard-row'+(index===0?' function-row':row.some(key=>key.supplemental)?' supplemental-row':arrows.length?' modifier-row':'');
      function createCell(key){
        const cell=document.createElement('div');cell.className='keyboard-key';cell.style.setProperty('--key-width',key.width);cell.dataset.key=key.id;cell.tabIndex=0;cell.setAttribute('role','img');
        const label=document.createElement('span');label.className='key-label';label.textContent=key.label;
        const count=document.createElement('span');count.className='key-count';cell.append(label,count);
        keyboardCells.set(key.id,cell);return cell;
      }
      let arrowCluster=null;
      for(const key of row){
        if(arrowIds.includes(key.id)){
          if(!arrowCluster){
            arrowCluster=document.createElement('div');arrowCluster.className='keyboard-arrow-cluster';
            for(const id of arrowIds){const arrow=arrows.find(item=>item.id===id);if(arrow)arrowCluster.append(createCell(arrow));}
            node.append(arrowCluster);
          }
        }else node.append(createCell(key));
      }
      return node;
    }));
  }
  for(const key of view.rows.flat()){
    const cell=keyboardCells.get(key.id),hex=keyColor(key.count,max);
    cell.style.setProperty('--key-bg',hex);cell.style.setProperty('--key-text',T.textFor(hex));
    const label=key.label;
    cell.firstElementChild.textContent=label;
    cell.lastElementChild.textContent=K.compact(key.count);
    const detail=t('keys.tooltip',{key:label,count:n(key.count),percent:I.percent(view.total?key.count/view.total:0)});
    cell.title=detail;cell.setAttribute('aria-label',detail);
  }
}
function keyColor(count,max){
  const intensity=count?Math.pow(Math.log1p(count)/Math.log1p(max),3):0;
  const palette=T.get(currentTheme),from=T.rgb(palette.empty),to=T.rgb(palette.full);
  return '#'+from.map((value,i)=>Math.round(value+(to[i]-value)*intensity).toString(16).padStart(2,'0')).join('');
}
function renderKeyboardTreemap(view){
  const container=$('keyboard-treemap'),width=container.clientWidth||640,height=container.clientHeight||279;
  const keys=view.visibleRanked,sum=keys.reduce((sum,key)=>sum+key.count,0),max=Math.max(1,...keys.map(key=>key.count));
  const signature=JSON.stringify(keys.map(key=>key.id).sort());
  if(container.dataset.keys!==signature){
    const active=new Set(keys.map(key=>key.id));
    for(const id of keyboardBlocks.keys())if(!active.has(id))keyboardBlocks.delete(id);
    container.dataset.keys=signature;
    container.replaceChildren(...keys.map(key=>{
      if(keyboardBlocks.has(key.id))return keyboardBlocks.get(key.id);
      const cell=document.createElement('div');cell.className='keyboard-block';cell.dataset.key=key.id;cell.tabIndex=0;cell.setAttribute('role','img');
      const surface=document.createElement('span');surface.className='keyboard-block-surface';
      const label=document.createElement('span');label.className='keyboard-block-label';label.textContent=key.label;surface.append(label);cell.append(surface);
      keyboardBlocks.set(key.id,cell);return cell;
    }));
    if(!keys.length){const empty=document.createElement('div');empty.className='empty-label';container.append(empty);}
  }
  if(!keys.length)container.firstElementChild.textContent=t('common.empty');
  for(const key of K.treemap(keys,width,height)){
    const cell=keyboardBlocks.get(key.id),color=keyColor(key.count,max);
    for(const [name,value]of [['left',key.x/width],['top',key.y/height],['width',key.w/width],['height',key.h/height]])cell.style[name]=value*100+'%';
    cell.style.setProperty('--block-gap',Math.min(.75,key.w*.1,key.h*.1)+'px');
    cell.style.setProperty('--block-font',Math.min(24,Math.max(11,Math.min(key.w,key.h)/4))+'px');
    cell.style.setProperty('--key-bg',color);cell.style.setProperty('--key-text',T.textFor(color));
    cell.firstElementChild.firstElementChild.hidden=key.w<Math.max(28,key.label.length*8+8)||key.h<18;
    const detail=t('keys.tooltip',{key:key.label,count:n(key.count),percent:I.percent(sum?key.count/sum:0)});
    cell.title=detail;cell.setAttribute('aria-label',detail);
  }
}
send('ready');

$('takeover').addEventListener('click',()=>send('takeover'));
$('autostart').addEventListener('click',()=>send('autostart'));
