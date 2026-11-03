'use strict';
const ClickPulseSport=(() => {
  const ranks=[{rank:'C',min:0,max:2},{rank:'B',min:2,max:5},{rank:'A',min:5,max:8},{rank:'S',min:8,max:12},{rank:'SS',min:12,max:16}];
  const gradeFor=cps=>ranks[cps>=12?4:cps>=8?3:cps>=5?2:cps>=2?1:0];
  // Distribute each observed delta over its sampling interval. This keeps a
  // delayed worker response from turning a batch into an instantaneous spike.
  function createMeter(){
    let previous=null,buckets=[],clicks=0,peak=0,best=0;
    function rate(now){
      // A counter feed is sampled, not an event stream. Hold the latest measured
      // one-second window between normal replies instead of inventing an empty
      // tail every animation frame. A stalled feed still decays after 400 ms.
      if(previous&&now-previous.now<=400)now=previous.now;
      buckets=buckets.filter(b=>b.end>now-1000);
      return buckets.reduce((sum,b)=>sum+b.count*Math.max(0,Math.min(now,b.end)-Math.max(now-1000,b.start))/(b.end-b.start),0);
    }
    function state(now){
      const cps=rate(now),grade=gradeFor(cps);
      peak=Math.max(peak,cps);best=Math.max(best,ranks.indexOf(grade));
      return {cps,grade,clicks,peak,best:ranks[best].rank,progress:Math.min(1,(cps-grade.min)/(grade.max-grade.min))};
    }
    function suspend(){previous=null;buckets=[];}
    function sample(counts,now){
      const {keyboard,mouse}=counts||{};
      if(!Number.isFinite(keyboard)||keyboard<0||!Number.isFinite(mouse)||mouse<0||!Number.isFinite(now)){suspend();return 0;}
      if(!previous||now<=previous.now||now-previous.now>1500||keyboard<previous.keyboard||mouse<previous.mouse){
        previous={keyboard,mouse,now};buckets=[];return 0;
      }
      const delta=keyboard-previous.keyboard+mouse-previous.mouse;
      if(delta){buckets.push({start:previous.now,end:now,count:delta});clicks+=delta;}
      previous={keyboard,mouse,now};state(now);return delta;
    }
    function reset(){suspend();clicks=0;peak=0;best=0;}
    return {sample,state,suspend,reset};
  }

  function createView({document:doc,window:win,translate:t,formatSpeed=value=>value.toFixed(1),send}){
    const el=id=>doc.getElementById(id),widget=el('mini-widget'),canvas=el('sport-canvas');
    const ctx=canvas.getContext?.('2d'),meter=createMeter();
    const clock=()=>win.performance?.now()??Date.now();
    const motion=win.matchMedia?.('(prefers-reduced-motion: reduce)');
    const colors={C:'#58c9ed',B:'#5a9bff',A:'#b59aff',S:'#ffd479',SS:'#ff676f'};
    let active=false,enabled=false,shown=true,paused=false,available=false,poll=null,frame=null;
    let marks=[],width=320,height=208,lastFrame=0,lastHit=-Infinity,phase=0,sequence=0;
    function resize(){
      if(!ctx)return;
      const rect=widget.getBoundingClientRect(),scale=Math.min(win.devicePixelRatio||1,2);
      if(canvas.width===Math.round(rect.width*scale)&&canvas.height===Math.round(rect.height*scale))return;
      width=rect.width;height=rect.height;canvas.width=Math.round(width*scale);canvas.height=Math.round(height*scale);
      ctx.setTransform(scale,0,0,scale,0,0);
    }
    function emit(count,now){
      if(count<=0)return;
      lastHit=now;
      if(!ctx||motion?.matches)return;
      const rank=meter.state(now).grade.rank,amount=Math.min(32,Math.ceil(count));
      // The native feed batches counts. Spread visual strikes across the batch;
      // these marks represent count/rhythm, not invented exact event timestamps.
      for(let i=0;i<amount;i++)marks.push({at:now-120+i/amount*120,step:sequence++,rank});
      if(marks.length>96)marks=marks.slice(-96);
    }
    function line(points,color,alpha=1,lineWidth=1){
      ctx.strokeStyle=color;ctx.globalAlpha=alpha;ctx.lineWidth=lineWidth;ctx.beginPath();
      points.forEach(([x,y],i)=>i?ctx.lineTo(x,y):ctx.moveTo(x,y));ctx.stroke();
    }
    function paintWarp(now,s){
      const cx=width*.76,cy=height*.40,color=colors[s.grade.rank];
      // A projected floor and approaching light gates create forward travel.
      for(let i=-2;i<=3;i++)line([[cx+i*3,cy],[cx+i*width*.33,height]],color,.11);
      for(let i=0;i<9;i++){
        const depth=Math.pow((i/9+phase)%1,2),y=cy+(height-cy)*depth,spread=10+depth*width;
        line([[cx-spread,y],[cx+spread,y]],color,.05+depth*.17);
      }
      for(let i=0;i<22;i++){
        const d=(i*.137+phase*.45)%1,angle=i*2.399,spread=d*d;
        const x=cx+Math.cos(angle)*spread*width*.64,y=cy+Math.sin(angle)*spread*height*.7;
        const tail=.015+Math.min(s.cps,24)*.001;
        line([[x,y],[x+(x-cx)*tail,y+(y-cy)*tail]],color,.13+spread*.42,1);
      }
      for(const m of marks.filter(m=>m.step%3===0)){
        const age=(now-m.at)/950;if(age<0||age>1)continue;
        const d=age*age,w=8+d*170,h=3+d*64;
        line([[cx-w,cy+h*.8],[cx-w*.72,cy-h],[cx+w*.72,cy-h],[cx+w,cy+h*.8]],colors[m.rank],(1-age)*.48,1.2);
      }
      line([[cx-12,cy],[cx+12,cy]],color,.6);
    }
    function paint(dt,now=clock(),s=meter.state(now)){
      if(!ctx)return;ctx.clearRect(0,0,width,height);
      if(!active||motion?.matches){marks=[];return;}
      marks=marks.filter(m=>now-m.at<1400);
      phase=(phase+dt*Math.min(s.cps,32)*.035)%1;
      paintWarp(now,s);
      ctx.globalAlpha=1;
    }
    function render(now=clock()){
      const s=meter.state(now),rank=s.grade.rank,moving=active&&!paused&&available&&s.cps>0&&!motion?.matches;
      widget.dataset.sport=String(active);widget.dataset.sportSkin='warp';widget.dataset.rank=rank;widget.dataset.moving=String(moving);
      widget.style.setProperty('--sport-color',colors[rank]);widget.style.setProperty('--sport-glow',colors[rank]+'33');
      widget.style.setProperty('--sport-energy',String(Math.min(s.cps/16,1)));
      widget.classList.toggle('sport-hit',moving&&now-lastHit<160);
      widget.style.setProperty('--sport-strike',String(moving?Math.max(0,1-(now-lastHit)/220):0));
      el('mini-sport-grade').textContent=rank;el('mini-sport-cps').textContent=formatSpeed(s.cps);
      el('mini-sport-caption').textContent=paused?t('sport.paused'):available?'Sport · '+t('sport.rank_'+rank.toLowerCase()):t('sport.waiting');
      el('mini-sport-readout').setAttribute('aria-label',t('sport.mini_readout',{rank,speed:formatSpeed(s.cps)}));
      return s;
    }
    function tick(now){
      frame=null;if(!active)return;
      const s=render(now);resize();paint(Math.min((now-(lastFrame||now))/1000,.05),now,s);lastFrame=now;
      if(s.cps>0||marks.length)wake();
    }
    function wake(){if(active&&!motion?.matches&&frame===null&&win.requestAnimationFrame)frame=win.requestAnimationFrame(tick);}
    function request(){if(active)send('sport_sample');}
    function setActive(value){
      value=!!value&&!doc.hidden;
      if(active===value){render();return;}
      active=value;meter.suspend();marks=[];lastFrame=0;lastHit=-Infinity;
      el('mini-sport-layer').hidden=!active;el('mini-sport-readout').hidden=!active;
      el('mini-sport-caption').hidden=!active;el('mini-status').hidden=active;
      if(active){resize();paint(0);request();poll=win.setInterval?.(()=>{request();render();},250);}
      else{if(poll!==null)win.clearInterval?.(poll);poll=null;if(frame!==null)win.cancelAnimationFrame?.(frame);frame=null;paint(0);}
      render();
    }
    function sync(view={}){
      enabled=view.sport_enabled===true;shown=view.sport_visible!==false;
      el('mini-sport').hidden=!shown;el('mini-sport').setAttribute('aria-pressed',String(enabled&&shown));
      const title=t(enabled?'sport.disable':'sport.enable');el('mini-sport').setAttribute('aria-label',title);el('mini-sport').title=title;
      el('sport-visible').checked=shown;el('sport-open-mini').disabled=!shown;
      setActive(!!view.compact&&enabled&&shown&&view.visible!==false);
    }
    function receive(sample){
      if(!active)return;paused=!!sample.paused;available=sample.available===true;
      if(paused||!available){meter.suspend();marks=[];lastHit=-Infinity;paint(0);}
      else{const now=clock(),delta=meter.sample(sample,now);emit(delta,now);}
      const s=render();if(s.cps>0||marks.length)wake();
    }
    el('mini-sport').addEventListener('click',()=>send('sport',{enabled:!enabled}));
    el('sport-visible').addEventListener('change',event=>send('sport_visible',{visible:!!event.target.checked}));
    el('sport-open-mini').addEventListener('click',()=>{send('sport',{enabled:true});send('compact');});
    doc.addEventListener?.('visibilitychange',()=>win.updateSportActivity?.());
    motion?.addEventListener?.('change',()=>{paint(0);render();wake();});
    win.addEventListener?.('resize',()=>{resize();wake();});
    render();
    return {sync,receive,render,get active(){return active;}};
  }
  return {ranks,gradeFor,createMeter,createView};
})();
if(typeof module!=='undefined')module.exports=ClickPulseSport;
