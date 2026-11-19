'use strict';
const ClickPulseInsights=(()=>{
const positive=value=>Number.isFinite(value)&&value>0?Math.floor(value):0;
function mouseBreakdown(snapshot){
 const b=snapshot.mouse_buttons||{},rows=['left','right','middle','side','other'].map(key=>({key,count:positive(b[key])}));
 const unknown=Math.max(0,positive(snapshot.lifetime?.mouse)-rows.reduce((sum,row)=>sum+row.count,0));
 if(unknown)rows.push({key:'unknown',count:unknown});return rows;
}
function dayKey(now,timeZone){const parts=new Intl.DateTimeFormat('en',{timeZone:timeZone==='system'?undefined:timeZone,year:'numeric',month:'2-digit',day:'2-digit'}).formatToParts(new Date(now));const get=key=>parts.find(p=>p.type===key).value;return get('year')+'-'+get('month')+'-'+get('day');}
function cardData(snapshot,day,timeZone,systemTimeZone=Intl.DateTimeFormat().resolvedOptions().timeZone){
 if(!/^\d{4}-\d{2}-\d{2}$/.test(day)||Number.isNaN(Date.parse(day+'T12:00:00Z'))||new Date(day+'T12:00:00Z').toISOString().slice(0,10)!==day)return null;
 const rows=snapshot.history_days||[],found=rows.find(row=>row.day===day),counts=found?.counts||{};
 const end=Date.parse(day+'T12:00:00Z');
 const week=Array.from({length:7},(_,i)=>{
  const key=new Date(end-(6-i)*86400000).toISOString().slice(0,10),d=rows.find(row=>row.day===key);
  return {day:key,keyboard:positive(d?.counts.keyboard),mouse:positive(d?.counts.mouse)};
 });
 // Shortcut dates cannot be reprojected: only daily recording-local totals exist.
 // Compare zone identities, not today's offset (DST can differ on another date).
 const zoneId=zone=>new Intl.DateTimeFormat('en',{timeZone:zone}).resolvedOptions().timeZone;
 const shortcutAligned=timeZone==='system'||zoneId(timeZone)===zoneId(systemTimeZone);
 const daily=snapshot.share_shortcut,shortcutLoaded=!shortcutAligned||daily?.day===day;
 return {day,shortcutLoaded,shortcut:shortcutAligned&&daily?.day===day&&positive(daily.top?.count)?daily.top:null,keyboard:positive(counts.keyboard),mouse:positive(counts.mouse),week,partial:day===dayKey(snapshot.now,timeZone),demo:!!snapshot.demo,timeZone};
}
function shortcutRows(rows,C,query,platform){
 return (rows||[]).filter(r=>positive(r.count)&&(!platform||r.platform===platform)).map(r=>({...r,entries:C.matches(r)}))
 .filter(r=>!query||C.matchesQuery(r,r.entries,query))
 .sort((a,b)=>b.count-a.count||a.platform.localeCompare(b.platform)||a.modifiers-b.modifiers||a.key-b.key);
}
// The input is already ranked and platform-filtered. Search only narrows the detail list.
function shortcutSummary(rows){
 const total=rows.reduce((sum,row)=>sum+row.count,0),topFive=rows.slice(0,5).reduce((sum,row)=>sum+row.count,0);
 return {total,distinct:rows.length,top:rows.slice(0,8),topFive,others:total-topFive};
}
function create({document,window,I,T,send,toast,syncSelect=()=>{},getSystemTimeZone=()=>Intl.DateTimeFormat().resolvedOptions().timeZone}){
 const C=ClickPulseShortcutCatalog,$=id=>document.getElementById(id),t=(key,vars)=>I.t(key,vars),n=value=>I.number(value);
 const datePicker=typeof ClickPulseDatePicker==='undefined'?null:ClickPulseDatePicker.create({document,window,I,input:$('share-date')});
 let paintVersion=0,painting=false,painted=false,exporting=false,suspended=false;
 const canCopy=()=>['macos','windows'].includes(window.clickpulsePlatform)&&!window.clickpulsePreview;
 $('share-copy').hidden=!canCopy();
 let snapshot=null,active='',signature='',library=false,showShift=false,query='',system='',style='blue',selectedDay='',followToday=true,card=null,visibleRows=100,listSignature='',listCounters=[],mouseButtonsSignature='';
 let shortcutChartSignature='';
 const node=(tag,text,cls)=>{const el=document.createElement(tag);if(text!==undefined)el.textContent=text;if(cls)el.className=cls;return el;};
 const svg=(tag,attrs)=>{const el=document.createElementNS('http://www.w3.org/2000/svg',tag);for(const [key,value]of Object.entries(attrs||{}))el.setAttribute(key,value);return el;};
 function invalidate(){signature='';if(snapshot)render(active,snapshot);}
 function remember(){window.captureSessionState?.();}
 function sourceLink(entry){const a=node('button',C.sources[entry.source].name+' ↗','shortcut-source');a.type='button';a.onclick=()=>window.clickpulsePreview?window.open(C.sources[entry.source].url,'_blank','noopener'):send('shortcut_reference',{source:entry.source});return a;}
 function chordKeys(row){const keys=node('div',undefined,'shortcut-chord');for(const key of C.chordParts(row))keys.append(node('kbd',key));return keys;}
 function chord(row){const el=node('div');el.append(chordKeys(row),node('div',row.platform==='macos'?'macOS':row.platform==='windows'?'Windows':'Ubuntu / GNOME','shortcut-system'));return el;}
 function renderShortcutCharts(rows,emptyKey){
  const summary=shortcutSummary(rows),{total,distinct,top,topFive,others}=summary;
  const mixedSystems=new Set(rows.map(row=>row.platform)).size>1;
  const next=JSON.stringify([I.locale,system,total,distinct,top,mixedSystems,emptyKey]);if(next===shortcutChartSignature)return;shortcutChartSignature=next;
  const percentage=value=>total?I.percent(value/total,1):'—';
  const metric=(label,value)=>{const el=node('div',undefined,'insight-metric');el.append(node('span',t(label)),node('strong',n(value)));return el;};
  const leader=node('div',undefined,'insight-metric shortcut-leader');leader.append(node('span',t('shortcuts.most_used')));
  if(top.length){leader.append(chordKeys(top[0]),node('small',t('shortcuts.share',{percent:percentage(top[0].count)})));}
  else leader.append(node('strong','—'));
  $('shortcut-metrics').replaceChildren(metric('shortcuts.presses',total),metric('shortcuts.distinct',distinct),leader);
  const ranking=top.map(row=>{
   const el=node('div',undefined,'shortcut-rank-row'),label=mixedSystems?chord(row):chordKeys(row),track=node('div',undefined,'shortcut-rank-track'),bar=node('div',undefined,'shortcut-rank-bar'),value=node('div',undefined,'shortcut-rank-value');
   bar.style.width=(row.count/top[0].count*100)+'%';track.setAttribute('aria-hidden','true');track.append(bar);
   value.append(node('strong',n(row.count)),node('small',percentage(row.count)));el.append(label,track,value);return el;
  });
  $('shortcut-ranking').replaceChildren(...ranking);
  if(!total){const empty=node('div',undefined,'shortcut-chart-empty');empty.append(node('strong',t('shortcuts.chart_empty')),node('p',t(emptyKey)));$('shortcut-ranking').append(empty);}
  const ring=node('div',undefined,'shortcut-ring'),drawing=svg('svg',{viewBox:'0 0 200 200','aria-hidden':'true'});
  drawing.append(svg('circle',{cx:100,cy:100,r:82,fill:'none',stroke:'var(--heat-1)','stroke-width':18}));
  if(total)drawing.append(svg('circle',{cx:100,cy:100,r:82,fill:'none',stroke:'var(--blue)','stroke-width':18,pathLength:100,'stroke-dasharray':(topFive/total*100)+' 100',transform:'rotate(-90 100 100)'}));
  const center=node('div',undefined,'shortcut-ring-label');center.append(node('strong',percentage(topFive)),node('span',t('shortcuts.top_five')));ring.append(drawing,center);
  const legend=node('div',undefined,'shortcut-ring-legend');
  for(const [key,count]of [['top_five',topFive],['others',others]]){const item=node('div',undefined,'shortcut-legend-row');item.append(node('i',undefined,key==='top_five'?'shortcut-legend-top':'shortcut-legend-other'),node('span',t('shortcuts.'+key)),node('strong',n(count)));legend.append(item);}
  $('shortcut-concentration').replaceChildren(ring,legend,node('p',t('shortcuts.concentration_note'),'insight-note'));
 }
 function renderShortcuts(){
  const pending=!library&&!Array.isArray(snapshot.shortcuts);
  const all=shortcutRows(snapshot.shortcuts,C,'',system).filter(row=>showShift||row.modifiers!==4);
  const emptyKey=!(snapshot.shortcuts||[]).some(r=>positive(r.count))&&!library&&!query?'shortcuts.empty':'shortcuts.no_results';
  $('shortcut-dashboard').hidden=library||pending;$('shortcut-period').hidden=library;
  $('shortcut-shift-filter').hidden=library;$('shortcut-show-shift').checked=showShift;
  $('shortcut-loading').hidden=!pending;$('shortcut-details').hidden=pending;
  $('shortcut-details-title').textContent=t(library?'shortcuts.library':'shortcuts.details');
  if(!library&&!pending)renderShortcutCharts(all,emptyKey);
  $('shortcut-search').placeholder=t('shortcuts.search');$('shortcut-search').setAttribute('aria-label',t('shortcuts.search'));
  $('shortcut-library').setAttribute('aria-pressed',String(library));$('shortcut-activity').setAttribute('aria-pressed',String(!library));
  $('shortcut-system').setAttribute('aria-label',t('shortcuts.all_systems'));$('shortcut-system').options[0].textContent=t('shortcuts.all_systems');
  syncSelect('shortcut-system');
  if(pending)return;
  const rows=library?C.entries.filter(e=>(!system||e.platform===system)&&C.search(e,query)).map(e=>({...e,entries:[e]})):all.filter(r=>!query||C.matchesQuery(r,r.entries,query));
  const more=$('shortcut-more');more.hidden=rows.length<=visibleRows;more.textContent=t('shortcuts.more',{count:n(Math.max(0,rows.length-visibleRows))});
  const shown=rows.slice(0,visibleRows);
  const nextList=JSON.stringify([library,showShift,query,system,I.locale,visibleRows,emptyKey,shown.map(r=>[r.platform,r.modifiers,r.key])]);
  const max=Math.max(1,...all.map(r=>r.count));
  if(nextList!==listSignature){
  listSignature=nextList;listCounters=[];
  $('shortcut-list').replaceChildren(...shown.map(row=>{
   const el=node('div',undefined,'shortcut-row');el.append(chord(row));const annotation=node('div',undefined,'shortcut-annotation');
   if(row.entries.length){for(const entry of row.entries){annotation.append(node('strong',C.title(entry,I.locale)),node('small',t('shortcuts.reference')+' · '+C.scope(entry,I.locale)));if(library)annotation.append(sourceLink(entry));}}
   else annotation.append(node('small',t('shortcuts.unknown')));
   el.append(annotation);const count=node('div',undefined,'shortcut-count');
   if(!library){const label=node('span'),track=node('i'),bar=node('b');track.append(bar);count.append(label,track);listCounters.push({label,bar});}el.append(count);return el;
  }));
  if(!rows.length)$('shortcut-list').append(node('div',t(emptyKey),'insight-empty'));
  }
  if(!library)shown.forEach((row,i)=>{listCounters[i].label.textContent=n(row.count);listCounters[i].bar.style.width=(row.count/max*100)+'%';});
 }
 function renderMouse(){
  const rows=mouseBreakdown(snapshot),total=rows.reduce((sum,row)=>sum+row.count,0),palette=T.get(document.documentElement.dataset.theme);
  const buttonsSignature=JSON.stringify([rows,I.locale,document.documentElement.dataset.theme]);
  if(buttonsSignature===mouseButtonsSignature)return;
  mouseButtonsSignature=buttonsSignature;
  const colors={left:palette.accent,right:palette.mouse,middle:palette.navy,side:'var(--heat-3)',other:'var(--muted)',unknown:'var(--heat-1)'};
  const percentage=count=>total?I.percent(count/total,1):'—';
  const caption=row=>t('chart.mouse_'+row.key)+': '+n(row.count)+' · '+percentage(row.count);
  const ring=node('div',undefined,'mouse-ring'),chart=svg('svg',{viewBox:'0 0 200 200',role:'img','aria-label':t('mouse.buttons')});
  chart.append(svg('circle',{cx:100,cy:100,r:82,fill:'none',stroke:'var(--heat-0)','stroke-width':23}));
  let offset=0;
  for(const row of rows.filter(row=>row.count)){
   const share=row.count/total*100,arc=svg('circle',{cx:100,cy:100,r:82,fill:'none',stroke:colors[row.key],'stroke-width':23,pathLength:100,'stroke-dasharray':share+' 100','stroke-dashoffset':-offset,transform:'rotate(-90 100 100)'});
   const title=svg('title');title.textContent=caption(row);arc.append(title);chart.append(arc);offset+=share;
  }
  const leader=rows.reduce((best,row)=>row.count>best.count?row:best,rows[0]),center=node('div',undefined,'mouse-ring-label');
  center.append(node('strong',total?I.percent(leader.count/total,1):'—'),node('span',total?t('chart.mouse_'+leader.key):t('common.empty')));
  ring.append(chart,center);$('mouse-chart').replaceChildren(ring);
  $('mouse-distribution').replaceChildren(...rows.filter(row=>row.count||['left','right','middle'].includes(row.key)).map(row=>{
   const el=node('div',undefined,'mouse-row'),label=node('span',undefined,'mouse-label'),swatch=node('i');swatch.style.backgroundColor=colors[row.key];swatch.setAttribute('aria-hidden','true');
   label.append(swatch,node('span',t('chart.mouse_'+row.key)));el.append(label,node('strong',n(row.count)),node('small',percentage(row.count)));return el;
  }));
  const drawing=svg('svg',{viewBox:'0 0 200 270',class:'mouse-drawing',role:'img','aria-label':t('mouse.buttons')});
  // Share one linear grayscale scale across physical buttons: more clicks, darker fill.
  // Unknown historic clicks have no physical region and do not set this scale.
  const ink={shell:'#f6f6f6',edge:'#b9b9b9'},buttons=rows.filter(row=>['left','right','middle','side'].includes(row.key));
  const maximum=Math.max(1,...buttons.map(row=>row.count));
  for(const row of buttons){const level=Math.round(242-146*row.count/maximum);ink[row.key]=`rgb(${level}, ${level}, ${level})`;}
  const side=svg('rect',{x:32,y:111,width:9,height:37,rx:4.5,fill:ink.side,stroke:ink.edge,'stroke-width':1,'data-mouse-button':'side'}),sideTitle=svg('title');sideTitle.textContent=caption(rows.find(row=>row.key==='side'));side.append(sideTitle);drawing.append(side);
  drawing.append(svg('path',{d:'M100 17C65 17 43 43 41 82L35 166C32 211 55 246 100 246S168 211 165 166L159 82C157 43 135 17 100 17Z',fill:ink.shell,stroke:ink.edge,'stroke-width':1.5}));
  for(const [key,d]of [['left','M97 24C67 25 49 46 47 81L45 108C61 114 79 117 97 117Z'],['right','M103 24C133 25 151 46 153 81L155 108C139 114 121 117 103 117Z']]){
   const shape=svg('path',{d,fill:ink[key],'data-mouse-button':key}),title=svg('title');title.textContent=caption(rows.find(row=>row.key===key));shape.append(title);drawing.append(shape);
  }
  const wheel=svg('g'),wheelTitle=svg('title');wheelTitle.textContent=caption(rows.find(row=>row.key==='middle'));
  wheel.append(wheelTitle,svg('rect',{x:91,y:45,width:18,height:40,rx:9,fill:ink.middle,stroke:ink.edge,'stroke-width':1.2,'data-mouse-button':'middle'}));
  for(const y of [57,63,69])wheel.append(svg('path',{d:'M97 '+y+'h6',stroke:ink.edge,'stroke-width':1,'stroke-linecap':'round',opacity:'.7'}));
  drawing.append(wheel);$('mouse-diagram').replaceChildren(drawing);
 }
 function rounded(ctx,x,y,w,h,r){r=Math.min(r,w/2,h/2);ctx.beginPath();ctx.moveTo(x+r,y);ctx.arcTo(x+w,y,x+w,y+h,r);ctx.arcTo(x+w,y+h,x,y+h,r);ctx.arcTo(x,y+h,x,y,r);ctx.arcTo(x,y,x+w,y,r);ctx.closePath();}
 function drawCard(){
  if(!card)return;const canvas=$('share-canvas');if(canvas.width!==1080||canvas.height!==1350){canvas.width=1080;canvas.height=1350;}const ctx=canvas.getContext('2d');if(!ctx)return;
  const dark=style==='ink',paper=style==='paper',background=dark?'#102543':paper?'#fbfaf6':'#edf4ff',ink=dark?'#f4f7fc':'#172b4a',accent=dark?'#87b7ff':'#2459a2',muted=dark?'#a9bedc':'#63758e',line=dark?'#29405f':'#d4dfef';
  const regional=I.locale==='zh-TW'?'"PingFang TC", "Microsoft JhengHei", "Noto Sans CJK TC"':I.locale==='ja'?'"Hiragino Sans", "Yu Gothic", "Noto Sans CJK JP"':I.locale==='ko'?'"Apple SD Gothic Neo", "Malgun Gothic", "Noto Sans CJK KR"':'"PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC"';
  const sans='"ClickPulse UI", '+regional+', system-ui, sans-serif';
  ctx.fillStyle=background;ctx.fillRect(0,0,1080,1350);
  ctx.fillStyle=paper?'#e8e3d7':dark?'#193456':'#dce9fc';ctx.beginPath();ctx.arc(1080,-20,420,0,Math.PI*2);ctx.fill();
  function text(value,x,y,size,weight=400,color=ink,max=936,align='left'){ctx.font=weight+' '+size+'px '+sans;ctx.fillStyle=color;ctx.textAlign=align;while(ctx.measureText(value).width>max&&size>14){size--;ctx.font=weight+' '+size+'px '+sans;}ctx.fillText(value,x,y,max);}
  const brand=document.querySelector('.brand-icon img');if(brand?.complete&&brand.naturalWidth){ctx.fillStyle='#ffffff';rounded(ctx,72,65,64,64,16);ctx.fill();ctx.drawImage(brand,82,75,44,44);text('ClickPulse',154,112,31,750);}else text('ClickPulse',72,112,31,750);text(t('share.activity'),72,235,23,650,accent);
  text(I.day(card.day,{year:'numeric',month:'long',day:'numeric'}),72,292,37,600);
  text(t('share.headline'),72,385,48,700);
  const total=card.keyboard+card.mouse;
  text(n(total),65,573,146,750,accent);text(t('share.events'),76,628,26,500,muted);
  ctx.strokeStyle=line;ctx.lineWidth=2;ctx.beginPath();ctx.moveTo(72,689);ctx.lineTo(1008,689);ctx.stroke();
  for(const [x,value,label,color]of [[72,card.keyboard,t('common.keyboard'),accent],[560,card.mouse,t('common.mouse'),dark?'#67c9f4':'#288ab7']]){
   text(n(value),x,776,54,700);ctx.fillStyle=color;rounded(ctx,x,805,12,12,3);ctx.fill();text(label,x+25,819,24,500,muted,400);
  }
  if(card.shortcut){
   text(t('share.top_shortcut'),72,878,22,500,muted,460);
   const combination=C.chordParts(card.shortcut).join(' + ');
   const value=combination+'  ·  '+t('share.shortcut_count',{count:n(card.shortcut.count)});
   text(value,1008,878,27,650,accent,440,'right');
  }
  text(t('share.week'),72,925,22,500,muted);
  const max=Math.max(1,...card.week.map(d=>d.keyboard+d.mouse)),barW=78,gap=51;
  card.week.forEach((d,i)=>{
   const x=78+i*(barW+gap),height=Math.max(4,(d.keyboard+d.mouse)/max*190);
   ctx.fillStyle=i===6?accent:dark?'#35567f':'#bdd0ed';rounded(ctx,x,1155-height,barW,height,6);ctx.fill();
   text(I.day(d.day,{weekday:'short'}),x,1196,19,400,muted,barW+25);
  });
  const note=card.demo?t('share.demo'):card.partial?t('share.partial'):'';
  text(note,72,1282,20,500,muted,580);
  ctx.font='400 18px '+sans;ctx.textAlign='right';ctx.fillStyle=muted;ctx.fillText('github.com/endNone/ClickPulse',1008,1282);
  canvas.setAttribute('aria-label',I.day(card.day)+' · '+n(total)+' '+t('share.events')+(card.shortcut?' · '+t('share.top_shortcut')+': '+C.chordParts(card.shortcut).join(' + ')+' · '+t('share.shortcut_count',{count:n(card.shortcut.count)}):''));
  return true;
 }
 function shareButtons(){
  $('share-save').disabled=suspended||painting||!painted||exporting||!card?.shortcutLoaded;
  $('share-copy').hidden=!canCopy();
  $('share-copy').disabled=suspended||painting||!painted||exporting||!card?.shortcutLoaded||!canCopy();
 }
 function releaseCard(){
  // Invalidate pending font/logo work before releasing the backing bitmap.
  paintVersion++;painting=false;painted=false;card=null;signature='';
  const canvas=$('share-canvas');canvas.width=0;canvas.height=0;shareButtons();
 }
 function setWindowState(view){
  const next=!!view.compact||view.visible===false;
  if(next===suspended)return;
  suspended=next;
  if(suspended)releaseCard();
  else if(snapshot)render(active,snapshot);
 }
 let fontsLoaded=false,fontLoad=null;
 async function paintCard(){
  if(suspended||active!=='share'||!card)return false;
  const version=++paintVersion;painting=true;painted=false;shareButtons();
  try {
   if(document.fonts&&!fontsLoaded){
    fontLoad ||= document.fonts.load('750 146px "ClickPulse UI"').then(()=>document.fonts.ready).then(()=>{fontsLoaded=true;}).catch(error=>{fontLoad=null;throw error;});
    await fontLoad;
   }
   if(version!==paintVersion||suspended||active!=='share'||!card)return false;
   painted=!!drawCard();return painted;
  } catch(error){
   if(version===paintVersion&&active==='share')toast(t('share.failed'));
   return false;
  } finally {if(version===paintVersion){painting=false;shareButtons();}}
 }
 function renderShare(){
  const today=dayKey(snapshot.now,I.effectiveTimeZone);
  const input=$('share-date');input.max=today;
  if(document.activeElement!==input)input.value=selectedDay;
  datePicker?.sync();
  $('share-empty').hidden=!!(card&&(card.keyboard||card.mouse));
  for(const button of document.querySelectorAll('[data-share-style]'))button.setAttribute('aria-pressed',String(button.dataset.shareStyle===style));
  void paintCard();
 }
 $('shortcut-search').addEventListener('input',e=>{query=e.target.value;visibleRows=100;invalidate();});
 $('shortcut-show-shift').addEventListener('change',e=>{showShift=e.target.checked;visibleRows=100;invalidate();remember();});
 $('shortcut-system').addEventListener('change',e=>{system=e.target.value;visibleRows=100;invalidate();remember();});
 $('shortcut-library').addEventListener('click',()=>{library=true;visibleRows=100;invalidate();remember();});
 $('shortcut-activity').addEventListener('click',()=>{library=false;visibleRows=100;invalidate();remember();});
 $('shortcut-more').addEventListener('click',()=>{visibleRows=Math.min(6000,visibleRows+100);invalidate();remember();});
 $('share-date').addEventListener('change',e=>{
  if(!snapshot)return;
  const today=dayKey(snapshot.now,I.effectiveTimeZone),value=e.target.value;
  if(!value||value>today||!cardData(snapshot,value,I.effectiveTimeZone)||(e.target.validity&&!e.target.validity.valid)){e.target.value=selectedDay||today;return;}
  selectedDay=value;followToday=selectedDay===today;invalidate();remember();
 });
 for(const button of document.querySelectorAll('[data-share-style]'))button.addEventListener('click',()=>{style=button.dataset.shareStyle;invalidate();remember();});
 async function exportCard(copy){
  if(suspended||active!=='share'||!card?.shortcutLoaded||exporting||painting||!painted||(copy&&!canCopy()))return;
  exporting=true;shareButtons();
  try {
   // The current bitmap is already font-ready. Capture it synchronously before
   // any hide/Mini transition; the native job owns the PNG after IPC handoff.
   const png=$('share-canvas').toDataURL('image/png');
   if(window.clickpulsePreview){const a=node('a');a.href=png;a.download='ClickPulse-'+card.day+'.png';a.click();}
   else send(copy?'copy_share_card':'save_share_card',{png,day:card.day});
  } catch(error){toast(t(copy?'share.copy_failed':'share.failed'));}
  finally {exporting=false;shareButtons();}
 }
 $('share-save').addEventListener('click',()=>exportCard(false));
 $('share-copy').addEventListener('click',()=>exportCard(true));
 const redrawCard=()=>{if(active==='share'&&!suspended)void paintCard();};
 document.fonts?.addEventListener('loadingdone',redrawCard);
 document.querySelector('.brand-icon img')?.addEventListener('load',redrawCard);
 function render(page,data){
  // The mouse charts now live below the keyboard on the shared Heatmap page.
  if(page==='keys')page='mouse';
  if(page!=='share')datePicker?.close();
  if(active==='share'&&page!=='share')releaseCard();
  snapshot=data;active=page;if(!['shortcuts','mouse','share'].includes(page))return;
  if(suspended)return;
  let today='';
  if(page==='share'){
   today=dayKey(data.now,I.effectiveTimeZone);
   if(!selectedDay||followToday||selectedDay>today){const changed=!followToday&&!!selectedDay;selectedDay=today;followToday=true;if(changed)remember();}
   card=cardData(data,selectedDay,I.effectiveTimeZone,getSystemTimeZone());
  }
  const next=JSON.stringify([page,I.locale,I.effectiveTimeZone,document.documentElement.dataset.theme,query,system,library,showShift,style,selectedDay,today,data.demo,page==='shortcuts'?data.shortcuts:page==='mouse'?[data.lifetime?.mouse,data.mouse_buttons]:card]);
  if(next===signature)return;signature=next;
  if(page==='shortcuts')renderShortcuts();else if(page==='mouse')renderMouse();else renderShare();
 }
 function sessionState(){return {shortcut_library:library,shortcut_show_shift:showShift,shortcut_system:system,shortcut_rows:visibleRows,share_style:style,share_day:followToday?null:selectedDay};}
 function restoreSession(state){
  if(!state)return;
  library=state.shortcut_library===true;showShift=state.shortcut_show_shift===true;system=['macos','windows','linux'].includes(state.shortcut_system)?state.shortcut_system:'';
  visibleRows=Number.isInteger(state.shortcut_rows)&&state.shortcut_rows>=100&&state.shortcut_rows<=6000&&state.shortcut_rows%100===0?state.shortcut_rows:100;
  style=['blue','paper','ink'].includes(state.share_style)?state.share_style:'blue';
  selectedDay=typeof state.share_day==='string'&&/^\d{4}-\d{2}-\d{2}$/.test(state.share_day)?state.share_day:'';followToday=!selectedDay;
  $('shortcut-system').value=system;syncSelect('shortcut-system');signature='';listSignature='';
 }
 releaseCard();
 return {render,sessionState,restoreSession,setWindowState};
}
return {create,mouseBreakdown,dayKey,cardData,shortcutRows,shortcutSummary};
})();
if(typeof module!=='undefined')module.exports=ClickPulseInsights;
