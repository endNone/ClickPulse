'use strict';
function createClickPulseI18n(messages, initialLocale='en',platform='macos') {
  function resolveLocale(value){
    const tag=String(value||'').trim().replace(/_/g,'-').split(/[.@]/)[0].toLowerCase();
    const parts=tag.split('-');
    if(parts[0]==='zh'){
      const traditional=!parts.includes('hans')&&parts.some(part=>['hant','tw','hk','mo'].includes(part));
      return traditional?'zh-TW':'zh-CN';
    }
    if(parts[0]==='ja')return 'ja';
    if(parts[0]==='ko')return 'ko';
    if(['fr','de','es'].includes(parts[0]))return parts[0];
    return 'en';
  }
  let locale=resolveLocale(initialLocale),timezone='system',zoneId=Intl.DateTimeFormat().resolvedOptions().timeZone;
  let numbers=new Intl.NumberFormat(locale);
  const percents=new Map(),dates=new Map(),decimals=new Map();
  const api={
    get locale(){return locale;},
    get timezone(){return timezone;},
    get effectiveTimeZone(){return zoneId;},
    setTimeZone(value='system'){
      if(value===timezone&&value!=='system')return false;
      const next=value==='system'?Intl.DateTimeFormat().resolvedOptions().timeZone:value;
      new Intl.DateTimeFormat('en',{timeZone:next});
      const changed=timezone!==value||zoneId!==next;
      timezone=value;zoneId=next;if(changed)dates.clear();return changed;
    },
    setLocale(value){locale=resolveLocale(value);numbers=new Intl.NumberFormat(locale);percents.clear();dates.clear();decimals.clear();},
    t(key,values={}) {
      const variant=key+'.windows';
      const platformKey=platform==='windows'&&(messages[locale]?.[variant]||messages.en?.[variant])?variant:key;
      const message=messages[locale]?.[platformKey]??messages.en?.[platformKey]??messages.en?.[key]??key;
      return message.replace(/\{([a-zA-Z_]+)\}/g,(match,name)=>Object.prototype.hasOwnProperty.call(values,name)?String(values[name]):match);
    },
    number(value){return numbers.format(value||0);},
    decimal(value,digits=1){
      if(!decimals.has(digits))decimals.set(digits,new Intl.NumberFormat(locale,{minimumFractionDigits:digits,maximumFractionDigits:digits}));
      return decimals.get(digits).format(value||0);
    },
    percent(value,digits=2){
      if(!percents.has(digits))percents.set(digits,new Intl.NumberFormat(locale,{style:'percent',minimumFractionDigits:digits,maximumFractionDigits:digits}));
      return percents.get(digits).format(value||0);
    },
    date(value,options={}){
      if(!options.timeZone)options={...options,timeZone:zoneId};
      const key=JSON.stringify(Object.entries(options).sort(([a],[b])=>a.localeCompare(b)));
      if(!dates.has(key))dates.set(key,new Intl.DateTimeFormat(locale,options));
      return dates.get(key).format(new Date(value));
    },
    day(value,options={year:'numeric',month:'short',day:'numeric'}){
      const day=value.length===7?value+'-01':value;
      return api.date(day+'T00:00:00Z',{...options,timeZone:'UTC'});
    },
    month(value){return api.date(Date.UTC(2020,Number(value)-1,1),{month:'short',timeZone:'UTC'});},
    weekdays(){return [0,1,2,3,4,5,6].map(offset=>api.date(Date.UTC(2020,0,6+offset),{weekday:'short',timeZone:'UTC'}));},
    apply(root) {
      root.documentElement.lang=locale;
      for(const node of root.querySelectorAll('[data-i18n]'))node.textContent=api.t(node.dataset.i18n,node.dataset);
      for(const [data,attribute] of [['i18nAria','aria-label'],['i18nTitle','title']]) {
        const selector=data==='i18nAria'?'[data-i18n-aria]':'[data-i18n-title]';
        for(const node of root.querySelectorAll(selector))node.setAttribute(attribute,api.t(node.dataset[data]));
      }
    }
  };
  return api;
}
const ClickPulseI18n=typeof window==='undefined'?null:createClickPulseI18n(window.clickpulseMessages,window.clickpulseLocale,window.clickpulsePlatform);
if(typeof module!=='undefined')module.exports=createClickPulseI18n;
