'use strict';
const ClickPulseInsights=(()=>{
const positive=value=>Number.isFinite(value)&&value>0?Math.floor(value):0;
function mouseBreakdown(snapshot){
 const b=snapshot.mouse_buttons||{},rows=['left','right','middle','side','other'].map(key=>({key,count:positive(b[key])}));
 const unknown=Math.max(0,positive(snapshot.lifetime?.mouse)-rows.reduce((sum,row)=>sum+row.count,0));
 if(unknown)rows.push({key:'unknown',count:unknown});return rows;
}
function dayKey(now,timeZone){const parts=new Intl.DateTimeFormat('en',{timeZone:timeZone==='system'?undefined:timeZone,year:'numeric',month:'2-digit',day:'2-digit'}).formatToParts(new Date(now));const get=key=>parts.find(p=>p.type===key).value;return get('year')+'-'+get('month')+'-'+get('day');}
const clock=hour=>String(hour).padStart(2,'0')+':00';
function calendarDate(day){
 const stamp=Date.parse(day+'T12:00:00Z');
 return /^\d{4}-\d{2}-\d{2}$/.test(day)&&Number.isFinite(stamp)&&new Date(stamp).toISOString().slice(0,10)===day?stamp:null;
}
// The demo has no database. Split only its synthetic daily totals into hours.
function demoReport(snapshot,start,end,today,currentHour,platform){
 const history=snapshot.history_days||[],hours=[],counts=[0,0,0,0,0];
 for(const row of history){
  const day=row.day;
  if(day<start||day>end)continue;
  if(day===today && snapshot.hours?.length===24){
   snapshot.hours.forEach((v,h)=>{if(v.keyboard||v.mouse)hours.push([day,h,v.keyboard,v.mouse]);});
  }else hours.push(...ClickPulseScatter.demoHours([row],day===today?currentHour:23));
  counts.forEach((_,i)=>counts[i]+=Math.floor(row.counts.keyboard/[80,101,137,194,286][i]));
 }
 const ranking=counts.map((count,i)=>({count,platform,modifiers:platform==='macos'?8:1,key:[8,9,6,0,1][i]})).filter(r=>r.count>0);
 return {hours,ranking};
}
const dayMs=86400000;
const MIN_WEEKLY_DAYS=2,MAX_WEEKLY_DAYS=31;
const isoDay=stamp=>new Date(stamp).toISOString().slice(0,10);
function shiftDay(day,delta){
 return isoDay(Math.max(calendarDate('0001-01-01'),calendarDate(day)+delta*dayMs));
}
function weekStart(day){
 return shiftDay(day,-6);
}
function clampWeeklyStart(day,selectedStart){
 const earliest=shiftDay(day,-(MAX_WEEKLY_DAYS-1)),latest=shiftDay(day,-(MIN_WEEKLY_DAYS-1));
 const chosen=selectedStart||weekStart(day);
 return chosen<earliest?earliest:chosen>latest?latest:chosen;
}
function cardData(snapshot,day,timeZone,systemTimeZone=Intl.DateTimeFormat().resolvedOptions().timeZone,period='daily',platform='macos',selectedStart=''){
 const end=calendarDate(day);
 if(end===null)return null;
 const weekly=period==='weekly',today=dayKey(snapshot.now,timeZone),partial=day===today;
 const zoneId=zone=>new Intl.DateTimeFormat('en',{timeZone:zone==='system'?systemTimeZone:zone}).resolvedOptions().timeZone;
 const currentHour=Number(new Intl.DateTimeFormat('en-GB',{timeZone:zoneId(timeZone),hour:'2-digit',hourCycle:'h23'}).format(new Date(snapshot.now)));
 const start=weekly?clampWeeklyStart(day,selectedStart):day,first=calendarDate(start);
 if(first===null||start>day)return null;
 const span=(end-first)/dayMs+1;
 // Never merge multiple days into one bar; keep one daily column per calendar date.
 const days=Array.from({length:span},(_,i)=>{
  const d=isoDay(first+i*dayMs);
  return {day:d,end:d,total:0};
 });
 let detail=snapshot.report;
 if(detail?.day!==day||detail.period!==period||(detail.start||(weekly?weekStart(day):day))!==start||zoneId(detail.timezone)!==zoneId(timeZone))detail=null;
 if(!detail&&snapshot.demo)detail=demoReport(snapshot,start,day,today,currentHour,platform);
 const hours=Array(24).fill(0);
 let keyboard=0,mouse=0,peak=null,ties=0,activeHours=0;
 for(const [date,h,k,m] of detail?.hours||[]){
  const d=days.find(row=>date>=row.day&&date<=row.end);
  if(!d||!Number.isInteger(h)||h<0||h>23||(date===today&&h>currentHour))continue;
  const keys=positive(k),clicks=positive(m),total=keys+clicks;
  keyboard+=keys;mouse+=clicks;d.total+=total;hours[h]+=total;
  // Each source row is one calendar date and local clock hour, including DST folds.
  if(total)activeHours++;
  if(total && (!peak||total>peak.total)){peak={day:date,hour:h,total};ties=1;}
  else if(peak&&total===peak.total)ties++;
 }
 // The card can label recording-local counts; prompt facts must share the activity dates.
 const localRanking=(detail?.ranking||[]).filter(row=>row.platform===platform&&positive(row.count)).slice(0,5);
 const sameZone=zoneId(timeZone)===zoneId(systemTimeZone);
 const ranking=keyboard&&sameZone?localRanking:[];
 const topShortcut=(weekly||keyboard)?localRanking[0]??null:null;
 const highest=Math.max(...hours),hour=hours.indexOf(highest);
 // Each clock hour has the same denominator, including dates with no activity.
 const clockPeak=weekly&&highest>0&&hours.lastIndexOf(highest)===hour?{hour,dailyAverage:highest/span}:null;
 return {day,start,span,period,weekly,loaded:!!detail,ranking,topShortcut,shortcutByLocalDate:!sameZone,keyboard,mouse,days,hours,activeHours,peak,clockPeak,ties,partial,currentHour,demo:!!snapshot.demo};
}
function peakLabel(card){
 return (card.weekly?card.peak.day+'  ':'')+clock(card.peak.hour)+'–'+clock(card.peak.hour+1);
}
const Recipes=typeof module==='undefined'?ClickPulseReportRecipes:require('./report-recipes.js');
const Facts=typeof module==='undefined'?ClickPulseReportFacts:require('./report-facts.js');
const History=typeof module==='undefined'?ClickPulseHistory:require('./history.js');
function recipeContext(card){
 const values=card.weekly?card.days.map(day=>day.total):card.hours;
 return {kind:card.weekly?'week':'day',total:card.keyboard+card.mouse,partial:card.partial,sparse:values.filter(count=>count>0).length<2};
}
const focusWeights={shortcut:1,ranking:2,mix:1,peak:1,rhythm:2,book:1};
// The pool has at most six entries. Consider every feasible combination so a
// cheap first pick cannot crowd out a ranking or leave a useful point unused.
function selectFocus(available,budget,previous=[],reroll=false,random=Math.random){
 const maxItems=budget<=4?3:4;
 let choices=[{ids:[],cost:0}];
 for(const id of available){
  const weight=focusWeights[id];
  const additions=[];
  for(const {ids,cost} of choices){
   if(cost+weight>budget||ids.length>=maxItems)continue;
   if((id==='ranking'&&ids.includes('shortcut'))||(id==='shortcut'&&ids.includes('ranking')))continue;
   additions.push({ids:[...ids,id],cost:cost+weight});
  }
  choices.push(...additions);
 }
 if(reroll){
  const alternatives=choices.filter(({ids})=>ids.length&&(ids.length!==previous.length||ids.some(id=>!previous.includes(id))));
  if(alternatives.length)choices=alternatives;
 }
 const maxCost=Math.max(...choices.map(row=>row.cost));
 choices=choices.filter(row=>row.cost===maxCost);
 if(!reroll&&previous.length){
  const overlap=ids=>ids.filter(id=>previous.includes(id)).length;
  const best=Math.max(...choices.map(row=>overlap(row.ids)));
  choices=choices.filter(row=>overlap(row.ids)===best);
 }
 return choices[Math.floor(random()*choices.length)].ids;
}
// Canvas colors are independent of the application theme, including in exported PNGs.
const cardPalettes={
 blue:     {background:'#edf4ff',ink:'#172b4a',accent:'#2459a2',muted:'#63758e',line:'#d4dfef',corner:'#dce9fc',bar:'#90adcf'},
 paper:    {background:'#fbfaf6',ink:'#172b4a',accent:'#2459a2',muted:'#63758e',line:'#d4dfef',corner:'#e8e3d7',bar:'#90adcf'},
 ink:      {background:'#102543',ink:'#f4f7fc',accent:'#87b7ff',muted:'#a9bedc',line:'#29405f',corner:'#193456',bar:'#537499'},
 pine:     {background:'#eff5ef',ink:'#1c352a',accent:'#31664e',muted:'#50695a',line:'#c7d8cd',corner:'#dfebe1',bar:'#87ad98'},
 clay:     {background:'#fbf2e9',ink:'#412d24',accent:'#984d28',muted:'#775c4b',line:'#e8cfbc',corner:'#f2e1d1',bar:'#c79c80'},
 lavender: {background:'#f5f1f9',ink:'#332a42',accent:'#70538c',muted:'#6b5d78',line:'#dfd4e8',corner:'#eae2f2',bar:'#b29ac7'},
 rose:     {background:'#fcf1f3',ink:'#422b32',accent:'#99495c',muted:'#775a63',line:'#e9cfd6',corner:'#f4dfe5',bar:'#cb98a5'}
};
let promptEnglish;
function reportPrompt(card,recipe,focus,depth,i18n){
 if(!card?.loaded)return '';
 const L=i18n||(promptEnglish??=typeof module==='undefined'?createClickPulseI18n(window.clickpulseMessages,'en'):require('./i18n.js')(require('./locales.json'),'en'));
 const catalog=typeof module==='undefined'?ClickPulseShortcutCatalog:require('./shortcuts-catalog.js');
 const {t,number:n}=L,total=card.keyboard+card.mouse;
 const design=Recipes.describe(recipe,L.locale);
 const dates=[card.weekly?card.start+' — '+card.day:card.day];
 if(card.demo)dates.push(t('share.demo'));
 const facts=[dates.join(' · '),t('report_prompt.counts',{total:n(total),keyboard:n(card.keyboard),mouse:n(card.mouse)})];
 for(const id of focus){
  const section=Facts.section(id,card,L,catalog,History,depth);
  facts.push('',section.label,...section.lines);
 }
 const customRange=card.weekly&&card.span!==7;
 const series=card.weekly?[t(customRange?'report_prompt.range_series':'report_prompt.week'),...card.days.filter(d=>d.total>0).map(d=>d.day+'   '+n(d.total))]:
  [t('report.hours'),...card.hours.flatMap((value,hour)=>value>0?[clock(hour)+'   '+n(value)]:[])];
 if(card.partial&&!card.weekly)series.push(L.locale==='zh-CN'?'截至 '+clock(card.currentHour)+' 时段，之后时段尚未到来。':L.locale==='zh-TW'?'截至 '+clock(card.currentHour)+' 時段，之後時段尚未到來。':'Observed through the '+clock(card.currentHour)+' hour. Later hours have not yet occurred.');
 if(!total)series.push(L.locale==='zh-CN'?'该时间段内暂无输入记录。':L.locale==='zh-TW'?'該時間段內暫無輸入記錄。':'No inputs recorded in the observed period.');
 const seriesRule=customRange?'report_prompt.range_chart':card.weekly?'report_prompt.chart':'report_prompt.hour_chart';
 const direction=[design.medium,design.layout,design.chart,design.palette].map(part=>part.label+' — '+part.instruction);
 return [t(card.weekly?'report_prompt.weekly_brief':'report_prompt.brief'),direction.join('\n'),t('report_prompt.compose'),t(seriesRule),facts.join('\n'),series.join('\n'),t('report_prompt.output')].join('\n\n');
}
function shortcutRows(rows,C,query,platform){
 return (rows||[]).filter(r=>positive(r.count)&&r.platform===platform).map(r=>({...r,entries:C.matches(r)}))
 .filter(r=>!query||C.matchesQuery(r,r.entries,query))
 .sort((a,b)=>b.count-a.count||a.modifiers-b.modifiers||a.key-b.key);
}
// The input is already ranked and platform-filtered. Search only narrows the detail list.
function shortcutSummary(rows){
 const total=rows.reduce((sum,row)=>sum+row.count,0);
 return {total,top:rows.slice(0,8)};
}
function create({document,window,I,send,toast,syncDropdown=()=>{},getSystemTimeZone=()=>Intl.DateTimeFormat().resolvedOptions().timeZone}){
 const C=ClickPulseShortcutCatalog,$=id=>document.getElementById(id),t=(key,vars)=>I.t(key,vars),n=value=>I.number(value);
 const githubPath=document.querySelector('#project-home path')?.getAttribute('d');
 const githubMark=githubPath?new window.Path2D(githubPath):null;
 const startPicker=typeof ClickPulseDatePicker==='undefined'?null:ClickPulseDatePicker.create({document,window,I,input:$('share-start')});
 const datePicker=typeof ClickPulseDatePicker==='undefined'?null:ClickPulseDatePicker.create({document,window,I,input:$('share-date')});
 let paintVersion=0,painting=false,painted=false,exporting=false,suspended=false;
 const platform=window.clickpulsePlatform;
 const canCopy=()=>['macos','windows'].includes(platform)&&!window.clickpulsePreview;
 $('share-copy').hidden=!canCopy();
 let snapshot=null,active='',signature='',query='',style='blue',period='daily',selectedDay='',selectedStart='',followToday=true,card=null,visibleRows=100,listSignature='',listCounters=[],mouseButtonsSignature='';
 let shortcutChartSignature='';
 const node=(tag,text,cls)=>{const el=document.createElement(tag);if(text!==undefined)el.textContent=text;if(cls)el.className=cls;return el;};
 const svg=(tag,attrs)=>{const el=document.createElementNS('http://www.w3.org/2000/svg',tag);for(const [key,value]of Object.entries(attrs||{}))el.setAttribute(key,value);return el;};
 function invalidate(){signature='';if(snapshot)render(active,snapshot);}
 function remember(){window.captureSessionState?.();}
 const modKeyClass=key=>key==='⌘'||key==='Win'||key==='Super'?'mod-meta':key==='Ctrl'?'mod-ctrl':key==='Option'||key==='Alt'?'mod-alt':key==='Shift'?'mod-shift':'';
 function chordKeys(row){const keys=node('div',undefined,'shortcut-chord');for(const key of C.chordParts(row))keys.append(node('kbd',key,modKeyClass(key)||undefined));return keys;}
 function renderShortcutRanking(rows,emptyKey){
  const {total,top}=shortcutSummary(rows);
  const next=JSON.stringify([I.locale,total,top,emptyKey]);if(next===shortcutChartSignature)return;shortcutChartSignature=next;
  const percentage=value=>total?I.percent(value/total,1):'—';
  const ranking=top.map(row=>{
   const el=node('div',undefined,'shortcut-rank-row'),label=chordKeys(row),track=node('div',undefined,'shortcut-rank-track'),bar=node('div',undefined,'shortcut-rank-bar'),value=node('div',undefined,'shortcut-rank-value');
   if(row.entries?.length)el.title=row.entries.map(e=>C.title(e,I.locale)+' · '+C.scope(e,I.locale)).join(' / ');
   bar.style.width=(row.count/top[0].count*100)+'%';track.setAttribute('aria-hidden','true');track.append(bar);
   value.append(node('strong',n(row.count)),node('small',percentage(row.count)));el.append(label,track,value);return el;
  });
  $('shortcut-ranking').replaceChildren(...ranking);
  if(!total){const empty=node('div',undefined,'shortcut-chart-empty');empty.append(node('strong',t('shortcuts.chart_empty')),node('p',t(emptyKey)));$('shortcut-ranking').append(empty);}
 }
 function renderShortcuts(){
  const pending=!Array.isArray(snapshot.shortcuts);
  const local=shortcutRows(snapshot.shortcuts,C,'',platform);
  // Shift alone commonly selects text; combinations with other modifiers remain visible.
  const all=local.filter(row=>row.modifiers!==4);
  const emptyKey=!local.length&&!query?'shortcuts.empty':'shortcuts.no_results';
  $('shortcut-dashboard').hidden=pending;
  $('shortcut-loading').hidden=!pending;$('shortcut-details').hidden=pending;
  $('shortcut-details-title').textContent=t('shortcuts.details');
  if(!pending)renderShortcutRanking(all,emptyKey);
  $('shortcut-search').placeholder=t('shortcuts.search');$('shortcut-search').setAttribute('aria-label',t('shortcuts.search'));
  if(pending)return;
  const rows=all.filter(r=>!query||C.matchesQuery(r,r.entries,query));
  const more=$('shortcut-more');more.hidden=rows.length<=visibleRows;more.textContent=t('shortcuts.more',{count:n(Math.max(0,rows.length-visibleRows))});
  const shown=rows.slice(0,visibleRows);
  const nextList=JSON.stringify([query,I.locale,visibleRows,emptyKey,shown.map(r=>[r.platform,r.modifiers,r.key])]);
  const max=Math.max(1,...all.map(r=>r.count));
  if(nextList!==listSignature){
  listSignature=nextList;listCounters=[];
  $('shortcut-list').replaceChildren(...shown.map(row=>{
   const el=node('div',undefined,'shortcut-row');el.append(chordKeys(row));const annotation=node('div',undefined,'shortcut-annotation');
   if(row.entries.length){for(const entry of row.entries){annotation.append(node('strong',C.title(entry,I.locale)),node('small',C.scope(entry,I.locale),'shortcut-scope'));}}
   else {
    const suggest=node('button',t('shortcuts.suggest'),'shortcut-source');
    suggest.type='button';
    suggest.onclick=()=>{
     const {platform,modifiers,key}=row;
     if(!window.clickpulsePreview){send('suggest_shortcut',{platform,modifiers,key});return;}
     const url=new URL('https://github.com/endNone/ClickPulse/issues/new');
     url.search=new URLSearchParams({template:'shortcut-suggestion.yml',platform,chord:C.chordParts(row).join(' + ')});
     window.open(url.href,'_blank','noopener,noreferrer');
    };
    annotation.append(node('small',t('shortcuts.unknown')),suggest);
   }
   el.append(annotation);const count=node('div',undefined,'shortcut-count');
   const label=node('span'),track=node('i'),bar=node('b');track.append(bar);count.append(label,track);listCounters.push({label,bar});el.append(count);return el;
  }));
  if(!rows.length)$('shortcut-list').append(node('div',t(emptyKey),'insight-empty'));
  }
  shown.forEach((row,i)=>{listCounters[i].label.textContent=n(row.count);listCounters[i].bar.style.width=(row.count/max*100)+'%';});
 }
 function renderMouse(){
  const rows=mouseBreakdown(snapshot),total=rows.reduce((sum,row)=>sum+row.count,0);
  const themeName=document.documentElement?.dataset?.theme||'blue';
  const buttonsSignature=JSON.stringify([rows,I.locale,themeName]);
  if(buttonsSignature===mouseButtonsSignature)return;
  mouseButtonsSignature=buttonsSignature;
  const palette=typeof ClickPulseThemes!=='undefined'?ClickPulseThemes.get(themeName):{empty:'#edf4ff',full:'#1f53a0',accent:'#2563eb'};
  const rgb=hex=>[1,3,5].map(i=>parseInt(hex.slice(i,i+2),16));
  const from=rgb(palette.empty),to=rgb(palette.accent);
  const colors={};
  for(const {key,count} of rows){
   // A cube-root scale keeps infrequent buttons distinguishable when one dominates.
   const share=total ? count/total : 0;
   const mix=total ? 0.08 + 0.92 * Math.cbrt(share) : 0;
   const [r,g,b]=from.map((v,i)=>Math.round(v+(to[i]-v)*mix));
   colors[key]=`rgb(${r}, ${g}, ${b})`;
  }
  const percentage=count=>{
   if(!total)return '—';
   const share=count/total;
   return share>0 && share<0.0005 ? '<'+I.percent(0.001,1) : I.percent(share,1);
  };
  const caption=row=>t('chart.mouse_'+row.key)+': '+n(row.count)+' · '+percentage(row.count);
  const ring=node('div',undefined,'mouse-ring'),chart=svg('svg',{viewBox:'0 0 200 200',role:'img','aria-label':t('mouse.buttons')});
  chart.append(svg('circle',{cx:100,cy:100,r:82,fill:'none',stroke:palette.empty,'stroke-width':23}));
  let offset=0;
  for(const row of rows.filter(row=>row.count)){
   const share=row.count/total*100,arc=svg('circle',{cx:100,cy:100,r:82,fill:'none',stroke:colors[row.key],'stroke-width':23,pathLength:100,'stroke-dasharray':share+' 100','stroke-dashoffset':-offset,transform:'rotate(-90 100 100)'});
   const title=svg('title');title.textContent=caption(row);arc.append(title);chart.append(arc);offset+=share;
  }
  const leader=rows.reduce((best,row)=>row.count>best.count?row:best,rows[0]),center=node('div',undefined,'mouse-ring-label');
  center.append(node('strong',total?I.percent(leader.count/total,1):'—'),node('span',total?t('chart.mouse_'+leader.key):t('common.empty')));
  ring.append(chart,center);$('mouse-chart').replaceChildren(ring);
  const maxRow=Math.max(1,...rows.map(r=>r.count));
  const rowEls=new Map(),shapeEls=new Map();
  const focusKey=key=>{
   for(const [k,el]of rowEls)el.classList?.toggle('is-active',k===key);
   for(const [k,el]of shapeEls)el.classList?.toggle('is-active',k===key);
  };
  $('mouse-distribution').replaceChildren(...rows.filter(row=>row.count>0||row.key==='left'||row.key==='right').map(row=>{
   const el=node('div',undefined,'mouse-row'),label=node('span',undefined,'mouse-label'),swatch=node('i');swatch.style.backgroundColor=colors[row.key];swatch.setAttribute('aria-hidden','true');
   label.append(swatch,node('span',t('chart.mouse_'+row.key)));
   const track=node('div',undefined,'mouse-bar'),fill=node('b');
   fill.style.width=(total?row.count/maxRow*100:0)+'%';fill.style.backgroundColor=colors[row.key];
   track.setAttribute('aria-hidden','true');track.append(fill);
   el.append(label,node('strong',n(row.count)),node('small',percentage(row.count)),track);
   el.addEventListener?.('mouseenter',()=>focusKey(row.key));
   el.addEventListener?.('mouseleave',()=>focusKey(null));
   rowEls.set(row.key,el);
   return el;
  }));
  const drawing=svg('svg',{viewBox:'0 0 200 270',class:'mouse-drawing',role:'img','aria-label':t('mouse.buttons')});
  const ink={shell:'#f9fbff',edge:palette.accent+'42',shadow:palette.accent+'18'};
  drawing.append(svg('ellipse',{cx:100,cy:253,rx:52,ry:9,fill:ink.shadow}));
  const side=svg('rect',{x:31,y:110,width:10,height:38,rx:5,fill:colors.side,stroke:ink.edge,'stroke-width':1.2,'data-mouse-button':'side'}),sideTitle=svg('title');sideTitle.textContent=caption(rows.find(row=>row.key==='side'));side.append(sideTitle);drawing.append(side);shapeEls.set('side',side);
  drawing.append(svg('path',{d:'M100 17C65 17 43 43 41 82L35 166C32 211 55 246 100 246S168 211 165 166L159 82C157 43 135 17 100 17Z',fill:ink.shell,stroke:ink.edge,'stroke-width':1.6}));
  for(const [key,d]of [['left','M97 24C67 25 49 46 47 81L45 108C61 114 79 117 97 117Z'],['right','M103 24C133 25 151 46 153 81L155 108C139 114 121 117 103 117Z']]){
   const shape=svg('path',{d,fill:colors[key],'data-mouse-button':key}),title=svg('title');title.textContent=caption(rows.find(row=>row.key===key));shape.append(title);drawing.append(shape);shapeEls.set(key,shape);
  }
  const wheel=svg('g'),wheelTitle=svg('title');wheelTitle.textContent=caption(rows.find(row=>row.key==='middle'));
  const wheelRect=svg('rect',{x:91,y:45,width:18,height:40,rx:9,fill:colors.middle,stroke:ink.edge,'stroke-width':1.2,'data-mouse-button':'middle'});
  wheel.append(wheelTitle,wheelRect);
  for(const y of [57,63,69])wheel.append(svg('path',{d:'M96.5 '+y+'h7',stroke:'#ffffffcc','stroke-width':1.2,'stroke-linecap':'round'}));
  drawing.append(wheel);shapeEls.set('middle',wheelRect);
  for(const [k,el]of shapeEls){
   el.addEventListener?.('mouseenter',()=>focusKey(k));
   el.addEventListener?.('mouseleave',()=>focusKey(null));
  }
  $('mouse-diagram').replaceChildren(drawing);
 }
 function rounded(ctx,x,y,w,h,r){r=Math.min(r,w/2,h/2);ctx.beginPath();ctx.moveTo(x+r,y);ctx.arcTo(x+w,y,x+w,y+h,r);ctx.arcTo(x+w,y+h,x,y+h,r);ctx.arcTo(x,y+h,x,y,r);ctx.arcTo(x,y,x+w,y,r);ctx.closePath();}
 function drawCard(){
  if(!card)return;const canvas=$('share-canvas');if(canvas.width!==1080||canvas.height!==1350){canvas.width=1080;canvas.height=1350;}const ctx=canvas.getContext('2d');if(!ctx)return;
  const {background,ink,accent,muted,line,corner,bar}=cardPalettes[style];
  const regional=I.locale==='zh-TW'?'"PingFang TC", "Microsoft JhengHei", "Noto Sans CJK TC"':I.locale==='ja'?'"Hiragino Sans", "Yu Gothic", "Noto Sans CJK JP"':I.locale==='ko'?'"Apple SD Gothic Neo", "Malgun Gothic", "Noto Sans CJK KR"':'"PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC"';
  const sans='"ClickPulse UI", '+regional+', system-ui, sans-serif';
  ctx.fillStyle=background;ctx.fillRect(0,0,1080,1350);
  // Subtle geometric corner accents + fine editorial poster frame
  ctx.fillStyle=corner;ctx.beginPath();ctx.arc(1060,-10,410,0,Math.PI*2);ctx.fill();
  ctx.save();ctx.globalAlpha=0.55;ctx.fillStyle=corner;ctx.beginPath();ctx.arc(40,1320,260,0,Math.PI*2);ctx.fill();ctx.restore();
  ctx.strokeStyle=line;ctx.lineWidth=2;rounded(ctx,28,28,1024,1294,32);ctx.stroke();
  function text(value,x,y,size,weight=400,color=ink,max=952,align='left'){ctx.font=weight+' '+size+'px '+sans;ctx.fillStyle=color;ctx.textAlign=align;while(ctx.measureText(value).width>max&&size>14){size--;ctx.font=weight+' '+size+'px '+sans;}ctx.fillText(value,x,y,max);}
  const brand=document.querySelector('.brand-icon img');
  const hasLogo=brand?.complete&&brand.naturalWidth>0;
  if(hasLogo){
   ctx.fillStyle='#fff';rounded(ctx,64,56,72,72,18);ctx.fill();
   ctx.drawImage(brand,72,64,56,56);
  }
  text('ClickPulse',hasLogo?152:64,104,36,750);
  text(t(card.weekly?'report.weekly':'report.daily'),1016,101,26,650,accent,320,'right');
  if(!card.loaded){
   const loading=t('common.loading');text(loading,64,300,40,500,muted);canvas.setAttribute('aria-label',loading);return false;
  }
  const dates=card.weekly?I.day(card.start)+' — '+I.day(card.day):I.day(card.day,{year:'numeric',month:'long',day:'numeric'});
  text(dates,64,208,38,600,ink,952);
  const total=card.keyboard+card.mouse;
  const unit='keys & clicks';
  text(n(total),60,372,126,750,accent);
  text(unit,66,436,28,500,muted);
  const kbShare=total?card.keyboard/total:0,msShare=total?card.mouse/total:0;
  for(const [x,value,label,share,dotColor]of [
   [64,card.keyboard,t('common.keyboard'),kbShare,accent],
   [552,card.mouse,t('common.mouse'),msShare,bar]
  ]){
   ctx.fillStyle=dotColor;rounded(ctx,x+26,482,12,12,4);ctx.fill();
   text(label,x+48,498,26,550,muted,260);
   text(total?I.percent(share,1):'—',x+438,498,24,600,muted,120,'right');
   text(n(value),x+26,556,48,700,ink,412);
  }
  ctx.fillStyle=line;rounded(ctx,64,596,952,10,5);ctx.fill();
  if(total>0){
   const kbW=Math.round(952*kbShare);
   if(kbW>0){ctx.fillStyle=accent;rounded(ctx,64,596,Math.max(10,kbW),10,5);ctx.fill();}
   if(952-kbW>0){ctx.fillStyle=bar;rounded(ctx,64+kbW+(kbW>0&&kbW<952?4:0),596,Math.max(0,952-kbW-(kbW>0&&kbW<952?4:0)),10,5);ctx.fill();}
  }
  text(t(card.weekly?'report.days':'report.hours'),96,684,30,650,ink,600);
  const values=card.weekly?card.days.map(d=>d.total):card.hours;
  const chartX=96,chartW=888,baseline=876,chartH=128;
  const max=Math.max(1,...values),step=chartW/values.length;
  const barW=Math.max(8,Math.min(step*(values.length<=7?0.52:0.68),64));
  ctx.save();ctx.strokeStyle=line;ctx.lineWidth=1.5;ctx.setLineDash([6,6]);
  ctx.beginPath();ctx.moveTo(chartX,baseline-chartH/2);ctx.lineTo(chartX+chartW,baseline-chartH/2);ctx.stroke();
  ctx.restore();
  // Keep every time slot; annotate the first maximum only when observed counts vary.
  const observed=card.partial&&!card.weekly?values.slice(0,card.currentHour+1):values;
  const peakIdx=observed.some(value=>value!==max)?observed.indexOf(max):-1;
  const multiYear=card.weekly&&card.start.slice(0,4)!==card.day.slice(0,4);
  const formatTick=day=>I.day(day,{month:'numeric',day:'numeric',...(multiYear?{year:'2-digit'}:{})});
  const tickSet=new Set();
  if(card.weekly){
   if(values.length<=7){for(let i=0;i<values.length;i++)tickSet.add(i);}
   else{
    const tickCount=Math.min(6,values.length);
    for(let tIdx=0;tIdx<tickCount;tIdx++)tickSet.add(Math.round(tIdx*(values.length-1)/(tickCount-1)));
   }
  }
  values.forEach((value,i)=>{
   const x=chartX+(i+.5)*step,height=value/max*chartH;
   if(height>0){
    ctx.fillStyle=peakIdx>=0&&value===max?accent:bar;
    rounded(ctx,x-barW/2,baseline-height,barW,height,Math.min(6,barW/2,height/2));ctx.fill();
   }
   if(i===peakIdx){
    const edge=step<120&&(i===0||i===values.length-1);
    const align=edge?(i===0?'left':'right'):'center';
    const tx=edge?(i===0?chartX:chartX+chartW):x;
    text(n(value),tx,baseline-height-12,24,700,accent,180,align);
   }
   if(tickSet.has(i)){
    const align=values.length>7?(i===0?'left':i===values.length-1?'right':'center'):'center';
    const tx=values.length>7?(i===0?chartX:i===values.length-1?chartX+chartW:x):x;
    text(formatTick(card.days[i].day),tx,baseline+38,24,500,muted,values.length<=7?step-4:130,align);
   }
  });
  if(!card.weekly){
   for(const hour of [0,6,12,18,24]){
    text(clock(hour),chartX+step*hour,baseline+38,24,500,muted,116,hour===0?'left':hour===24?'right':'center');
   }
  }
  ctx.strokeStyle=line;ctx.lineWidth=2;ctx.beginPath();ctx.moveTo(chartX,baseline);ctx.lineTo(chartX+chartW,baseline);ctx.stroke();
  let rowY=1018;
  function summaryRow(label,value){
   ctx.font='650 28px '+sans;
   const valueWidth=Math.min(536,Math.ceil(ctx.measureText(value).width)+1);
   text(label,92,rowY,26,550,muted,988-92-24-valueWidth);
   text(value,988,rowY,28,650,ink,valueWidth,'right');
   rowY+=52;
  }
  if(card.peak)summaryRow(t(card.ties>1?'report.peak_tied':'report.peak'),peakLabel(card)+'  ·  '+t('share.shortcut_count',{count:n(card.peak.total)}));
  const busiestSlot=card.clockPeak?Facts.clockPeakValue(card.clockPeak,I):'';
  if(busiestSlot)summaryRow(t('report.clock_peak'),busiestSlot);
  const top=card.topShortcut;
  const shortcutLabel=t(top&&card.shortcutByLocalDate?'share.top_shortcut_local_date':'share.top_shortcut');
  let shortcutValue='';
  if(top)shortcutValue=C.chordParts(top).join(' + ')+'  ·  '+t('share.shortcut_count',{count:n(top.count)});
  else if(card.weekly)shortcutValue=t('share.no_shortcuts_in_period');
  if(shortcutValue)summaryRow(shortcutLabel,shortcutValue);
  const url='github.com/endNone/ClickPulse';
  ctx.font='500 30px '+sans;
  const markX=1008-ctx.measureText(url).width-12-32;
  ctx.fillStyle=muted;
  if(githubMark){
   ctx.save();ctx.translate(markX,1256);ctx.scale(2,2);ctx.fill(githubMark);ctx.restore();
  }
  ctx.textAlign='right';ctx.fillText(url,1008,1282);
  if(card.demo)text(t('share.demo'),72,1282,22,500,muted,Math.min(580,markX-72-24));
  canvas.setAttribute('aria-label',[dates,n(total)+' '+unit,card.peak?peakLabel(card):'',busiestSlot?t('report.clock_peak')+' · '+busiestSlot:'',shortcutValue?shortcutLabel+' · '+shortcutValue:''].filter(Boolean).join(' · '));
  return true;
 }
 function shareButtons(){
  $('share-save').disabled=suspended||painting||!painted||exporting||!card?.loaded;
  $('share-copy').hidden=!canCopy();
  $('share-copy').disabled=suspended||painting||!painted||exporting||!card?.loaded||!canCopy();
 }
 const promptPanel=$('share-prompt-panel'),promptText=$('share-prompt-text'),promptCopy=$('share-prompt-copy');
 const regenerate=$('share-prompt-regenerate');
 const picker=Recipes.createPicker();
 let promptReport='',promptDraft=null,focusKey='',focus=[],rankingDepth=2;
 function selectPromptFocus(recipe,reroll){
  const budget=Math.max(Recipes.registry.medium[recipe.medium].budget+2,5),available=Facts.available(card,History);
  const key=[budget,...available].join('/');
  if(reroll||focusKey!==key){
   const selected=selectFocus(available,budget,focus,reroll);
   if(selected.includes('ranking')&&(reroll||!focus.includes('ranking')))rankingDepth=3;
   focus=selected;focusKey=key;
  }
  return focus;
 }
 function updatePrompt(reroll=false){
  let value='';
  const ready=promptPanel.open&&active==='share'&&!suspended&&card?.loaded;
  if(ready){
   const report=JSON.stringify([card.period,card.start,card.day,I.effectiveTimeZone,card.demo]);
   reroll=reroll||report!==promptReport;
   if(reroll)promptDraft=null;
   promptReport=report;
   if(promptDraft===null){
    const context=recipeContext(card);
    const recipe=reroll?picker.next(context):picker.adapt(context);
    const selected=selectPromptFocus(recipe,reroll);
    value=reportPrompt(card,recipe,selected,rankingDepth,I);
   }else value=promptDraft;
  }
  if(promptText.value!==value){
   const allSelected=document.activeElement===promptText&&promptText.value&&promptText.selectionStart===0&&promptText.selectionEnd===promptText.value.length;
   promptText.value=value;
   if(allSelected&&value)promptText.select();
  }
  promptCopy.disabled=!value.trim();regenerate.disabled=!ready;
  promptText.disabled=!ready;
  promptText.placeholder=ready?'':t('common.loading');
 }
 promptText.addEventListener('input',()=>{
  promptDraft=promptText.value;
  promptCopy.disabled=!promptDraft.trim();
 });
 promptPanel.addEventListener('toggle',()=>updatePrompt());
 regenerate.addEventListener('click',()=>{
  if(regenerate.disabled)return;
  updatePrompt(true);
  promptText.scrollTop=0;
  toast(t('report_prompt.regenerated'));
 });
 promptCopy.addEventListener('click',()=>{
  if(!promptText.value||promptCopy.disabled)return;
  promptText.focus();
  promptText.select();
  // WebView origins do not consistently expose the asynchronous clipboard API.
  // Keep the text selected for the system Copy command if the write is denied.
  let copied=false;
  try {copied=document.execCommand('copy');} catch {}
  if(copied)promptCopy.focus();
  toast(t(copied?'report_prompt.copied':'report_prompt.manual'));
 });
 function releaseCard(){
  // Invalidate pending font/logo work before releasing the backing bitmap.
  paintVersion++;painting=false;painted=false;card=null;signature='';
  const canvas=$('share-canvas');canvas.width=0;canvas.height=0;shareButtons();updatePrompt();
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
  const weekly=period==='weekly',input=$('share-date'),start=$('share-start');
  $('share-workspace').dataset.period=period;
  input.dataset.numericDate=String(weekly);
  $('share-start-field').hidden=!weekly;
  const minStart=weekly?shiftDay(selectedDay,-(MAX_WEEKLY_DAYS-1)):'';
  const maxStart=weekly?shiftDay(selectedDay,-(MIN_WEEKLY_DAYS-1)):'';
  start.disabled=!weekly;start.min=minStart;start.max=maxStart;start.dataset.today=today;
  start.value=weekly?card.start:'';
  const minEnd=weekly?shiftDay(card.start,MIN_WEEKLY_DAYS-1):'';
  const maxEndRaw=weekly?shiftDay(card.start,MAX_WEEKLY_DAYS-1):today;
  input.min=minEnd;input.max=weekly&&maxEndRaw<today?maxEndRaw:today;
  if(document.activeElement!==input)input.value=selectedDay;
  startPicker?.sync();datePicker?.sync();updatePrompt();
  $('share-empty').hidden=!card?.loaded||!!(card.keyboard||card.mouse||card.topShortcut);
  $('share-date-caption').textContent=t(period==='weekly'?'report.end_date':'share.date');
  for(const button of periodButtons)button.setAttribute('aria-pressed',String(button.dataset.reportPeriod===period));
  if(styleSelect.value!==style){styleSelect.value=style;syncDropdown(styleSelect);}
  syncShareDeck();
  void paintCard();
 }
 const styleOrder=Object.keys(cardPalettes);
 let deckDots=[];
 const setCssVar=(el,name,value)=>{if(el.style?.setProperty)el.style.setProperty(name,value);else if(el.style)el.style[name]=value;};
 function setPeek(el,nameEl,key){
  if(!el)return;
  const p=cardPalettes[key],label=t('share.'+key);
  setCssVar(el,'--peek-bg',p.background);
  setCssVar(el,'--peek-corner',p.corner);
  setCssVar(el,'--peek-accent',p.accent);
  setCssVar(el,'--peek-bar',p.bar);
  setCssVar(el,'--peek-ink',p.ink);
  setCssVar(el,'--peek-line',p.line);
  el.setAttribute('aria-label',label);
  el.title=label;
  if(nameEl)nameEl.textContent=label;
 }
 function syncShareDeck(){
  const idx=Math.max(0,styleOrder.indexOf(style));
  const prev=styleOrder[(idx-1+styleOrder.length)%styleOrder.length];
  const next=styleOrder[(idx+1)%styleOrder.length];
  setPeek($('share-card-prev'),$('share-prev-name'),prev);
  setPeek($('share-card-next'),$('share-next-name'),next);
  const dotsRoot=$('share-deck-dots');
  if(!dotsRoot)return;
  if(!deckDots.length){
   deckDots=styleOrder.map(key=>{
    const btn=node('button',undefined,'share-deck-dot');
    btn.type='button';
    const swatch=cardPalettes[key].background==='#fbfaf6'?'#e8e3d7':cardPalettes[key].background==='#102543'?'#102543':cardPalettes[key].accent;
    setCssVar(btn,'--dot-color',swatch);
    setCssVar(btn,'--dot-ring',cardPalettes[key].accent);
    btn.addEventListener('click',()=>{
     if(style===key)return;
     const dir=styleOrder.indexOf(key)>styleOrder.indexOf(style)?1:-1;
     applyStyle(key,dir);
    });
    return btn;
   });
   dotsRoot.replaceChildren(...deckDots);
  }
  deckDots.forEach((btn,i)=>{
   const key=styleOrder[i],label=t('share.'+key);
   btn.setAttribute('aria-pressed',String(key===style));
   btn.setAttribute('aria-label',label);
   btn.title=label;
  });
 }
 function applyStyle(next,direction=0){
  if(!Object.prototype.hasOwnProperty.call(cardPalettes,next)||next===style)return;
  style=next;
  if(styleSelect.value!==style){styleSelect.value=style;syncDropdown(styleSelect);}
  const stage=$('share-stage');
  if(stage&&direction&&stage.classList){
   stage.classList.remove('swap-left','swap-right');
   void stage.offsetWidth;
   stage.classList.add(direction>0?'swap-right':'swap-left');
  }
  invalidate();remember();
 }
 function stepStyle(delta){
  const idx=Math.max(0,styleOrder.indexOf(style));
  applyStyle(styleOrder[(idx+delta+styleOrder.length)%styleOrder.length],delta);
 }
 $('share-card-prev')?.addEventListener('click',()=>stepStyle(-1));
 $('share-card-next')?.addEventListener('click',()=>stepStyle(1));
 $('share-deck-prev')?.addEventListener('click',()=>stepStyle(-1));
 $('share-deck-next')?.addEventListener('click',()=>stepStyle(1));
 $('share-deck')?.addEventListener('keydown',event=>{
  if(event.key==='ArrowLeft'){event.preventDefault();stepStyle(-1);}
  else if(event.key==='ArrowRight'){event.preventDefault();stepStyle(1);}
 });
 $('shortcut-search').addEventListener('input',e=>{query=e.target.value;visibleRows=100;invalidate();});
 $('shortcut-atlas')?.addEventListener('click',()=>{
  if(window.clickpulsePreview)window.open('https://endnone.github.io/ClickPulse/','_blank','noopener,noreferrer');
  else send('shortcut_atlas');
 });
 $('shortcut-more').addEventListener('click',()=>{visibleRows=Math.min(6000,visibleRows+100);invalidate();remember();});
 const periodButtons=document.querySelectorAll('[data-report-period]');
 for(const button of periodButtons)button.addEventListener('click',()=>{
  if(period===button.dataset.reportPeriod)return;
  period=button.dataset.reportPeriod;invalidate();remember();
 });
 $('share-start').addEventListener('change',e=>{
  const value=e.target.value;
  const earliest=selectedDay?shiftDay(selectedDay,-(MAX_WEEKLY_DAYS-1)):'0001-01-01';
  const latest=selectedDay?shiftDay(selectedDay,-(MIN_WEEKLY_DAYS-1)):'0001-01-01';
  if(period!=='weekly'||!snapshot||calendarDate(value)===null||value<earliest||value>latest){e.target.value=card?.start||'';return;}
  selectedStart=value;invalidate();remember();
 });
 $('share-date').addEventListener('change',e=>{
  if(!snapshot)return;
  const today=dayKey(snapshot.now,I.effectiveTimeZone),value=e.target.value;
  const minEnd=period==='weekly'&&card?shiftDay(card.start,MIN_WEEKLY_DAYS-1):'0001-01-01';
  const maxEndRaw=period==='weekly'&&card?shiftDay(card.start,MAX_WEEKLY_DAYS-1):today;
  const maxEnd=period==='weekly'&&maxEndRaw<today?maxEndRaw:today;
  if(!value||value>maxEnd||value<minEnd||calendarDate(value)===null||(e.target.validity&&!e.target.validity.valid)){e.target.value=selectedDay||today;return;}
  if(period==='weekly'&&!selectedStart&&value!==selectedDay)selectedStart=card.start;
  selectedDay=value;followToday=selectedDay===today;invalidate();remember();
 });
 const styleSelect=$('share-style-select');
 styleSelect.addEventListener('change',()=>{
  if(!Object.prototype.hasOwnProperty.call(cardPalettes,styleSelect.value))return;
  style=styleSelect.value;invalidate();remember();
 });
 async function exportCard(copy){
  if(suspended||active!=='share'||!card?.loaded||exporting||painting||!painted||(copy&&!canCopy()))return;
  exporting=true;shareButtons();
  try {
   // The current bitmap is already font-ready. Capture it synchronously before
   // any hide/Mini transition; the native job owns the PNG after IPC handoff.
   const png=$('share-canvas').toDataURL('image/png');
   if(window.clickpulsePreview){const a=node('a');a.href=png;a.download='ClickPulse-'+(card.weekly?'Weekly-'+card.start+'_':'')+card.day+'.png';a.click();}
   else send(copy?'copy_share_card':'save_share_card',{png,day:card.day,...(!copy&&card.weekly?{period:'weekly',start:card.start}:{})});
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
  if(page!=='share'){datePicker?.close();startPicker?.close();}
  if(active==='share'&&page!=='share')releaseCard();
  snapshot=data;active=page;if(!['shortcuts','mouse','share'].includes(page))return;
  if(suspended)return;
  let today='';
  if(page==='share'){
   today=dayKey(data.now,I.effectiveTimeZone);
   if(!selectedDay||followToday||selectedDay>today){const changed=!followToday&&!!selectedDay;selectedDay=today;followToday=true;if(changed)remember();}
   if(period==='weekly'&&selectedStart>selectedDay)selectedStart='';
   card=cardData(data,selectedDay,I.effectiveTimeZone,getSystemTimeZone(),period,platform,selectedStart);
  }
  const next=JSON.stringify([page,I.locale,I.effectiveTimeZone,document.documentElement.dataset.theme,query,style,selectedDay,today,data.demo,page==='shortcuts'?data.shortcuts:page==='mouse'?[data.lifetime?.mouse,data.mouse_buttons]:card]);
  if(next===signature)return;signature=next;
  if(page==='shortcuts')renderShortcuts();else if(page==='mouse')renderMouse();else renderShare();
 }
 function sessionState(){return {shortcut_rows:visibleRows,share_style:style,share_period:period,share_start:selectedStart||null,share_day:followToday?null:selectedDay};}
 function restoreSession(state){
  if(!state)return;
  visibleRows=Number.isInteger(state.shortcut_rows)&&state.shortcut_rows>=100&&state.shortcut_rows<=6000&&state.shortcut_rows%100===0?state.shortcut_rows:100;
  style=Object.prototype.hasOwnProperty.call(cardPalettes,state.share_style)?state.share_style:'blue';
  period=state.share_period==='weekly'?'weekly':'daily';
  selectedStart=calendarDate(state.share_start)!==null&&state.share_start>='0001-01-01'?state.share_start:'';
  selectedDay=calendarDate(state.share_day)!==null&&state.share_day>='0001-01-01'?state.share_day:'';followToday=!selectedDay;
  signature='';listSignature='';
 }
 releaseCard();
 return {render,sessionState,restoreSession,setWindowState};
}
return {create,mouseBreakdown,dayKey,cardData,recipeContext,selectFocus,reportPrompt,shortcutRows,shortcutSummary};
})();
if(typeof module!=='undefined')module.exports=ClickPulseInsights;
