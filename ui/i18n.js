'use strict';
function createClickPulseI18n(messages, initialLocale='en',platform='macos') {
  let locale=initialLocale==='zh-CN'?'zh-CN':'en';
  let numbers=new Intl.NumberFormat(locale);
  const percents=new Map(),dates=new Map();
  const api={
    get locale(){return locale;},
    setLocale(value){locale=value==='zh-CN'?'zh-CN':'en';numbers=new Intl.NumberFormat(locale);percents.clear();dates.clear();},
    t(key,values={}) {
      const platformKey=platform==='windows'&&messages[locale]?.[key+'.windows']?key+'.windows':key;
      const message=messages[locale]?.[platformKey]??messages.en?.[key]??key;
      return message.replace(/\{([a-zA-Z_]+)\}/g,(match,name)=>Object.prototype.hasOwnProperty.call(values,name)?String(values[name]):match);
    },
    number(value){return numbers.format(value||0);},
    percent(value,digits=2){
      if(!percents.has(digits))percents.set(digits,new Intl.NumberFormat(locale,{style:'percent',minimumFractionDigits:digits,maximumFractionDigits:digits}));
      return percents.get(digits).format(value||0);
    },
    date(value,options={}){
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
