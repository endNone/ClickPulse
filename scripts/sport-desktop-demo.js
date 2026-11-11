/* Documentation-only scenery around the unchanged production Mini iframe. */
(() => {
  const options=new URLSearchParams(location.search);
  const scale=options.get('scale')==='2'?2:1;
  document.body.style.zoom=String(scale);
  const mini=document.querySelector('iframe'),byId=id=>document.getElementById(id);
  const text='make every keystroke count. keep the next update clear';
  byId('save-state').textContent='Draft';
  byId('draft-placeholder').textContent='Start writing…';
  byId('caret').style.display='none';
  byId('play').hidden=true;byId('previous').hidden=true;
  function step(){mini.contentWindow.postMessage({type:'sport-demo-step'},location.origin);}
  byId('next').addEventListener('click',step);
  document.addEventListener('keydown',event=>{
    if(event.key==='ArrowRight'){event.preventDefault();step();}
    if(event.key==='Home'){event.preventDefault();location.reload();}
  });
  window.addEventListener('message',async event=>{
    if(event.origin!==location.origin||event.source!==mini.contentWindow||event.data?.type!=='sport-demo-state')return;
    const {frame,frameCount,rank,keys,clicks}=event.data;
    byId('draft-text').textContent=text.slice(0,keys);
    byId('draft-placeholder').hidden=keys>0;
    byId('caret').style.display=keys>0&&keys<text.length?'inline-block':'none';
    byId('frame-state').textContent=`Frame ${frame} / ${frameCount-1} · ${rank} · Keys +${keys} · Clicks +${clicks}`;
    Object.assign(document.documentElement.dataset,{frame:String(frame),rank,keys:String(keys),clicks:String(clicks)});
    await document.fonts.ready;
    const wallpaper=document.querySelector('.wallpaper');
    if(!wallpaper.complete)await new Promise(resolve=>wallpaper.addEventListener('load',resolve,{once:true}));
    document.documentElement.dataset.previewReady='true';
  });
  const requestState=()=>mini.contentWindow.postMessage({type:'sport-demo-request-state'},location.origin);
  mini.addEventListener('load',requestState);
  if(mini.contentDocument?.readyState==='complete')requestState();
})();
