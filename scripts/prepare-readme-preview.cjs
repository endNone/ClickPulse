#!/usr/bin/env node
'use strict';

// Render the App's own UI with reproducible synthetic data, never a live DB.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const root = path.resolve(__dirname, '..');
const output = path.join(root, 'dist', 'readme-preview');
const K = require('../ui/keyboard.js');
let seed = 0xC11C2026;
const random = () => {
  seed ^= seed << 13; seed ^= seed >>> 17; seed ^= seed << 5;
  return (seed >>> 0) / 4294967296;
};
const history = [];
for (let stamp = Date.UTC(2026, 3, 1); stamp <= Date.UTC(2026, 9, 8); stamp += 86400000) {
  const date = new Date(stamp), weekend = [0, 6].includes(date.getUTCDay());
  const idle = random() < (weekend ? .42 : .04);
  const effort = weekend ? .35 + random() * .45 : .7 + random() * .8;
  history.push({day: date.toISOString().slice(0, 10), counts: {
    keyboard: idle ? 0 : Math.round((3500 + random() * 11000) * effort),
    mouse: idle ? 0 : Math.round((750 + random() * 3400) * effort)
  }});
}
history.at(-1).counts = {keyboard: 14862, mouse: 3918};
const sum = rows => rows.reduce((a, d) => ({keyboard:a.keyboard+d.counts.keyboard, mouse:a.mouse+d.counts.mouse}), {keyboard:0, mouse:0});
const lifetime = sum(history);
const weights = {space:18,backspace:9,E:8,T:6,A:6,O:5,I:5,N:4.5,S:4.5,R:4,return:3,'command-left':2.5,H:2.3,L:2.1,D:2,C:1.8,U:1.5,M:1.4,'shift-left':1.2};
const mainKeys = K.distribution().rows.flat();
const entries = mainKeys.map(key => ({code:key.vk[0], weight:weights[key.id] || (key.id.startsWith('f') || key.id === 'fn' ? 0 : .08 + random() * .75)}));
const weightTotal = entries.reduce((a, k) => a + k.weight, 0);
const physical = entries.map(k => ({code:k.code,count:Math.floor(k.weight / weightTotal * lifetime.keyboard)}));
physical.find(k => k.code === 49).count += lifetime.keyboard - physical.reduce((a, k) => a + k.count, 0);
const snapshot = {
  now:'2026-10-08T15:42:00Z', today:history.at(-1).counts, week:sum(history.slice(-7)), lifetime,
  days:history.slice(-90), history_days:history,
  windows:[{seconds:60,counts:{keyboard:38,mouse:9},approximate:false},{seconds:600,counts:{keyboard:452,mouse:108},approximate:false},{seconds:3600,counts:{keyboard:2146,mouse:594},approximate:false},{seconds:86400,counts:history.at(-1).counts,approximate:false}],
  keyboard_stats:{physical,history_available:false}, top_keys:physical,
  capture:{listening:true,permission_needed:false,error:null,error_code:null},
  demo:true,paused:false,source_mode:false,setup_pending:false,auto_start:false,legacy_imported:false,
  database_path:'Demo data — no database connection'
};
assert.equal(physical.reduce((n,k)=>n+k.count,0), lifetime.keyboard);
assert.deepEqual(sum(history), lifetime);
const scripts = ['i18n.js','themes.js','dropdown.js','history.js','keyboard.js','app.js','presentation.js'];
const styles = ['style.css','history.css','compact.css','themes.css','dropdown.css'];
const fonts = [['ClickPulse UI','Inter.ttf'],['ClickPulse Sans','NotoSansSC.ttf'],['ClickPulse Mono','NotoSansMono.ttf']];
fs.mkdirSync(path.join(output,'fonts'),{recursive:true});
for (const [,name] of fonts) fs.copyFileSync(path.join(root,'assets','fonts',name),path.join(output,'fonts',name));
const fontCSS = fonts.map(([family,name]) => `@font-face{font-family:'${family}';src:url('fonts/${name}') format('truetype');font-weight:100 900;font-style:normal;font-display:swap}`).join('\n');
// Full-page documentation capture reveals scrollable content without changing
// the App's card geometry. Optional density renders original text/vectors at
// more pixels; it does not enlarge an already-compressed screenshot.
const captureCSS = `body:not(.compact){height:auto;min-height:100vh;overflow:visible}body:not(.compact)>main{overflow:visible}
body[data-capture-scale]{zoom:var(--capture-scale);width:var(--capture-width)}
body:not(.compact)[data-capture-scale]{min-height:calc(100vh / var(--capture-scale))}
body.compact[data-capture-scale]{width:320px;height:208px}`;
const boot = `(() => {
  const options = new URLSearchParams(location.search);
  const locale = options.get('locale') === 'zh-CN' ? 'zh-CN' : 'en';
  const scale = Number(options.get('scale'));
  if([2,3].includes(scale)){
    document.body.dataset.captureScale=String(scale);
    document.body.style.setProperty('--capture-scale',String(scale));
    document.body.style.setProperty('--capture-width','1280px');
  }
  const view = {compact:options.get('page') === 'mini',pinned:true,visible:true,locale,language_preference:locale,theme:'blue'};
  const snapshot = ${JSON.stringify(snapshot)};
  window.ipc = {postMessage(message){
    const request = JSON.parse(message);
    if(request.action === 'language'){view.locale = request.language === 'zh-CN' ? 'zh-CN' : 'en';view.language_preference=request.language;}
    if(request.action === 'theme')view.theme=request.theme;
    if(request.action === 'compact')view.compact=true;
    if(request.action === 'expand')view.compact=false;
    if(request.action === 'pin')view.pinned=!view.pinned;
    if(request.action === 'pause')snapshot.paused=!snapshot.paused;
    window.receiveView(view);window.receive(snapshot);
  }};
  window.receiveView(view);window.receive(snapshot);
  const page = options.get('page') || 'overview';
  if(['history','keys','settings'].includes(page))document.querySelector('nav button[data-page="'+page+'"]').click();
  if(page === 'keys' && options.get('view') === 'treemap')document.getElementById('keyboard-view-toggle').click();
  document.fonts.ready.then(() => document.documentElement.dataset.previewReady='true');
})();`;
let html = fs.readFileSync(path.join(root,'ui/index.html'),'utf8')
  .replaceAll('__LOCALE__','en').replaceAll('__PLATFORM__','macos').replaceAll('__THEME__','blue').replaceAll('__VERSION__','0.1.0')
  .replaceAll('__BRAND_MARK__','data:image/png;base64,'+fs.readFileSync(path.join(root,'assets/clickpulse-mark-blue-transparent-v1-1024.png')).toString('base64'))
  .replace('/*__MESSAGES__*/',fs.readFileSync(path.join(root,'ui/locales.json'),'utf8'))
  .replace('/*__STYLE__*/',fontCSS+'\n'+styles.map(name=>fs.readFileSync(path.join(root,'ui',name),'utf8')).join('\n')+'\n'+captureCSS)
  .replace('/*__SCRIPT__*/',scripts.map(name=>fs.readFileSync(path.join(root,'ui',name),'utf8')).join('\n')+'\n'+boot);
assert(!/__VERSION__|__BRAND_MARK__|\/\*__SCRIPT__\*\//.test(html));
fs.writeFileSync(path.join(output,'index.html'),html);
fs.writeFileSync(path.join(output,'synthetic-snapshot.json'),JSON.stringify(snapshot,null,2)+'\n');
// The desktop animation embeds this same isolated mini UI, not a rebuilt mock.
fs.copyFileSync(path.join(__dirname,'mini-desktop-demo.html'),path.join(output,'mini-desktop.html'));
fs.copyFileSync(path.join(__dirname,'mini-demo-timeline.js'),path.join(output,'mini-demo-timeline.js'));
fs.copyFileSync(path.join(root,'docs/images/mini-desktop-scenery.png'),path.join(output,'mini-desktop-scenery.png'));
const miniTimeline = require('./mini-demo-timeline.js');
fs.writeFileSync(path.join(output,'mini-demo-manifest.json'),JSON.stringify({
  width:1350,height:850,logical_width:1080,logical_height:680,capture_scale:1.25,frame_count:miniTimeline.frameCount,
  frame_ms:miniTimeline.frameMs,duration_ms:miniTimeline.durationMs,
  initial_counts:snapshot.today,
  input_events:miniTimeline.events,
  final_state:miniTimeline.stateAt(miniTimeline.maxFrame)
},null,2)+'\n');
console.log(`Preview prepared in ${output}; ${history.length} synthetic days. No database read or capture started.`);
