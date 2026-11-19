'use strict';
// Entries are already in the HTML. JavaScript only translates and filters them.
(()=>{
 const C=ClickPulseShortcutCatalog,$=id=>document.getElementById(id);
 let locale='zh-CN',category='',query='';
 const translations=Array.from(document.querySelectorAll('[data-en]'),element=>({element,zh:element.textContent,en:element.dataset.en}));
 const byId=new Map(C.entries.map(entry=>[entry.id,entry]));
 const rows=Array.from(document.querySelectorAll('[data-entry-id]'),element=>({element,entry:byId.get(element.dataset.entryId)}));
 const categories=Array.from(document.querySelectorAll('[data-category]'));
 function filter(){
  let count=0;
  for(const {element,entry}of rows){
   const visible=!!entry&&(!category||entry.category===category)&&C.search(entry,query);
   element.hidden=!visible;if(visible)count++;
  }
  $('count').textContent=locale==='zh-CN'?count+' 条参考词条':count+' reference '+(count===1?'entry':'entries');
  $('empty').hidden=count!==0;
  for(const button of categories)button.setAttribute('aria-pressed',String(button.dataset.category===category));
 }
 for(const button of categories)button.addEventListener('click',()=>{category=button.dataset.category;filter();});
 $('query').addEventListener('input',event=>{query=event.target.value;filter();});
 $('language').addEventListener('click',()=>{
  locale=locale==='zh-CN'?'en':'zh-CN';document.documentElement.lang=locale;
  for(const {element,zh,en}of translations)element.textContent=locale==='zh-CN'?zh:en;
  $('query').placeholder=locale==='zh-CN'?'复制、截屏、Ctrl…':'Copy, screenshot, Ctrl…';
  filter();
 });
 filter();
 for(const element of document.querySelectorAll('[data-js]'))element.hidden=false;
})();
