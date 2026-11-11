#!/usr/bin/env node
'use strict';
// An isolated view of the production Mini renderer, with simulated IPC only.
const {execFileSync}=require('node:child_process');
const fs=require('node:fs');
const path=require('node:path');
const root=path.resolve(__dirname,'..');
const output=path.resolve(process.argv[2]||path.join(root,'dist','sport-preview'));
execFileSync(process.execPath,[path.join(__dirname,'prepare-readme-preview.cjs'),output],{stdio:'inherit'});
const adapter=`<script>
(() => {
  const options=new URLSearchParams(location.search);
  const rates={C:1,B:3,A:6,S:10,SS:14};
  let rank=Object.prototype.hasOwnProperty.call(rates,options.get('rank'))?options.get('rank'):'SS';
  let keyboard=2000,mouse=1000,last=performance.now(),paused=false,manual=false,fractional=0;
  const original=window.ipc.postMessage.bind(window.ipc);
  function advance(){const now=performance.now();if(!paused&&!manual){fractional+=(now-last)/1000*rates[rank];const whole=Math.floor(fractional);if(options.get('input')==='keyboard')keyboard+=whole;else mouse+=whole;fractional-=whole;}last=now;}
  window.ipc.postMessage=message=>{
    const request=JSON.parse(message);
    if(request.action==='sport_sample'){advance();window.receiveSport({keyboard,mouse,paused,available:true,now_ms:Date.now()});return;}
    if(request.action==='pause'){advance();paused=!paused;}
    if(request.action==='expand'&&window.top!==window.self){window.top.location.href='app.html?locale='+encodeURIComponent(options.get('locale')||'en')+'&page=settings';return;}
    original(message);
  };
  window.addEventListener('message',event=>{
    if(event.origin!==location.origin)return;
    if(event.data?.type==='preview-tap'){advance();manual=true;if(!paused)mouse+=1;return;}
    if(event.data?.type!=='preview-rank'||!Object.prototype.hasOwnProperty.call(rates,event.data.rank))return;
    advance();rank=event.data.rank;manual=false;
  });
  if(options.get('page')==='mini'){

    original(JSON.stringify({action:'sport_visible',visible:true}));
    original(JSON.stringify({action:'sport',enabled:true}));
  }
})();
</script>`;
const full=fs.readFileSync(path.join(output,'index.html'),'utf8');
const mobile='<style>@media(max-width:700px){body:not(.compact)>.sidebar{display:none}body:not(.compact) .content{padding:20px 14px}body:not(.compact) header{padding:0 10px}body:not(.compact) .header-actions{gap:6px}body:not(.compact) .page-heading{align-items:flex-start;flex-direction:column;gap:10px}body:not(.compact) h1{font-size:23px}body:not(.compact) .heading-right{display:none}.setting-row{gap:12px}}</style>';
fs.writeFileSync(path.join(output,'app.html'),full.replace('</head>',mobile+'</head>').replace('</body>',adapter+'</body>'));
// The documentation animation steps the unmodified renderer with synthetic input.
const driver=fs.readFileSync(path.join(__dirname,'sport-demo-driver.js'),'utf8');
const timing={fps:60,frame_count:540,duration_ms:9000};
timing.frame_durations_ms=Array.from({length:timing.frame_count},(_,i)=>Math.round((i+1)*1000/timing.fps)-Math.round(i*1000/timing.fps));
const manifest={...timing,ranks:['C','B','A','S','SS'],synthetic:true};
fs.writeFileSync(path.join(output,'sport-demo.html'),full.replace('<head>','<head><script>window.clickpulseSportDemoTiming='+JSON.stringify(timing)+';window.clickpulseSportDemoSnapshot='+fs.readFileSync(path.join(output,'synthetic-snapshot.json'),'utf8')+';'+driver+'</script>'));
fs.writeFileSync(path.join(output,'sport-demo-manifest.json'),JSON.stringify({width:640,height:416,...manifest},null,2)+'\n');
// Reuse the existing scenery/notes scene; keep the Mini UI inside its own iframe.
const desktop=fs.readFileSync(path.join(__dirname,'mini-desktop-demo.html'),'utf8').split('<script src="mini-demo-timeline.js">')[0];
const sceneryStyle='<style>.mini-wrap{left:650px;top:250px;width:384px;height:249.6px}iframe{width:960px;height:624px;transform:scale(.4)}.cursor,.click-ring,.driver{display:none}</style>';
fs.writeFileSync(path.join(output,'sport-desktop.html'),desktop
  .replace('</head>',sceneryStyle+'</head>')
  .replace('index.html?locale=en&amp;page=mini','sport-demo.html?locale=en&amp;page=mini&amp;scale=3&amp;sport=1')
  .replace('ClickPulse · Mini mode desktop demo','ClickPulse · Sport desktop demo')
  .replace('Simulated desktop · Demo data','Simulated desktop · Demo data · C → SS')
  +'<script src="sport-desktop-demo.js"></script></body></html>');
fs.copyFileSync(path.join(__dirname,'sport-desktop-demo.js'),path.join(output,'sport-desktop-demo.js'));
fs.writeFileSync(path.join(output,'sport-desktop-manifest.json'),JSON.stringify({width:2160,height:1360,...manifest},null,2)+'\n');
const skins=[['warp','跃迁','越点越快，穿过层层光门。']];
const cards=skins.map(([skin,name,caption])=>`<article class="preview-card"><div class="card-title"><h2>${name}</h2><span>MINI · 320 × 208</span></div><div class="mini-frame"><iframe title="${name} Mini Sport 演示" data-preview-skin="${skin}" src="about:blank"></iframe></div><p>${caption}</p></article>`).join('');
const galleryCopy={
  "zh-CN": {
    "title": "点击，进入跃迁。",
    "description": "Sport 只在 Mini 模式中显示。用小窗右上角的闪电开关开启；在偏好设置的「个性化」中可以隐藏开关。",
    "rank": "体验等级",
    "settings": "查看个性化设置 →",
    "red": "SS 红色 ≥ 12 CPS",
    "note": "亲手点击，体验每一击的反馈；也可以切换速度档位，看看持续加速的变化。",
    "tap": "点我，感受节奏"
  },
  "zh-TW": {
    "title": "點擊，進入躍遷。",
    "description": "Sport 只在迷你模式中顯示。用小窗右上角的閃電開關開啟；在偏好設定的「個人化」中可以隱藏開關。",
    "rank": "體驗等級",
    "settings": "檢視個人化設定 →",
    "red": "SS 紅色 ≥ 12 CPS",
    "note": "親手點擊，感受每一擊；也可切換速度檔位，體驗持續加速的變化。",
    "tap": "點我，感受節奏"
  },
  "fr": {
    "title": "Cliquez vers l’espace.",
    "description": "Sport apparaît uniquement en mode Mini. Activez-le avec l’éclair. Masquez l’interrupteur dans Réglages → Personnalisation.",
    "rank": "Essayer un niveau",
    "settings": "Ouvrir Personnalisation →",
    "red": "SS rouge ≥ 12 CPS",
    "note": "Cliquez pour ressentir chaque frappe, ou choisissez une vitesse pour explorer l’accélération.",
    "tap": "Cliquez pour essayer"
  },
  "de": {
    "title": "Mit einem Klick in den Warpflug.",
    "description": "Sport erscheint nur im Mini-Modus. Aktivieren Sie den Blitzschalter. Unter Einstellungen → Personalisierung können Sie den Schalter ausblenden.",
    "rank": "Rang testen",
    "settings": "Personalisierung öffnen →",
    "red": "Rotes SS ≥ 12 CPS",
    "note": "Klicken Sie selbst oder wählen Sie eine Geschwindigkeit, um die Beschleunigung zu erleben.",
    "tap": "Selbst klicken"
  },
  "es": {
    "title": "Un clic hacia el salto espacial.",
    "description": "Sport solo aparece en modo Mini. Actívalo con el rayo. Puedes ocultar el interruptor en Configuración → Personalización.",
    "rank": "Probar nivel",
    "settings": "Abrir Personalización →",
    "red": "SS rojo ≥ 12 CPS",
    "note": "Pulsa para sentir cada golpe o elige una velocidad para explorar la aceleración.",
    "tap": "Pulsa para probar"
  },
  "en": {
    "title": "Click into Warp.",
    "description": "Sport lives only in Mini mode. Turn it on with the lightning switch. Hide the switch in Settings → Personalization.",
    "rank": "Try a rank",
    "settings": "Open Personalization →",
    "red": "Red SS ≥ 12 CPS",
    "note": "Tap to feel each strike, or pick a speed to explore sustained acceleration.",
    "tap": "Tap to feel it"
  },
  "ja": {
    "title": "クリックでワープへ。",
    "description": "Sportはミニモード専用です。右上の稲妻スイッチで有効にし、設定の「カスタマイズ」でスイッチを非表示にできます。",
    "rank": "ランクを試す",
    "settings": "カスタマイズを開く →",
    "red": "赤のSS ≥ 12 CPS",
    "note": "クリックで一打ずつ体験するか、速度を選んで加速の変化を試せます。",
    "tap": "クリックで体験"
  },
  "ko": {
    "title": "클릭으로 워프하세요.",
    "description": "Sport는 미니 모드 전용입니다. 오른쪽 위 번개 스위치로 켜고 설정의 개인 설정에서 스위치를 숨길 수 있습니다.",
    "rank": "등급 체험",
    "settings": "개인 설정 열기 →",
    "red": "빨간 SS ≥ 12 CPS",
    "note": "직접 눌러 타격감을 느끼거나 속도를 선택해 가속의 변화를 살펴보세요.",
    "tap": "눌러서 체험"
  }
};
const catalog=require('../ui/locales.json');
const localizationScript=`window.clickpulseMessages=${JSON.stringify(catalog)};${fs.readFileSync(path.join(root,'ui/i18n.js'),'utf8')}
(() => {
  const options=new URLSearchParams(location.search),i=createClickPulseI18n(window.clickpulseMessages,options.get('locale')||'zh-CN');
  const locale=i.locale,copy=${JSON.stringify(galleryCopy)}[locale];
  document.documentElement.lang=locale;
  document.querySelector('h1').textContent=copy.title;
  document.querySelector('header p').textContent=copy.description;
  document.querySelector('.toolbar>span').textContent=copy.rank;
  document.getElementById('tap-preview').textContent=copy.tap;
  document.querySelector('.settings-link').textContent=copy.settings;
  document.querySelector('.settings-link').href='app.html?locale='+encodeURIComponent(locale)+'&page=settings';
  document.querySelector('.note>b').textContent=copy.red;
  document.querySelector('.note').lastChild.textContent=copy.note;
  const select=document.getElementById('preview-language');select.value=locale;select.setAttribute('aria-label',i.t('settings.language'));
  select.addEventListener('change',()=>{location.href='index.html?locale='+encodeURIComponent(select.value);});
  document.querySelectorAll('iframe[data-preview-skin]').forEach(frame=>{
    const skin=frame.dataset.previewSkin,card=frame.closest('.preview-card');
    const name=i.t('sport.skin_'+skin);card.querySelector('h2').textContent=name;card.querySelector('p').textContent=i.t('sport.skin_'+skin+'_description');
    frame.title=name+' · Mini Sport';frame.src='app.html?locale='+encodeURIComponent(locale)+'&page=mini&skin='+skin+'&rank=SS';
  });
})();`;
const gallery=`<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>ClickPulse · Mini Sport</title><style>
@font-face{font-family:Inter;src:url('fonts/Inter.ttf');font-weight:100 900;font-display:swap}@font-face{font-family:Noto;src:url('fonts/NotoSansSC.ttf');font-weight:100 900;font-display:swap}*{box-sizing:border-box}body{margin:0;padding:28px 24px;background:#f3f6fc;color:#172b4a;font-family:Inter,Noto,sans-serif}.wrap{max-width:560px;margin:auto}.eyebrow{font-size:10px;letter-spacing:2px;color:#58728f}h1{font-size:30px;letter-spacing:-1px;margin:10px 0 12px}header p{font-size:12px;line-height:1.9;color:#698099;max-width:610px;margin:0}.toolbar{display:flex;flex-wrap:wrap;align-items:center;gap:8px;margin:22px 0}.toolbar>span{font-size:10px;color:#6d829b;margin-right:3px}.ranks{display:flex;gap:5px}button,select,.settings-link{border:1px solid #d8e3f1;border-radius:6px;padding:7px 12px;background:white;font:600 11px Inter,Noto,sans-serif;color:#48627f;cursor:pointer;text-decoration:none}button[aria-pressed=true]{border-color:#df6a75;color:#b34452;background:#fff0f2}.settings-link{margin-left:auto;color:#315c9c}.grid{display:grid;grid-template-columns:minmax(0,1fr);gap:18px}.preview-card{min-width:0;max-width:370px;width:100%;justify-self:center;padding:17px;background:white;border:1px solid #e0e8f2;border-radius:12px}.card-title{display:flex;align-items:center;justify-content:space-between;gap:8px;margin-bottom:15px}.card-title h2{font-size:13px;margin:0}.card-title span{font-size:7px;letter-spacing:.8px;color:#8b9bb0}.mini-frame{position:relative;width:320px;height:208px;margin:0 auto}iframe{position:absolute;top:0;left:0;width:320px;height:208px;border:0;border-radius:14px;display:block}.preview-card>p{font-size:10px;color:#7c8da4;line-height:1.8;margin:13px 0 0}.note{font-size:10px;line-height:1.9;color:#7c8da4;margin-top:20px}.note b{font-weight:500;color:#a04756}@media(max-width:740px){.grid{grid-template-columns:minmax(0,1fr)}body{padding:24px 12px}.preview-card{padding:15px 10px}.settings-link{margin-left:0}h1{font-size:26px}}@media(max-width:360px){.mini-frame{width:256px;height:166.4px}iframe{transform:scale(.8);transform-origin:0 0}.card-title span{font-size:6px}.toolbar{gap:9px}}
</style></head><body><div class="wrap"><header><span class="eyebrow">CLICKPULSE · PERSONALIZATION</span><h1>小窗，也可以很有动感。</h1><p>Sport 只在 Mini 模式中显示。用小窗右上角的闪电开关开启；在偏好设置的「个性化」中可以隐藏开关。</p></header><div class="toolbar"><select id="preview-language" aria-label="Language"><option value="zh-CN">简体中文</option><option value="zh-TW">繁體中文</option><option value="en">English</option><option value="ja">日本語</option><option value="ko">한국어</option><option value="fr">Français</option><option value="de">Deutsch</option><option value="es">Español</option></select><span>体验等级</span><div class="ranks">${['C','B','A','S','SS'].map(r=>`<button data-rank="${r}" aria-pressed="${r==='SS'}">${r}</button>`).join('')}</div><button id="tap-preview" aria-pressed="false">点我，感受节奏</button><a class="settings-link" href="app.html?locale=zh-CN&page=settings">查看个性化设置 →</a></div><section class="grid">${cards}</section><p class="note"><b>SS 红色 ≥ 12 CPS</b> · S ≥ 8 · A ≥ 5 · B ≥ 2 · C &lt; 2。<br>亲手点击，或选择速度档位感受持续加速。</p></div><script>document.querySelectorAll('[data-rank]').forEach(button=>button.addEventListener('click',()=>{document.getElementById('tap-preview').setAttribute('aria-pressed','false');document.querySelectorAll('[data-rank]').forEach(b=>b.setAttribute('aria-pressed',String(b===button)));for(const frame of document.querySelectorAll('iframe'))frame.contentWindow.postMessage({type:'preview-rank',rank:button.dataset.rank},location.origin);}));document.getElementById('tap-preview').addEventListener('click',()=>{document.querySelectorAll('[data-rank]').forEach(b=>b.setAttribute('aria-pressed','false'));document.getElementById('tap-preview').setAttribute('aria-pressed','true');for(const frame of document.querySelectorAll('iframe'))frame.contentWindow.postMessage({type:'preview-tap'},location.origin);});</script></body></html>`;
fs.writeFileSync(path.join(output,'index.html'),gallery.replace('<script>document.querySelectorAll', '<script>'+localizationScript+'</script><script>document.querySelectorAll'));
console.log('Mini-only Sport preview prepared: the production Warp Mini effect and the actual personalization settings.');
