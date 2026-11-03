const test=require('node:test');
const assert=require('node:assert/strict');
const {readFileSync}=require('node:fs');
const create=require('../ui/i18n.js');
const messages=require('../ui/locales.json');

test('Windows copy follows locale while using PC and system tray terminology',()=>{
  const i=create(messages,'en','windows');
  assert.match(i.t('settings.privacy_description'),/this PC/);
  assert.equal(i.t('native.hide'),'Hide to system tray');
  assert.equal(i.t('mini.hide_title'),'Hide to system tray');
  i.setLocale('zh-CN');assert.equal(i.t('native.hide'),'收起到系统托盘');
  assert.equal(create(messages,'en','macos').t('native.hide'),'Hide to menu bar');
});

test('all eight catalogs cover the same keys and preserve every template parameter',()=>{
  assert.deepEqual(Object.keys(messages).sort(),['de','en','es','fr','ja','ko','zh-CN','zh-TW']);
  const parameters=value=>[...value.matchAll(/\{([a-zA-Z_]+)\}/g)].map(match=>match[1]).sort();
  for(const [locale,catalog] of Object.entries(messages)){
    assert.deepEqual(Object.keys(messages.en).sort(),Object.keys(catalog).sort(),locale);
    for(const key of Object.keys(messages.en)){
      assert.ok(catalog[key].trim(),locale+': '+key);
      assert.deepEqual(parameters(messages.en[key]),parameters(catalog[key]),locale+': '+key);
    }
  }
  for(const [key,message] of Object.entries(messages.en))assert.doesNotMatch(message,/[\p{Script=Han}]/u,key);
});

test('all static UI and native message keys resolve in every language',()=>{
  const html=readFileSync('ui/index.html','utf8');
  const files=['src/desktop.rs','src/capture.rs','src/runtime.rs','ui/app.js','ui/presentation.js','ui/sport.js'];
  const keys=new Set([...html.matchAll(/data-i18n(?:-aria|-title)?="([^"]+)"/g)].map(match=>match[1]));
  for(const file of files){
    const source=readFileSync(file,'utf8');
    for(const match of source.matchAll(/["']((?:page|metric|common|chart|calendar|history|keys|settings|status|mini|native|action|message|error|demo|window|close|sport)\.[a-zA-Z_.]+)["']/g))if(!match[1].endsWith('_'))keys.add(match[1]);
  }
  for(const key of keys)for(const language of Object.keys(messages))assert.equal(typeof messages[language][key],'string',`${language}: ${key}`);
});

test('regional aliases and Chinese scripts select the intended complete catalog',()=>{
  for(const [tag,locale] of [['zh_CN.UTF-8','zh-CN'],['zh-Hans-TW','zh-CN'],['zh-SG','zh-CN'],['zh-Hant-HK','zh-TW'],['ZH_hk','zh-TW'],['zh-MO','zh-TW'],['ja-JP','ja'],['ko_KR.UTF-8','ko'],[' en-GB ','en'],['pt-BR','en'],['fr-CA','fr'],['de_DE.UTF-8','de'],['es-MX','es']]){
    const i=create(messages,tag);assert.equal(i.locale,locale,tag);
    assert.equal(i.t('settings.personalization'),messages[locale]['settings.personalization'],tag);
    i.setLocale('zh-Hant');assert.equal(i.locale,'zh-TW');
  }
});

test('missing translations use the English platform variant and decimal formatting survives language switches',()=>{
  const partial={'zh-CN':{},en:{'native.hide.windows':'Hide to system tray','native.hide':'Hide to menu bar'}};
  assert.equal(create(partial,'zh-CN','windows').t('native.hide'),'Hide to system tray');
  const i=create(messages,'ja-JP');assert.equal(i.decimal(12.75,1),'12.8');
  i.setLocale('ko-KR');assert.equal(i.decimal(0,1),'0.0');
  assert.equal(i.t('sport.skin_warp'),messages.ko['sport.skin_warp']);
  for(const locale of ['fr','de','es']){i.setLocale(locale);assert.equal(i.decimal(12.75,1),'12,8');}
  i.setLocale('unsupported');assert.equal(i.locale,'en');assert.equal(i.decimal(12.75,1),'12.8');
});

test('switching language updates formatting without changing statistics values',()=>{
  const i=create(messages,'en');
  assert.equal(i.t('metric.today'),'Today');
  assert.equal(i.number(1987549),'1,987,549');
  assert.equal(i.percent(.125),'12.50%');
  assert.equal(i.month(10),'Oct');
  i.setLocale('zh-CN');
  assert.equal(i.t('metric.today'),'今日');
  assert.equal(i.number(1987549),'1,987,549');
  assert.equal(i.month(10),'10月');
  i.setLocale('unsupported');
  assert.equal(i.locale,'en');
});

test('calendar labels stay in their original month and weekday across time zones',()=>{
  const i=create(messages,'en');
  assert.deepEqual(i.weekdays(),['Mon','Tue','Wed','Thu','Fri','Sat','Sun']);
  assert.equal(i.month(1),'Jan');
  assert.equal(i.month(12),'Dec');
  assert.match(i.date('2026-03-08T12:00:00-07:00',{month:'long',day:'numeric',timeZone:'America/Los_Angeles'}),/March 8/);
});

test('display dates are localized without moving a saved calendar day across time zones',()=>{
  const i=create(messages,'en'),zone=process.env.TZ;
  try{
    for(const timezone of ['America/Los_Angeles','Pacific/Honolulu','Asia/Shanghai']){
      process.env.TZ=timezone;
      for(const day of ['2026-03-08','2026-11-01','2024-02-29']){
        assert.equal(i.day(day,{day:'numeric'}),String(Number(day.slice(-2))));
      }
      assert.equal(i.day('2026-09-25',{month:'short',day:'numeric'}),'Sep 25');
      assert.equal(i.day('2026-10',{month:'short'}),'Oct');
    }
  }finally{if(zone===undefined)delete process.env.TZ;else process.env.TZ=zone;}
  assert.equal(i.day('2026-10-08'),'Oct 8, 2026');
  i.setLocale('zh-CN');
  assert.equal(i.day('2026-09-25',{month:'short',day:'numeric'}),'9月25日');
  assert.equal(i.day('2026-10-08'),'2026年10月8日');
  assert.equal(i.day('2026-10',{month:'short'}),'10月');
});

test('template values are inserted literally and never interpreted as other parameters',()=>{
  const i=create(messages,'en');
  const path='/tmp/<file>{count}.csv';
  assert.equal(i.t('message.exported',{path}),`Exported: ${path}`);
  assert.equal(i.t('keys.tooltip',{key:'A',count:'12',percent:'1.00%'}),'A · 12 key presses · 1.00%');
});

test('static translation changes labels and access names while preserving icon controls',()=>{
  const label={dataset:{i18n:'action.mini'},textContent:''};
  const icon={dataset:{i18nAria:'mini.pin',i18nTitle:'mini.pin'},children:['svg'],attributes:{},setAttribute(name,value){this.attributes[name]=value;}};
  const root={documentElement:{lang:'zh-CN'},querySelectorAll(selector){return selector==='[data-i18n]'?[label]:[icon];}};
  create(messages,'en').apply(root);
  assert.equal(root.documentElement.lang,'en');
  assert.equal(label.textContent,'Mini mode');
  assert.equal(icon.attributes['aria-label'],'Pin window');
  assert.equal(icon.attributes.title,'Pin window');
  assert.deepEqual(icon.children,['svg']);
});
