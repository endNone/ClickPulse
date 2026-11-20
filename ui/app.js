'use strict';
// Keep browser UI out of the desktop app; text editing retains its native menu.
document.addEventListener('contextmenu',event=>{
  const target=event.target;
  if(!(target instanceof Element)||!target.closest('input,textarea,[contenteditable="true"],[contenteditable=""]'))event.preventDefault();
});
const $=id=>document.getElementById(id);
// Keep the Mac control outside both the scrolling pane and the collapsible sidebar.
if(window.clickpulsePlatform==='macos')document.body.append($('sidebar-toggle'));
// Mac puts the title in its toolbar; other platforms keep it above the content.
// Expose exactly the visible heading to assistive technology.
$('toolbar-title').setAttribute('aria-hidden',String(window.clickpulsePlatform!=='macos'));
$('page-title').setAttribute('aria-hidden',String(window.clickpulsePlatform==='macos'));
function setSelectableText(node,value){
  value=String(value);
  if(node.textContent===value)return;
  const selection=window.getSelection?.();
  const text=node.firstChild;
  if(text?.nodeType!==3||node.childNodes.length!==1){node.textContent=value;return;}
  if(!selection||selection.isCollapsed||!selection.rangeCount||!selection.setBaseAndExtent||!selection.getRangeAt(0).intersectsNode(node)){text.data=value;return;}
  // Retain selection (including backwards selections) without freezing live counts.
  const {anchorNode,anchorOffset,focusNode,focusOffset}=selection,oldLength=text.data.length;
  const offset=(container,position)=>container===text?(position===oldLength?value.length:Math.min(position,value.length)):position;
  text.data=value;
  selection.setBaseAndExtent(anchorNode,offset(anchorNode,anchorOffset),focusNode,offset(focusNode,focusOffset));
}
window.setSelectableText=setSelectableText;
const I=ClickPulseI18n;
const literaryEnglish=createClickPulseI18n(window.clickpulseMessages,'en',window.clickpulsePlatform);
const T=ClickPulseThemes;
let currentTheme=T.get(window.clickpulseTheme).name;
T.apply(document.documentElement,currentTheme);
const t=(key,values={})=>I.t(key,values);
const n=value=>I.number(value);
I.apply(document);
$('theme-select').value=currentTheme;
for(const option of $('theme-select').options)option.dataset.swatch=T.get(option.value).accent;
const Dropdowns=ClickPulseDropdowns.create(document,window,t);
const total=c=>(c?.keyboard||0)+(c?.mouse||0);
const send=(action,values={})=>window.ipc?.postMessage(JSON.stringify({action,...values}));
let miniData=null;
let data=null,page='overview',range=14,toastTimer;
let historyRevision=0;
const historyValues=[];
function updateHistoryRevision(rows=[]){
  let changed=historyValues.length!==rows.length;
  rows.forEach((row,index)=>{
    const old=historyValues[index];
    if(!old||old.day!==row.day||old.keyboard!==row.counts.keyboard||old.mouse!==row.counts.mouse){
      historyValues[index]={day:row.day,keyboard:row.counts.keyboard,mouse:row.counts.mouse};changed=true;
    }
  });
  historyValues.length=rows.length;
  if(changed)historyRevision++;
}
let sidebarCollapsed=false;
let recordingPaused=null;
let dataOperationBusy=false;
let updateState={status:'idle',current:'',version:null,can_install:false,update_command:null,automatic:true,received:0,total:0};
const updateBusy=()=>['downloading','verifying','preparing','installing'].includes(updateState.status);
const updateInstalling=()=>['preparing','installing'].includes(updateState.status);
function renderUpdateState(){
  const button=$('check-updates'),status=$('update-status'),details=$('update-details');
  if(!button||!status||!details)return;
  const state=updateState,available=['available','ready'].includes(state.status),managed=!!state.update_command;
  const key={checking:'update.checking',current:'update.current',ready:'update.ready',available:managed||state.can_install?'update.available':'update.manual',error:'update.failed',limited:'update.limited',verifying:'update.verifying',preparing:'update.preparing',installing:'update.restarting'}[state.status];
  const percent=state.total>0?Math.min(100,Math.floor(state.received/state.total*100)):0;
  const text=state.status==='downloading'?t('update.downloading',{percent}):key?t(key,{version:state.version||state.current}):state.current?t('update.version',{version:state.current}):'';
  setSelectableText(status,text);
  const managedHint=$('update-managed');
  if(managedHint){managedHint.hidden=!managed;setSelectableText(managedHint,managed?t('update.managed',{manager:state.update_command.startsWith('winget ')?'WinGet':'Scoop',command:state.update_command}):'');}
  const installHelp=$('update-install-help');
  if(installHelp){installHelp.hidden=!available||managed||state.can_install||state.automatic===false;setSelectableText(installHelp,t('update.install_help'));}
  // Demo, read-only and independent sessions cannot schedule checks or change
  // the installed app's preference, so show why this setting is unavailable.
  const scheduled=state.automatic!==false;
  const automatic=$('automatic-updates'),last=$('update-last-checked');
  if(automatic){automatic.checked=scheduled&&window.presentationView?.auto_check_updates!==false;automatic.disabled=!scheduled||updateInstalling();}
  const unavailable=$('update-automatic-unavailable');
  if(unavailable)unavailable.hidden=scheduled;
  const checked=window.presentationView?.last_update_check;
  const valid=scheduled&&Number.isFinite(checked)&&Math.abs(checked)<=8640000000000;
  if(last){last.hidden=!valid;setSelectableText(last,valid?t('update.last_checked',{time:I.date(checked*1000,{year:'numeric',month:'short',day:'numeric',hour:'numeric',minute:'2-digit'})}):'');}
  button.textContent=t(state.status==='checking'?'update.checking':updateBusy()?'update.updating':available&&!managed?(state.status==='ready'&&state.can_install?'update.install':'update.download'):'update.check');
  button.disabled=dataOperationBusy||state.status==='checking'||updateBusy();
  button.setAttribute('aria-busy',String(state.status==='checking'||updateBusy()));
  details.hidden=!available&&!['error','limited'].includes(state.status);
}
window.renderUpdatePreferences=renderUpdateState;
window.receiveUpdateState=state=>{Object.assign(updateState,state);renderDataOperation();};
window.receiveUpdateAvailable=version=>window.toast(t(updateState.update_command?'update.found_managed':'update.found',{version,command:updateState.update_command}),8000);
// An ad-hoc signed update can leave macOS showing a stale, ineffective grant.
let updatedThisLaunch=false;
window.receiveUpdateNotice=notice=>{
  updatedThisLaunch=updatedThisLaunch||!!notice.success;
  window.toast(t(notice.success?'update.installed':'update.failed_last',{version:notice.version}));
  if(data)renderPermission();
};
$('check-updates')?.addEventListener('click',()=>{
  if(dataOperationBusy||updateBusy()||updateState.status==='checking')return;
  send(['available','ready'].includes(updateState.status)&&!updateState.update_command?(updateState.can_install?'install_update':'update_release'):'check_updates');
});
$('update-details')?.addEventListener('click',event=>{event.preventDefault();send('update_release');});
$('ignore-software-input')?.addEventListener('change',event=>send('ignore_software_input',{enabled:event.target.checked}));
$('automatic-updates')?.addEventListener('change',event=>{
  if(updateInstalling()||updateState.automatic===false)return;
  send('automatic_updates',{enabled:event.target.checked});
});

function renderDataOperation(){
  $('data-operation-status').hidden=!dataOperationBusy;
  $('data-operation-message').textContent=t('storage.processing');
  for(const id of ['storage-directory','import-history','reveal'])$(id).disabled=dataOperationBusy||updateInstalling()||(id==='reveal'&&!!data?.demo);
  $('takeover').disabled=dataOperationBusy||!data?.capture?.listening||!!data?.capture?.permission_needed;
  $('uninstall').disabled=dataOperationBusy||updateBusy()||updateState.status==='checking';
  renderUpdateState();
}
window.receiveDataBusy=busy=>{
  if(dataOperationBusy&&!busy)scatter.clear();
  dataOperationBusy=!!busy;
  renderDataOperation();
};
let calendarRange='365',historyRange='all',historyPage=0,calendarLayout='',historySignature='';
let calendarCells=new Map();
const H=ClickPulseHistory;
const scatter=ClickPulseScatter.create({document,window,I,H,send});
window.receiveHistoryHours=(id,rows)=>scatter.receive(id,rows);
window.clearHistoryScatter=()=>scatter.clear();
let calendarBook=H.nextBook(-1,Math.random()),calendarPhrase=H.nextPhrase(-1,Math.random());
const titles={overview:'page.overview',history:'page.history',keys:'page.heatmap',shortcuts:'page.shortcuts',mouse:'page.heatmap',share:'page.share',settings:'page.settings'};
const navTitles={...titles,share:'nav.share'};
const insights=typeof ClickPulseInsights==='undefined'?null:ClickPulseInsights.create({document,window,I,send,syncDropdown:Dropdowns.sync,toast:message=>window.toast(message)});
window.updateInsightsVisibility=()=>insights?.setWindowState(window.presentationView);
const sportView=typeof ClickPulseSport==='undefined'?null:ClickPulseSport.createView({document,window,translate:t,formatSpeed:value=>I.decimal(value,1),send});
window.receiveSport=sample=>sportView?.receive(recordingPaused===null?sample:{...sample,paused:recordingPaused});
window.receivePaused=paused=>{
  const changed=recordingPaused!==paused;recordingPaused=paused;
  if(miniData)miniData={...miniData,paused};
  if(data){data={...data,paused};renderRecordingState();}
  if(window.presentationView?.compact&&miniData)window.presentationStats?.(miniData);
  else if(data)window.presentationStats?.(data);
  else{
    $('pause').textContent=t(paused?'action.resume':'action.pause');
    window.presentationRecordingState?.({paused,capture:{listening:!paused}});
  }
  if(changed)sportView?.setPaused(paused);
};
window.updateSportActivity=()=>sportView?.sync(window.presentationView);
function updateHeading(){
  document.body.dataset.page=page;
  $('page-title').textContent=t(titles[page]);
  $('toolbar-title').dataset.i18n=titles[page];$('toolbar-title').textContent=t(titles[page]);
  renderSidebar();
}
function renderSidebar(){
  document.body.dataset.sidebarCollapsed=String(sidebarCollapsed);
  const button=$('sidebar-toggle'),label=t(sidebarCollapsed?'nav.expand':'nav.collapse');
  button.setAttribute('aria-expanded',String(!sidebarCollapsed));
  const mac=window.clickpulsePlatform==='macos';
  button.setAttribute('aria-label',label);button.title=`${label} (${mac?'⌘B':'Ctrl+B'})`;
  button.setAttribute('aria-keyshortcuts',mac?'Meta+B':'Control+B');
  $('sidebar-toggle-arrow').setAttribute('d',sidebarCollapsed?'m14 9 3 3-3 3':'m16 9-3 3 3 3');
  for(const link of document.querySelectorAll('nav button[data-page]')){
    const name=t(navTitles[link.dataset.page]);
    link.classList.toggle('selected',link.dataset.page===page);
    link.setAttribute('aria-current',link.dataset.page===page?'page':'false');
    link.setAttribute('aria-label',name);link.title=sidebarCollapsed?name:'';
  }
}
window.setInterfaceLanguage=(locale,preference)=>{
  $('language-select').value=preference||'system';
  Dropdowns.sync('language-select');
  if(I.locale===locale)return;
  I.setLocale(locale);I.apply(document);window.syncPresentationLabels?.();updateHeading();renderDataOperation();
  window.renderTodayPeriod?.();
  renderTimeZoneOptions();Dropdowns.syncAll();
  calendarLayout='';historySignature='';
  sportView?.render();
  for(const id of ['calendar-range-select','history-range-select'])delete $(id).dataset.options;
};
let timezoneNames=[],timezoneChoicesSignature='';
function renderTimeZoneOptions(){
  const select=$('timezone-select'),value=window.presentationView?.timezone||'system';
  const options=['system',...timezoneNames].map(value=>{
    const option=document.createElement('option');option.value=value;
    option.textContent=value==='system'?t('settings.timezone_system'):value.replace(/_/g,' ');return option;
  });
  select.replaceChildren(...options);select.value=value;Dropdowns.sync('timezone-select');
}
window.setTimeZoneChoices=names=>{
  const signature=JSON.stringify(names);if(signature===timezoneChoicesSignature)return;timezoneChoicesSignature=signature;
  timezoneNames=names.filter(name=>{try{new Intl.DateTimeFormat('en',{timeZone:name});return true;}catch{return false;}});
  renderTimeZoneOptions();
};
window.setInterfaceTimeZone=value=>{
  try{
    if(I.setTimeZone(value)){renderSignatures.clear();calendarLayout='';historySignature='';}
    $('timezone-select').value=value;Dropdowns.sync('timezone-select');return true;
  }catch{window.toast(t('settings.timezone_unavailable'));return false;}
};
$('timezone-select').addEventListener('change',event=>send('timezone',{timezone:event.target.value}));
$('today-period-select').addEventListener('change',event=>send('today_period',{period:event.target.value}));
const todayPeriod=()=>window.presentationView?.today_period||'calendar_day';
window.counterSummary=snapshot=>{
  if(!snapshot)return null;
  if(!Array.isArray(snapshot.windows)){
    if((snapshot.today_period||'calendar_day')!==todayPeriod())return null;
    return {counts:snapshot.today,approximate:!!snapshot.approximate};
  }
  return todayPeriod()==='rolling_24h'
    ?snapshot.windows.find(window=>window.seconds===86400)||null
    :{counts:snapshot.today,approximate:false};
};
window.renderTodayPeriod=()=>{
  const rolling=todayPeriod()==='rolling_24h',label=rolling?'metric.last24':'metric.today';
  for(const id of ['today-label','mini-period-label']){$(id).dataset.i18n=label;$(id).textContent=t(label);}
  const scope=rolling?'metric.last24.scope':'metric.today.scope';
  $('mini-period-label').title=t(scope);
  $('timezone-row').hidden=rolling;
  if(rolling)Dropdowns.controllers.get('timezone-select')?.close();
  $('today-period-select').value=todayPeriod();Dropdowns.sync('today-period-select');
};
$('language-select').addEventListener('change',event=>send('language',{language:event.target.value}));
$('theme-select').addEventListener('change',event=>send('theme',{theme:event.target.value}));
$('ui-scale-select').addEventListener('change',event=>send('ui_scale',{scale:event.target.value}));
window.renderDisplaySettings=()=>{
  $('ui-scale-select').value=window.presentationView?.ui_scale||'100';Dropdowns.sync('ui-scale-select');
};
const K=ClickPulseKeyboard;
const onWindows=window.clickpulsePlatform==='windows',onLinux=window.clickpulsePlatform==='linux';
const keyboardCells=new Map();
let keyboardLayoutSignature='';
let keyboardDistribution=null;
const usesKeyboardTreemap=()=>window.presentationView?.keyboard_view==='treemap';
const keyboardBlocks=new Map();
function updateKeyboardView(){
  const blocks=usesKeyboardTreemap();
  $('keyboard-layout').parentElement.hidden=blocks;
  $('keyboard-treemap').hidden=!blocks;
  $('keyboard-view-toggle').setAttribute('aria-pressed',String(blocks));
  $('keyboard-blocks-icon').hidden=blocks;$('keyboard-layout-icon').hidden=!blocks;
  const key=blocks?'keys.show_layout':'keys.show_treemap';
  $('keyboard-view-label').dataset.i18n=key;$('keyboard-view-label').textContent=t(key);
  $('keyboard-layout-mode').hidden=blocks;
  const full=!!keyboardDistribution?.fullSize;
  $('keyboard-compact').setAttribute('aria-pressed',String(!full));
  $('keyboard-full').setAttribute('aria-pressed',String(full));
}
$('keyboard-compact').addEventListener('click',()=>send('keyboard_layout',{layout:'compact'}));
$('keyboard-full').addEventListener('click',()=>send('keyboard_layout',{layout:'full_size'}));
$('keyboard-view-toggle').addEventListener('click',()=>{
  const view=usesKeyboardTreemap()?'layout':'treemap';
  window.receiveView({keyboard_view:view});
  send('keyboard_view',{view});
});
if(typeof ResizeObserver==='function')new ResizeObserver(()=>{
  if(page==='keys'&&usesKeyboardTreemap()&&!window.presentationView?.compact&&keyboardDistribution)renderKeyboardTreemap(keyboardDistribution);
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
const settingsLinks=[...document.querySelectorAll('[data-settings-section]')];
let settingsSection='personalization-settings';
function syncSettingsNavigation(){
  $('settings-subnav').hidden=page!=='settings'||sidebarCollapsed;
  $('settings-nav').setAttribute('aria-expanded',String(page==='settings'&&!sidebarCollapsed));
  for(const link of settingsLinks)link.hidden=$(link.dataset.settingsSection).hidden;
  if(!settingsLinks.some(link=>!link.hidden&&link.dataset.settingsSection===settingsSection))
    settingsSection=settingsLinks.find(link=>!link.hidden)?.dataset.settingsSection;
  for(const link of settingsLinks){
    const active=link.dataset.settingsSection===settingsSection;
    link.classList.toggle('current-section',active);
    link.setAttribute('aria-current',active?'location':'false');
  }
}
function toggleSidebar(){
  pendingSessionState=null;cancelSessionScrollRestore();
  Dropdowns.closeAll();if(typeof ClickPulseDatePicker!=='undefined')ClickPulseDatePicker.closeAll();sidebarCollapsed=!sidebarCollapsed;
  renderSidebar();syncSettingsNavigation();window.captureSessionState?.();
}
$('sidebar-toggle').addEventListener('click',toggleSidebar);
const openProjectHome=()=>window.clickpulsePreview
  ?window.open('https://github.com/endNone/ClickPulse','_blank','noopener')
  :send('project_home');
$('project-home').addEventListener('click',openProjectHome);
const starNotice=$('star-notice');
window.receiveStarReminder=()=>{starNotice.hidden=false;};
$('star-dismiss').addEventListener('click',()=>{starNotice.hidden=true;});
$('star-open').addEventListener('click',()=>{starNotice.hidden=true;openProjectHome();});
$('send-feedback').addEventListener('click',()=>window.clickpulsePreview
  ?window.open('https://github.com/endNone/ClickPulse/issues/new/choose','_blank','noopener')
  :send('send_feedback'));
updateHeading();
for(const link of settingsLinks)link.addEventListener('click',()=>{
  const target=$(link.dataset.settingsSection);
  if(page!=='settings'||target.hidden)return;
  cancelSessionScrollRestore();
  Dropdowns.closeAll();settingsSection=link.dataset.settingsSection;syncSettingsNavigation();
  target.focus({preventScroll:true});
  target.scrollIntoView({block:'start',behavior:window.matchMedia?.('(prefers-reduced-motion: reduce)').matches?'auto':'smooth'});
  window.captureSessionState?.();
});
function navigatePage(next){
  pendingSessionState=null;cancelSessionScrollRestore();
  Dropdowns.closeAll();if(typeof ClickPulseDatePicker!=='undefined')ClickPulseDatePicker.closeAll();
  if(next==='overview'&&page!=='overview'){
    calendarBook=H.nextBook(calendarBook,Math.random());
    calendarPhrase=H.nextPhrase(calendarPhrase,Math.random());
  }
  if(page==='history'&&next!=='history')scatter.clear();
  page=next==='mouse'?'keys':next;
  document.querySelectorAll('.page').forEach(el=>el.hidden=el.id!==page);
  updateHeading();syncSettingsNavigation();
  document.querySelector('main').scrollTop=0;
  window.updateSportActivity();
  if(data)window.receive(data);
  window.captureSessionState?.();
}
document.querySelectorAll('nav button[data-page]').forEach(button=>button.addEventListener('click',()=>navigatePage(button.dataset.page)));
document.querySelectorAll('[data-range]').forEach(button=>button.addEventListener('click',()=>{
  range=Number(button.dataset.range);document.querySelectorAll('[data-range]').forEach(b=>{b.classList.toggle('selected',b===button);b.setAttribute('aria-pressed',String(b===button));});if(data)renderTrend($('trend-chart'),data.days.slice(-range));window.captureSessionState?.();
}));
$('pause').addEventListener('click',()=>send('pause'));
for(const id of ['permission','settings-permission'])$(id).addEventListener('click',()=>send('permission'));
$('storage-directory').addEventListener('click',()=>send('storage_directory'));
$('import-history').addEventListener('click',()=>send('import_history'));
$('reveal').addEventListener('click',()=>send('reveal'));
$('calendar-range-select').addEventListener('change',event=>{
  calendarRange=event.target.value;
  Dropdowns.sync(event.target);
  if(data)renderCalendar();
  window.captureSessionState?.();
});
$('history-range-select').addEventListener('change',event=>{historyRange=event.target.value;historyPage=0;if(data)renderHistory();window.captureSessionState?.();});
$('history-prev').addEventListener('click',()=>{historyPage--;if(data)renderHistory();window.captureSessionState?.();});
$('history-next').addEventListener('click',()=>{historyPage++;if(data)renderHistory();window.captureSessionState?.();});
window.toast=(message,duration=4500)=>{$('toast').textContent=message;$('toast').hidden=false;clearTimeout(toastTimer);toastTimer=setTimeout(()=>$('toast').hidden=true,duration)};
window.failure=message=>{
  sportView?.receive({available:false});
  $('error-banner').hidden=false;$('error-banner').textContent=message;
  const failed=snapshot=>({...snapshot,capture:{...snapshot.capture,listening:false,error:message,error_code:'error.stats'}});
  if(data)data=failed(data);
  if(miniData)miniData=failed(miniData);
  const current=window.presentationView?.compact?(miniData||data):data;
  if(current){
    window.presentationStats?.(current);
  }else{
    $('mini-status').textContent=t('status.unavailable');$('mini-status').classList.toggle('error',true);
  }
};
window.receiveMini=snapshot=>{
  if(!window.presentationView?.compact||snapshot.timezone!==(window.presentationView?.timezone||'system')||!window.counterSummary(snapshot))return;
  miniData=recordingPaused===null?snapshot:{...snapshot,paused:recordingPaused};
  window.presentationStats?.(miniData);
};
window.renderMiniSnapshot=()=>{
  const snapshot=[miniData,data].find(value=>value&&(value.timezone||'system')===(window.presentationView?.timezone||'system')&&window.counterSummary(value));
  if(snapshot)window.presentationStats?.(snapshot);
  else for(const id of ['mini-today','mini-keyboard','mini-mouse'])setSelectableText($(id),'—');
};
// Navigation survives renderer disposal, without saving history or input text.
let pendingSessionState=null,pendingSessionScroll=null,lastSessionState='',scrollStateTimer;
window.captureSessionState=()=>{
  if(pendingSessionState||pendingSessionScroll||window.presentationView?.compact||window.presentationView?.visible===false)return;
  const state={page,settings_section:settingsSection,trend_days:range,calendar_range:calendarRange,history_range:historyRange,history_page:historyPage,sidebar_collapsed:sidebarCollapsed,scroll_top:Math.max(0,Math.round(document.querySelector('main').scrollTop||0))};
  if(insights)state.insights=insights.sessionState();
  const encoded=JSON.stringify(state);
  if(encoded!==lastSessionState){lastSessionState=encoded;send('session_state',{state});}
};
window.restoreSessionState=state=>{cancelSessionScrollRestore();pendingSessionState=state;};
function cancelSessionScrollRestore(){
  if(pendingSessionScroll)document.querySelector('main').style.setProperty('overflow-anchor','auto');
  pendingSessionScroll=null;
}
function applySessionScroll(){
  const pending=pendingSessionScroll;
  if(!pending?.ready||window.presentationView?.compact||window.presentationView?.visible===false)return;
  document.querySelector('main').scrollTop=pending.scrollTop;
  cancelSessionScrollRestore();window.captureSessionState();
}
function restoreSessionScroll(state){
  cancelSessionScrollRestore();
  const main=document.querySelector('main'),pending={scrollTop:state.scroll_top,ready:false};
  pendingSessionScroll=pending;
  // A fallback font can make the page shorter and clamp a near-bottom position.
  // Suppress anchoring and transient state saves until the real fonts are laid out.
  main.style.setProperty('overflow-anchor','none');main.scrollTop=pending.scrollTop;
  const finish=()=>{
    if(pendingSessionScroll!==pending)return;
    pending.ready=true;applySessionScroll();
  };
  const afterLayout=()=>window.requestAnimationFrame
    ?window.requestAnimationFrame(()=>window.requestAnimationFrame(finish)):finish();
  if(document.fonts)document.fonts.ready.then(afterLayout,afterLayout);
  else finish();
}
// User interaction always takes priority over a delayed cold-view restoration.
for(const event of ['pointerdown','wheel','touchstart','keydown'])
  document.addEventListener?.(event,()=>{pendingSessionState=null;cancelSessionScrollRestore();},{capture:true,passive:true});
document.addEventListener('keydown',event=>{
  const mac=window.clickpulsePlatform==='macos';
  if(event.defaultPrevented||event.isComposing||event.altKey||event.shiftKey
    ||!(mac?event.metaKey&&!event.ctrlKey:event.ctrlKey&&!event.metaKey)
    ||!(event.code==='KeyB'||event.key?.toLowerCase()==='b')
    ||window.presentationView?.compact||window.presentationView?.visible===false
    ||$('close-overlay').hidden===false)return;
  const target=event.target;
  if(target instanceof Element&&(target.isContentEditable||target.closest('input,textarea,select,[role="textbox"]')))return;
  event.preventDefault();
  if(!event.repeat)toggleSidebar();
});
function applySessionState(state){
  insights?.restoreSession(state.insights);
  page=state.page==='mouse'?'keys':state.page;range=state.trend_days;settingsSection=state.settings_section;
  sidebarCollapsed=state.sidebar_collapsed===true;
  historyPage=state.history_page;
  document.querySelectorAll('.page').forEach(el=>el.hidden=el.id!==page);
  document.querySelectorAll('[data-range]').forEach(b=>{const selected=Number(b.dataset.range)===range;b.classList.toggle('selected',selected);b.setAttribute('aria-pressed',String(selected));});
  for(const [id,value,recent] of [['calendar-range-select',state.calendar_range,true],['history-range-select',state.history_range,false]]){
    const select=$(id);updateRangeOptions(select,recent);
    select.value=Array.from(select.options).some(option=>option.value===value)?value:recent?'365':'all';Dropdowns.sync(select);
  }
  calendarRange=$('calendar-range-select').value;historyRange=$('history-range-select').value;
  updateHeading();syncSettingsNavigation();updateKeyboardView();
}
document.querySelector('main').addEventListener('scroll',()=>{
  clearTimeout(scrollStateTimer);scrollStateTimer=setTimeout(()=>window.captureSessionState(),100);
},{passive:true});
window.receiveCapture=capture=>{
  if(data){data.capture=capture;renderRecordingState();renderPermission();if(page==='settings')renderSettings();}
  if(miniData){miniData.capture=capture;window.presentationStats?.(miniData);}
  $('error-banner').hidden=!capture.error;
  if(capture.error)$('error-banner').textContent=t(capture.error_code||'error.recording');
};
window.receive=snapshot=>{
  if(snapshot.timezone && snapshot.timezone!==(window.presentationView?.timezone||'system'))return;
  if(!window.setInterfaceTimeZone(snapshot.timezone||window.presentationView?.timezone||'system'))return;
  updateHistoryRevision(snapshot.history_days);
  data=recordingPaused===null?snapshot:{...snapshot,paused:recordingPaused};
  window.presentationStats?.(data);
  if(window.presentationView?.compact){miniData={...data,timezone:data.timezone||'system'};return;}
  const restoring=pendingSessionState;
  if(restoring){pendingSessionState=null;applySessionState(restoring);}
  const demo=!!data.demo;$('demo-badge').hidden=!demo;
  renderRecordingState();
  renderPermission();
  $('error-banner').hidden=!data.capture.error;
  if(data.capture.error)$('error-banner').textContent=t(data.capture.error_code||'error.recording');
  if(page==='settings')renderSettings();
  if(page==='overview'){
    const summary=window.counterSummary(data);
    for(const key of ['today','week','lifetime']){
      const counts=key==='today'?summary?.counts:data[key],prefix=key==='today'&&summary?.approximate?'≈ ':'';
      setSelectableText($(key),counts?prefix+n(total(counts)):'—');
      const keyboard=counts?prefix+n(counts.keyboard):'—',mouse=counts?prefix+n(counts.mouse):'—';
      setSelectableText($(key+'-detail'),`Key ${keyboard}\nMouse ${mouse}`);
    }
    renderChanged('windows',data.windows,()=>renderWindows(data.windows));
    renderChanged('hourly',data.hourly||[],()=>renderHourly(data.hourly||[]));
    renderChanged('trend',[range,data.days.slice(-range)],()=>renderTrend($('trend-chart'),data.days.slice(-range)));
    renderChanged('donut',data.lifetime,()=>renderDonut(data.lifetime));
    updateRangeOptions($('calendar-range-select'),true);calendarRange=$('calendar-range-select').value;
    renderChanged('calendar',[data.now.split('T')[0],calendarRange,calendarBook,calendarPhrase,historyRevision],renderCalendar);
  }else if(page==='history'){
    updateRangeOptions($('history-range-select'),false);historyRange=$('history-range-select').value;
    renderHistory();
  }else if(page==='keys'){
    renderChanged('keyboard',[data.keyboard_stats||data.top_keys,window.presentationView?.keyboard_layout,usesKeyboardTreemap()],renderKeyboard);
  }
  insights?.render(page,data);
  if(restoring)restoreSessionScroll(restoring);
  else applySessionScroll();
  window.captureSessionState();
};
function renderPermission(){
  if(onLinux)$('permission-reset').hidden=true;
  $('permission-banner').hidden=!data.capture.permission_needed;
  const needsMacUpdatePermission=updatedThisLaunch&&!onWindows&&!onLinux;
  $('permission-title').hidden=needsMacUpdatePermission;$('permission-updated-title').hidden=!needsMacUpdatePermission;
}
function renderRecordingState(){
  $('pause').textContent=t(data.source_mode?(data.paused?'action.resume_refresh':'action.pause_refresh'):(data.paused?'action.resume':'action.pause'));
}
function renderSettings(){
  renderChanged('settings',[data.capture,data.source_mode,data.setup_pending,data.auto_start_available,data.auto_start,data.demo,data.database_path,data.database_mode,dataOperationBusy,window.presentationView?.ignore_software_input],renderSettingsBody);
}
function renderSettingsBody(){
  $('permission-description').textContent=t(onLinux?(data.capture.listening&&!data.capture.permission_needed?'settings.authorized':'settings.permission_description'):onWindows?(data.capture.listening?'settings.windows_capture':'status.unavailable'):data.source_mode && !data.setup_pending?'settings.source_permission':data.capture.listening?'settings.authorized':'settings.permission_description');
  $('permission-row').hidden=onWindows || (!!data.source_mode && !data.setup_pending);
  $('permission-reset-settings').hidden=onLinux || $('permission-row').hidden || data.capture.listening;
  $('capture-settings').hidden=!!data.source_mode && !data.setup_pending;
  $('ignore-software-row').hidden=onLinux || !!data.source_mode;
  $('ignore-software-input').checked=!!window.presentationView?.ignore_software_input;
  $('handoff-settings').hidden=!data.setup_pending;
  $('takeover').disabled=dataOperationBusy || !data.capture.listening || data.capture.permission_needed;
  $('handoff-description').textContent=t(data.capture.listening?'settings.takeover_ready':'settings.takeover_wait');
  $('autostart-row').hidden=data.auto_start_available!==true;
  $('autostart').checked=!!data.auto_start;
  setSelectableText($('db-path'),data.demo?t('demo.storage'):data.database_path);
  const storageMode=data.demo?'demo':data.database_mode||'custom';
  const storageDescriptions={custom:'settings.storage_custom',source:'settings.storage_source',demo:'settings.storage_demo'};
  $('uninstall-row').hidden=window.clickpulsePlatform!=='windows'||data.demo||data.source_mode||storageMode!=='default';
  $('db-description').hidden=storageMode==='default';
  $('db-description').textContent=storageMode==='default'?'':t(storageDescriptions[storageMode]||storageDescriptions.custom);
  $('storage-directory').hidden=storageMode!=='default';
  $('import-history-row').hidden=storageMode!=='default';
  $('reveal').disabled=!!data.demo||dataOperationBusy||updateInstalling();
  syncSettingsNavigation();
}
const svgNode=(tag,attrs={})=>{const node=document.createElementNS('http://www.w3.org/2000/svg',tag);for(const[key,value]of Object.entries(attrs))node.setAttribute(key,String(value));return node};
const svgText=(text,x,y,attrs={})=>{const el=svgNode('text',{x,y,...attrs});el.textContent=text;return el};
const axisObservers=new WeakMap();
function observeTrendLabels(container,axis){
  if(typeof ResizeObserver!=='function')return;
  axisObservers.get(container)?.disconnect();
  const labels=Array.from(axis.children);
  const observer=new ResizeObserver(()=>{
    const width=axis.getBoundingClientRect().width;
    if(!width)return;
    const ticks=labels.map(label=>({
      center:parseFloat(label.style.left)*width/100,
      size:label.getBoundingClientRect().width
    }));
    const offsets=H.trendLabelOffsets(width,ticks);
    labels.forEach((label,i)=>{
      const offset=offsets[i];
      label.style.visibility=offset===null?'hidden':'';
      if(offset!==null)label.style.transform='translateX('+offset+'px)';
    });
  });
  axisObservers.set(container,observer);
  observer.observe(axis);
  // Observe the labels too: a new font or text size can change them at a fixed window width.
  for(const label of labels)observer.observe(label);
}
let hourlyNodes=null;
function renderHourly(hours){
  const container=$('hourly-chart'),keys=hours.map((h,i)=>i?h.start_ms:h.end_ms-3600000).join(','),
    counts=hours.reduce((sum,h)=>({keyboard:sum.keyboard+h.counts.keyboard,mouse:sum.mouse+h.counts.mouse}),{keyboard:0,mouse:0}),
    approximate=hours.some(h=>h.approximate),{ceiling,ticks}=H.countScale(Math.max(0,...hours.map(h=>total(h.counts))));
  const formatRange=h=>I.date(h.start_ms,{month:'short',day:'numeric',hour:'2-digit',minute:'2-digit',hourCycle:'h23',timeZoneName:'short'})+' — '+I.date(h.end_ms,{month:'short',day:'numeric',hour:'2-digit',minute:'2-digit',hourCycle:'h23',timeZoneName:'short'});
  const description=h=>t('chart.tooltip',{period:formatRange(h),keyboard:(h.approximate?'≈ ':'')+n(h.counts.keyboard),mouse:(h.approximate?'≈ ':'')+n(h.counts.mouse)});
  setSelectableText($('hourly-total'),(approximate?'≈ ':'')+n(total(counts)));
  $('hourly-total').setAttribute('aria-label',t('chart.hourly_total',{count:(approximate?'≈ ':'')+n(total(counts))}));
  setSelectableText($('hourly-range'),hours.length?formatRange({start_ms:hours[0].start_ms,end_ms:hours[hours.length-1].end_ms}):'');
  if(!hourlyNodes||hourlyNodes.keys!==keys){
    const focused=document.activeElement?.dataset.hour,active=focused||hourlyNodes?.active;
    const plot=document.createElement('div'),yAxis=document.createElement('div'),xAxis=document.createElement('div'),empty=document.createElement('div');
    plot.className='hourly-plot';yAxis.className='trend-y-axis';xAxis.className='trend-x-axis';empty.className='empty-label';
    const slots=hours.map((h,i)=>{
      const node=document.createElement('div'),keyboard=document.createElement('span'),mouse=document.createElement('span');
      node.className='hourly-slot';node.tabIndex=0;node.dataset.hour=String(i?h.start_ms:h.end_ms-3600000);node.setAttribute('role','img');
      keyboard.className='hourly-keyboard';mouse.className='hourly-mouse';node.append(keyboard,mouse);
      for(const event of ['mouseenter','focus'])node.addEventListener(event,()=>{if(hourlyNodes?.plot!==plot)return;hourlyNodes.active=node.dataset.hour;updateHourlyDetail();});
      node.addEventListener('keydown',event=>{
        if(hourlyNodes?.plot!==plot)return;
        const direction=event.key==='ArrowRight'?1:event.key==='ArrowLeft'?-1:0;
        const next=event.key==='Home'?0:event.key==='End'?hours.length-1:Math.max(0,Math.min(hours.length-1,i+direction));
        if(direction||event.key==='Home'||event.key==='End'){event.preventDefault();hourlyNodes.slots[next].node.focus();}
      });
      plot.append(node);return {node,keyboard,mouse};
    });
    plot.addEventListener('focusout',event=>{if(hourlyNodes?.plot!==plot)return;if(!plot.contains(event.relatedTarget)){hourlyNodes.active=null;updateHourlyDetail();}});
    plot.addEventListener('mouseleave',()=>{if(hourlyNodes?.plot!==plot)return;if(!slots.some(slot=>slot.node===document.activeElement)){hourlyNodes.active=null;updateHourlyDetail();}});
    hourlyNodes={keys,plot,yAxis,xAxis,empty,slots,active};
    container.replaceChildren(plot,yAxis,xAxis,empty);
    slots.find(slot=>slot.node.dataset.hour===focused)?.node.focus();
  }
  const chart=hourlyNodes;
  chart.hours=hours;chart.description=description;
  const scaleKey=I.locale+':'+ceiling;
  if(chart.scaleKey!==scaleKey){chart.scaleKey=scaleKey;
  container.style.setProperty('--trend-gutter',Math.max(5.6,...ticks.map(value=>n(value).length*0.6+1.2))+'rem');
  chart.plot.style.gridTemplateColumns=`repeat(${Math.max(1,hours.length)},minmax(0,1fr))`;
  chart.yAxis.replaceChildren(...ticks.map(value=>{
    const label=document.createElement('span');label.textContent=n(value);label.style.top=(1-value/ceiling)*100+'%';return label;
  }));
  chart.plot.style.setProperty('--hourly-grid-step',100/(ticks.length-1)+'%');
  }
  chart.slots.forEach(({node,keyboard,mouse},i)=>{
    const h=hours[i],stamp=[I.locale,I.effectiveTimeZone,h.start_ms,h.end_ms,h.approximate,h.counts.keyboard,h.counts.mouse,ceiling].join(':');
    if(node.dataset.rendered===stamp)return;node.dataset.rendered=stamp;node.setAttribute('aria-label',description(h));
    keyboard.style.height=h.counts.keyboard/ceiling*100+'%';mouse.style.height=h.counts.mouse/ceiling*100+'%';
    node.classList.toggle('partial',h.end_ms-h.start_ms<3600000);
  });
  const labelsKey=[keys,I.locale,I.effectiveTimeZone].join(':');
  if(chart.labelsKey!==labelsKey){chart.labelsKey=labelsKey;
  chart.xAxis.replaceChildren(...H.trendLabelIndices(hours.length).map(i=>{
    const h=hours[i],label=document.createElement('span'),time=document.createElement('span'),date=document.createElement('small');
    label.className='trend-date';label.style.left=(i+.5)/hours.length*100+'%';
    time.textContent=I.date(h.start_ms,{hour:'2-digit',minute:'2-digit',hourCycle:'h23'});date.textContent=I.date(h.start_ms,{month:'short',day:'numeric'});label.append(time,date);return label;
  }));
  observeTrendLabels(container,chart.xAxis);
  }
  chart.empty.hidden=hours.some(h=>total(h.counts)>0);chart.empty.textContent=t('common.empty');
  updateHourlyDetail();
}
function updateHourlyDetail(){
  if(!hourlyNodes?.hours)return;
  const chart=hourlyNodes,index=chart.slots.findIndex(slot=>slot.node.dataset.hour===chart.active);
  chart.slots.forEach((slot,i)=>{slot.node.classList.toggle('active',i===index);slot.node.tabIndex=i===(index>=0?index:chart.slots.length-1)?0:-1;});
  setSelectableText($('hourly-detail'),index>=0?chart.description(chart.hours[index]):'');
}
function renderWindows(windows){
  const container=$('windows'),layout=windows.map(w=>w.seconds).join(',');
  if(container.dataset.layout!==layout){
    container.replaceChildren(...windows.map(()=>{
      const node=document.createElement('div');node.className='window';node.append(document.createElement('strong'),document.createElement('span'));return node;
    }));
    container.dataset.layout=layout;
  }
  windows.forEach((w,index)=>{
    setSelectableText(container.children[index].children[0],(w.approximate?'≈ ':'')+n(total(w.counts)));
    setSelectableText(container.children[index].children[1],t('window.'+w.seconds));
  });
}
const trendNodes=new WeakMap();
function renderTrend(container,days,mouseColor){
  const mouseFill=mouseColor||T.get(currentTheme).mouse;
  const layout=JSON.stringify([I.locale,currentTheme,mouseFill,days.map(d=>[d.day,d.start,d.end,d.grouping])]);
  let chart=trendNodes.get(container);
  if(!chart||chart.layout!==layout){
    const svg=svgNode('svg',{viewBox:'0 0 600 132',preserveAspectRatio:'none',class:'trend-plot',role:'img','aria-label':t('chart.aria')});
    const yAxis=document.createElement('div'),xAxis=document.createElement('div'),empty=document.createElement('div');
    yAxis.className='trend-y-axis';xAxis.className='trend-x-axis';empty.className='empty-label';empty.textContent=t('common.empty');
    const grid=svgNode('g');svg.append(grid);
    const width=600/Math.max(1,days.length),barWidth=Math.min(26,width*.6),palette=T.get(currentTheme);
    const bars=days.map((d,i)=>[palette.accent,mouseFill].map(color=>{
      const node=svgNode('rect',{x:i*width+(width-barWidth)/2,width:barWidth,rx:1.4,fill:color}),title=svgNode('title');node.append(title);svg.append(node);return {node,title};
    }));
    const multipleYears=new Set(days.map(d=>(d.start||d.day).slice(0,4))).size>1;
    for(const i of H.trendLabelIndices(days.length)){
      const d=days[i],date=d.start||d.day,monthly=d.grouping==='month',label=document.createElement('span');
      label.className='trend-date';
      label.style.left=(i+.5)/days.length*100+'%';
      const primary=document.createElement('span');primary.textContent=I.day(date,monthly?{month:'short'}:{month:'short',day:'numeric'});label.append(primary);
      if(monthly||multipleYears){const year=document.createElement('small');year.textContent=I.day(date,{year:'numeric'});label.append(year)}
      xAxis.append(label);
    }
    container.replaceChildren(svg,yAxis,xAxis,empty);
    observeTrendLabels(container,xAxis);
    chart={layout,grid,yAxis,bars,empty,counts:[]};trendNodes.set(container,chart);
  }
  const {ceiling,ticks}=H.countScale(Math.max(0,...days.map(d=>total(d.counts))));
  const scaleChanged=chart.ceiling!==ceiling;
  if(scaleChanged){
    container.style.setProperty('--trend-gutter',Math.max(5.6,...ticks.map(value=>n(value).length*0.6+1.2))+'rem');
    chart.grid.replaceChildren(...ticks.map(value=>svgNode('line',{x1:0,x2:600,y1:(1-value/ceiling)*132,y2:(1-value/ceiling)*132,stroke:'var(--line)','stroke-width':1,'vector-effect':'non-scaling-stroke'})));
    chart.yAxis.replaceChildren(...ticks.map(value=>{const label=document.createElement('span');label.textContent=n(value);label.style.top=(1-value/ceiling)*100+'%';return label;}));
    chart.ceiling=ceiling;
  }
  days.forEach((d,i)=>{
    const previous=chart.counts[i],changed=!previous||previous.keyboard!==d.counts.keyboard||previous.mouse!==d.counts.mouse;
    if(!changed&&!scaleChanged)return;
    const kh=d.counts.keyboard/ceiling*132,mh=d.counts.mouse/ceiling*132;
    chart.bars[i].forEach((bar,index)=>{bar.node.setAttribute('y',index?132-kh-mh:132-kh);bar.node.setAttribute('height',index?mh:kh);});
    if(changed){
      const period=d.start?I.day(d.start)+' — '+I.day(d.end):I.day(d.day);
      const description=t('chart.tooltip',{period,keyboard:n(d.counts.keyboard),mouse:n(d.counts.mouse)});
      chart.bars[i].forEach(bar=>bar.title.textContent=description);chart.counts[i]={...d.counts};
    }
  });
  chart.empty.hidden=days.some(d=>total(d.counts)>0);
}
let donutNodes;
function renderDonut(counts){
  const all=total(counts),circ=2*Math.PI*51,palette=T.get(currentTheme);
  const entries=[
    [t('common.keyboard'),counts.keyboard,palette.accent],
    [t('common.mouse'),counts.mouse,palette.mouse]
  ];
  const describe=([label,value])=>`${label}: ${n(value)} (${I.percent(all?value/all:0,1)})`;
  const layout=JSON.stringify([I.locale,currentTheme,entries.map(([label,value,color])=>[label,value>0&&all>0,color])]);
  if(!donutNodes||donutNodes.layout!==layout){
    const svg=svgNode('svg',{viewBox:'0 0 170 170',role:'img'});
    svg.append(svgNode('circle',{cx:85,cy:85,r:51,fill:'none',stroke:palette.empty,'stroke-width':13}));
    const arcs=entries.map(([,value,color])=>{
      if(!value||!all)return null;
      const arc=svgNode('circle',{cx:85,cy:85,r:51,fill:'none',stroke:color,'stroke-width':13,transform:'rotate(-90 85 85)'}),title=svgNode('title');
      arc.append(title);svg.append(arc);return {arc,title};
    });
    const percent=svgText('',85,85,{'text-anchor':'middle','dominant-baseline':'central','aria-hidden':'true',class:'composition-percent'});
    svg.append(percent);
    $('donut').replaceChildren(svg);
    const legend=$('composition-legend');
    const labels=entries.map(([label,,color])=>{
      const node=document.createElement('div'),count=document.createElement('strong'),name=document.createElement('span'),dot=document.createElement('i');
      dot.className='composition-dot';dot.style.backgroundColor=color;name.append(dot,label);node.append(count,name);return {node,count};
    });
    legend.replaceChildren(...labels.map(label=>label.node));
    donutNodes={layout,svg,arcs,labels,percent};
  }
  donutNodes.svg.setAttribute('aria-label',entries.map(describe).join(', '));
  setSelectableText(donutNodes.percent,all?I.percent(counts.keyboard/all,0):'—');
  let offset=0;
  entries.forEach((entry,index)=>{
    const value=entry[1],description=describe(entry),length=all?value/all*circ:0,arc=donutNodes.arcs[index];
    if(arc){arc.arc.setAttribute('stroke-dasharray',`${length} ${circ}`);arc.arc.setAttribute('stroke-dashoffset',-offset);arc.title.textContent=description;}
    offset+=length;
    donutNodes.labels[index].node.title=description;setSelectableText(donutNodes.labels[index].count,n(value));
  });
}
function updateRangeOptions(select,isCalendar){
  const years=H.years(data.history_days,isCalendar?data.now.slice(0,4):undefined);
  const options=isCalendar
    ? [['365',t('common.past_year')],...years.map(year=>[year,year])]
    : [['all',t('common.all_history')],...years.map(year=>[year,t('common.year',{year})])];
  const signature=I.locale+':'+options.map(option=>option[0]).join(',');
  if(select.dataset.options===signature)return;
  const selected=select.value;
  select.replaceChildren(...options.map(([value,label])=>{const option=document.createElement('option');option.value=value;option.textContent=label;option.hidden=isCalendar&&value==='365';return option}));
  select.value=options.some(option=>option[0]===selected)?selected:options[0][0];select.dataset.options=signature;
  Dropdowns.sync(select);
}
function renderCalendar(){
  const today=data.now.split('T')[0],bounds=H.bounds(data.history_days,today,calendarRange);
  const days=H.calendarDays(data.history_days,bounds.start,bounds.end);
  const layout=I.locale+':'+calendarRange+':'+bounds.start+':'+bounds.end;
  if(calendarLayout!==layout){
    calendarCells=new Map();
    const block=document.createElement('div');block.className='calendar-year';
    const months=document.createElement('div');months.className='calendar-months';
    const body=document.createElement('div');body.className='calendar-body';
    const weekdays=document.createElement('div');weekdays.className='calendar-weekdays';
    for(const text of I.weekdays().map((day,index)=>index%2===0?day:'')){const el=document.createElement('span');el.textContent=text;weekdays.append(el)}
    const grid=document.createElement('div');grid.className='heatmap';
    const padding=H.weekday(bounds.start);
    for(let i=0;i<padding;i++){const cell=document.createElement('div');cell.className='heat-cell blank';grid.append(cell)}
    for(const marker of H.monthMarkers(days,month=>I.month(month))){
      const el=document.createElement('span');el.textContent=marker.label;el.style.left=marker.left+'px';months.append(el);
    }
    for(const d of days){
      const cell=document.createElement('div');cell.dataset.day=d.day;grid.append(cell);calendarCells.set(d.day,cell);
    }
    body.append(weekdays,grid);block.append(months,body);
    $('heatmap').replaceChildren(block);
    calendarLayout=layout;
    if(calendarRange==='365')$('heatmap').scrollLeft=$('heatmap').scrollWidth;
  }
  const max=Math.max(1,...days.map(d=>total(d.counts)));
  for(const d of days){
    const cell=calendarCells.get(d.day),value=total(d.counts),level=value?Math.max(1,Math.ceil(value/max*4)):0;
    const className='heat-cell level-'+level+(d.day===today?' today':'');
    if(cell.className!==className)cell.className=className;
    const previous=cell.calendarCounts;
    if(!previous||previous.keyboard!==d.counts.keyboard||previous.mouse!==d.counts.mouse){
      cell.title=t('calendar.tooltip',{day:d.day,count:n(value),keyboard:n(d.counts.keyboard),mouse:n(d.counts.mouse)});cell.setAttribute('aria-label',cell.title);
      cell.calendarCounts={...d.counts};
    }
  }
  $('active-days').textContent=t('calendar.recorded',{count:n(days.filter(d=>total(d.counts)>0).length)});
  const comparison=H.bookComparison(days.reduce((sum,d)=>sum+d.counts.keyboard,0),calendarBook),aside=$('calendar-aside');
  aside.hidden=!comparison;
  if(comparison){
    const copy=I.locale.startsWith('zh')?I:literaryEnglish;
    const reference=H.books[calendarBook],book=copy.t('calendar.book_'+reference.id);
    const phrase='calendar.literary_'+calendarPhrase+(comparison.copies===1?'_one':'');
    aside.setAttribute('lang',copy.locale);
    aside.textContent=comparison.opening?copy.t('calendar.literary_opening'):copy.t(phrase,{count:copy.number(comparison.copies),book});
    aside.title=copy.t('calendar.literary_basis',{book,characters:copy.number(comparison.characters)});
  }else{aside.textContent='';aside.title='';}
  $('calendar-range').textContent=t('calendar.period',{start:I.day(bounds.start),end:I.day(bounds.end),count:n(days.reduce((sum,d)=>sum+total(d.counts),0))});
}
function renderHistory(){
  const today=data.now.split('T')[0],records=H.recordedInRange(data.history_days,today,historyRange),bounds=H.bounds(data.history_days,today,historyRange);
  if(!dataOperationBusy&&window.presentationView?.visible!==false)scatter.update(data,records,bounds,currentTheme);
  const pagination=H.paginate(records,historyPage);historyPage=pagination.page;
  const signature=I.locale+':'+currentTheme+':'+historyRange+':'+historyPage+':'+today+':'+historyRevision;
  if(signature===historySignature)return;
  historySignature=signature;
  const days=H.calendarDays(records,bounds.start,bounds.end),trend=H.trend(days);
  $('history-range').textContent=t('history.range',{start:I.day(bounds.start),end:I.day(bounds.end)});
  const keyboard=records.reduce((sum,d)=>sum+d.counts.keyboard,0),mouse=records.reduce((sum,d)=>sum+d.counts.mouse,0);
  $('history-summary').textContent=t('history.summary',{days:n(pagination.total),count:n(keyboard+mouse),keyboard:n(keyboard),mouse:n(mouse)});
  renderTrend($('history-chart'),trend,'#e5a00d');
  const table=$('history-table');
  if(table.children.length!==pagination.rows.length)table.replaceChildren(...pagination.rows.map(()=>{
    const tr=document.createElement('tr');for(let i=0;i<4;i++)tr.append(document.createElement('td'));return tr;
  }));
  pagination.rows.forEach((d,index)=>[I.day(d.day),n(d.counts.keyboard),n(d.counts.mouse),n(total(d.counts))].forEach((value,column)=>setSelectableText(table.children[index].children[column],value)));
  $('history-prev').disabled=historyPage===0;$('history-next').disabled=historyPage+1>=pagination.pages;
  $('history-page-info').textContent=pagination.total?t('history.pagination',{start:n(historyPage*30+1),end:n(Math.min((historyPage+1)*30,pagination.total)),count:n(pagination.total),page:n(historyPage+1),pages:n(pagination.pages)}):t('common.empty_history');
}
const arrowIds=['up','left','down','right'];
function renderKeyboard(){
  const stats=data.keyboard_stats||{physical:data.top_keys};
  const view=K.distribution(stats,'history',window.presentationView?.keyboard_layout);
  keyboardDistribution=view;updateKeyboardView();
  if(usesKeyboardTreemap()){
    renderKeyboardTreemap(view);return;
  }
  // Keep the hardware layout fixed; additional recorded keys remain in Treemap.
  const rows=view.rows.filter(row=>!row[0]?.supplemental);
  const max=Math.max(1,...rows.flat().map(k=>k.count));
  const signature=JSON.stringify([view.fullSize,rows.map(row=>row.map(key=>key.id))]);
  if(keyboardLayoutSignature!==signature){
    keyboardCells.clear();keyboardLayoutSignature=signature;
    const layout=$('keyboard-layout');layout.classList.toggle('full-size',view.fullSize);
    const main=document.createElement('div');main.className='keyboard-main';
    const clusters=[];
    for(const [index,row] of rows.entries()){
      const arrows=row.filter(key=>arrowIds.includes(key.id));
      const node=document.createElement('div');node.className='keyboard-row'+(index===0?' function-row':row.some(key=>key.supplemental)?' supplemental-row':arrows.length?' modifier-row':'');
      function createCell(key){
        const cell=document.createElement('div');cell.className='keyboard-key';cell.style.setProperty('--key-width',key.width);cell.dataset.key=key.id;cell.tabIndex=0;cell.setAttribute('role','img');
        const label=document.createElement('span');label.className='key-label';label.textContent=key.label;
        const count=document.createElement('span');count.className='key-count';cell.append(label,count);
        if(key.zone){cell.style.gridColumn=`${key.column} / span ${key.colSpan}`;cell.style.gridRow=`${key.row} / span ${key.rowSpan}`;label.textContent=key.label.replace(/^Num /,'');}
        keyboardCells.set(key.id,cell);return cell;
      }
      if(row[0]?.zone){
        node.className='keyboard-'+row[0].zone;
        for(const key of row)node.append(createCell(key));
        clusters.push(node);continue;
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
      if(row[0]?.supplemental)clusters.push(node);else main.append(node);
    }
    layout.replaceChildren(...(view.fullSize?[main]:Array.from(main.children)),...clusters);
  }
  for(const key of rows.flat()){
    const cell=keyboardCells.get(key.id),hex=keyColor(key.count,max);
    cell.style.setProperty('--key-bg',hex);cell.style.setProperty('--key-text',T.textFor(hex));
    const label=key.zone?key.label.replace(/^Num /,''):key.label;
    cell.firstElementChild.textContent=label;
    cell.lastElementChild.textContent=K.compact(key.count,I.locale,arrowIds.includes(key.id));
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
  const keys=view.visibleRanked,max=Math.max(1,...keys.map(key=>key.count));
  const signature=JSON.stringify(keys.map(key=>key.id).sort());
  if(container.dataset.keys!==signature){
    const active=new Set(keys.map(key=>key.id));
    for(const id of keyboardBlocks.keys())if(!active.has(id))keyboardBlocks.delete(id);
    container.dataset.keys=signature;
    container.replaceChildren(...keys.map(key=>{
      if(keyboardBlocks.has(key.id))return keyboardBlocks.get(key.id);
      const cell=document.createElement('div');cell.className='keyboard-block';cell.dataset.key=key.id;cell.tabIndex=0;cell.setAttribute('role','img');
      const surface=document.createElement('span');surface.className='keyboard-block-surface';
      const label=document.createElement('span');label.className='keyboard-block-label';label.textContent=key.label;
      const count=document.createElement('span');count.className='keyboard-block-count';
      surface.append(label,count);cell.append(surface);
      keyboardBlocks.set(key.id,cell);return cell;
    }));
    if(!keys.length){const empty=document.createElement('div');empty.className='empty-label';container.append(empty);}
  }
  if(!keys.length)container.firstElementChild.textContent=t('common.empty');
  for(const key of K.treemap(keys,width,height)){
    const cell=keyboardBlocks.get(key.id),color=keyColor(key.count,max);
    for(const [name,value]of [['left',key.x/width],['top',key.y/height],['width',key.w/width],['height',key.h/height]])cell.style[name]=value*100+'%';
    cell.style.setProperty('--block-gap',Math.min(1,key.w*.1,key.h*.1)+'px');
    cell.style.setProperty('--block-font',Math.min(24,Math.max(11,Math.min(key.w,key.h)/4))+'px');
    cell.style.setProperty('--key-bg',color);cell.style.setProperty('--key-text',T.textFor(color));
    const surface=cell.firstElementChild;
    const hideLabel=key.w<Math.max(28,key.label.length*8+8)||key.h<18;
    surface.firstElementChild.hidden=hideLabel;
    const countEl=surface.lastElementChild;
    const compactText=K.compact(key.count,I.locale,true);
    countEl.textContent=compactText;
    countEl.hidden=hideLabel||key.w<Math.max(46,compactText.length*8+12)||key.h<42;
    const detail=t('keys.tooltip',{key:key.label,count:n(key.count),percent:I.percent(view.total?key.count/view.total:0)});
    cell.title=detail;cell.setAttribute('aria-label',detail);
  }
}
send('ready');

$('takeover').addEventListener('click',()=>send('takeover'));
$('autostart').addEventListener('change',event=>{if(data?.auto_start_available&&event.target.checked!==!!data.auto_start)send('autostart');});

$('uninstall').addEventListener('click',()=>{if(!$('uninstall-row').hidden&&!$('uninstall').disabled)send('uninstall');});
