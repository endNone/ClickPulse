/* Documentation only: deterministic clock/input for the production Sport view.
 * Arrow Right advances one capture frame; Home reloads the initial state.
 * No OS events, database connection or saved preferences are involved.
 */
(() => {
  const timing=window.clickpulseSportDemoTiming;
  let now=0,nextId=1,frame=0,keyboard=2000,mouse=1000,fraction=0;
  const intervals=new Map(),animations=new Map();
  const realPost=()=>window.ipc.postMessage.bind(window.ipc);
  Object.defineProperty(window.performance,'now',{value:()=>now});
  window.requestAnimationFrame=callback=>{const id=nextId++;animations.set(id,callback);return id;};
  window.cancelAnimationFrame=id=>animations.delete(id);
  window.setInterval=(callback,ms)=>{const id=nextId++;intervals.set(id,{callback,ms,next:now+ms});return id;};
  window.clearInterval=id=>intervals.delete(id);
  const rate=ms=>ms<1500?1:ms<3000?3:ms<4500?6:ms<6000?10:14;
  let inputIndex=0;
  function advance(){
    const end=Math.round((frame+1)*1000/timing.fps);
    while(now<end){
      const step=Math.min(10,end-now);now+=step;
      fraction+=rate(now)*step/1000;
      while(fraction>=1){if(inputIndex++%4===3)mouse++;else keyboard++;fraction--;}
      for(const timer of intervals.values())while(timer.next<=now){timer.next+=timer.ms;timer.callback();}
      const callbacks=[...animations.values()];animations.clear();callbacks.forEach(callback=>callback(now));
    }
    frame++;renderCounters();
  }
  let snapshot;
  function renderCounters(){
    const keys=keyboard-2000,clicks=mouse-1000,updated=structuredClone(snapshot);
    for(const field of ['today','week','lifetime']){updated[field].keyboard+=keys;updated[field].mouse+=clicks;}
    window.receive(updated);
    document.documentElement.dataset.frame=String(frame);
    document.documentElement.dataset.rank=document.getElementById('mini-sport-grade').textContent;
    document.documentElement.dataset.keys=String(keys);document.documentElement.dataset.clicks=String(clicks);
    if(window.parent!==window)window.parent.postMessage({type:'sport-demo-state',frame,frameCount:timing.frame_count,keys,clicks,rank:document.documentElement.dataset.rank},location.origin);
  }
  window.addEventListener('DOMContentLoaded',async()=>{
    await document.fonts.ready;
    snapshot=window.clickpulseSportDemoSnapshot;
    const forward=realPost();
    window.ipc.postMessage=message=>{
      if(JSON.parse(message).action==='sport_sample'){window.receiveSport({keyboard,mouse,paused:false,available:true});return;}
      forward(message);
    };
    // Establish a fresh cumulative baseline at virtual time zero.
    window.receiveSport({keyboard,mouse,paused:true,available:true});
    window.receiveSport({keyboard,mouse,paused:false,available:true});
    renderCounters();document.documentElement.dataset.previewReady='true';
    window.addEventListener('message',event=>{
      if(event.origin!==location.origin||event.source!==window.parent)return;
      if(event.data?.type==='sport-demo-step'&&frame<timing.frame_count-1)advance();
      if(event.data?.type==='sport-demo-request-state')renderCounters();
    });
    document.addEventListener('keydown',event=>{
      if(event.key==='ArrowRight'){event.preventDefault();if(frame<timing.frame_count-1)advance();}
      if(event.key==='Home'){event.preventDefault();location.reload();}
    });
  });
})();
