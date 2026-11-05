const test=require('node:test');
const assert=require('node:assert/strict');
const {createMeter,gradeFor}=require('../ui/sport.js');
const mouseSample=(meter,mouse,now)=>meter.sample({keyboard:0,mouse},now);

test('input speed uses exact C B A S SS boundaries',()=>{
  assert.deepEqual([0,1.99,2,4.99,5,7.99,8,11.99,12,30].map(cps=>gradeFor(cps).rank),['C','C','B','B','A','A','S','S','SS','SS']);
});

test('SS begins at twelve CPS and remains the session best after energy decays',()=>{
  const meter=createMeter();mouseSample(meter,0,0);
  for(let i=1;i<=4;i++)mouseSample(meter,i*3,i*250);
  assert.equal(meter.state(1000).grade.rank,'SS');
  assert.equal(meter.state(2100).grade.rank,'C');assert.equal(meter.state(2100).best,'SS');
});

test('sampled SS speed stays stable between normal replies but a stalled feed decays',()=>{
  const meter=createMeter();mouseSample(meter,0,0);
  for(let i=1;i<=4;i++)mouseSample(meter,i*3,i*250);
  assert.equal(meter.state(1100).cps,12);
  assert.equal(meter.state(1250).grade.rank,'SS');
  assert.equal(meter.state(1500).cps,6);
  assert.equal(meter.state(2100).cps,0);
});

test('historical totals establish a baseline; sustained input builds and then decays',()=>{
  const meter=createMeter();mouseSample(meter,100000,0);
  assert.equal(meter.state(0).clicks,0);
  for(let i=1;i<=4;i++)mouseSample(meter,100000+i*2,i*250);
  const top=meter.state(1000);
  assert.equal(top.cps,8);assert.equal(top.grade.rank,'S');assert.equal(top.clicks,8);
  assert.equal(meter.state(1500).cps,4);
  const idle=meter.state(2100);
  assert.equal(idle.cps,0);assert.equal(idle.grade.rank,'C');assert.equal(idle.peak,8);assert.equal(idle.best,'S');
});

test('delayed batches are weighted across their actual interval rather than spiking',()=>{
  const meter=createMeter();mouseSample(meter,100,0);mouseSample(meter,108,1000);
  assert.equal(meter.state(1000).cps,8);
  assert.equal(meter.state(1500).cps,4);
});

test('pause, hidden gaps, counter resets and invalid samples cannot create phantom clicks',()=>{
  const meter=createMeter();mouseSample(meter,100,0);mouseSample(meter,101,250);
  meter.suspend();mouseSample(meter,1000,1000);
  assert.equal(meter.state(1000).clicks,1);assert.equal(meter.state(1000).cps,0);
  mouseSample(meter,2000,4000);mouseSample(meter,4,4250);mouseSample(meter,8,4000);
  assert.equal(meter.state(4500).clicks,1);
  mouseSample(meter,NaN,4600);mouseSample(meter,5000,4750);
  assert.equal(meter.state(4750).clicks,1);
  meter.reset();mouseSample(meter,5000,5000);
  assert.equal(meter.state(5000).clicks,0);assert.equal(meter.state(5000).peak,0);assert.equal(meter.state(5000).best,'C');
});

// Drive actual canvas frames, rather than a DOM adapter with canvas disabled.
function scene(reducedMotion=false){
  let now=0,next=0,operations=0;
  const frames=new Map(),timers=new Map(),nodes=new Map();
  const finite=(...values)=>{operations++;for(const value of values)assert.ok(Number.isFinite(value));};
  const ctx={clearRect:finite,setTransform:finite,beginPath(){operations++;},moveTo:finite,lineTo:finite,stroke(){operations++;},fillRect:finite};
  const node=id=>{
    if(!nodes.has(id)){
      const classes=new Set();
      nodes.set(id,{dataset:{},style:{setProperty(){}},setAttribute(){},addEventListener(){},
        classList:{toggle(key,on){on?classes.add(key):classes.delete(key);},contains:key=>classes.has(key)},
        getContext:()=>ctx,getBoundingClientRect:()=>({width:320,height:208})});
    }
    return nodes.get(id);
  };
  const doc={getElementById:node,querySelectorAll:()=>[],addEventListener(){}};
  const win={performance:{now:()=>now},matchMedia:()=>({matches:reducedMotion,addEventListener(){}}),addEventListener(){},
    setInterval(fn){const id=++next;timers.set(id,fn);return id;},clearInterval:id=>timers.delete(id),
    requestAnimationFrame(fn){const id=++next;frames.set(id,fn);return id;},cancelAnimationFrame:id=>frames.delete(id)};
  const view=require('../ui/sport.js').createView({document:doc,window:win,translate:key=>key,send(){}});
  view.sync({compact:true,sport_enabled:true});
  return {view,node,frames,timers,setTime:value=>{now=value;},get operations(){return operations;},
    frame(value){now=value;const pending=[...frames.values()];frames.clear();pending.forEach(fn=>fn(now));}};
}

test('Warp finishes its animation, ignore empty samples as strikes, and stop when hidden',()=>{
  const a=scene();a.view.receive({keyboard:0,mouse:100,available:true});
  a.setTime(250);a.view.receive({keyboard:0,mouse:103,available:true});
  assert.equal(a.node('mini-widget').classList.contains('sport-hit'),true);
  a.frame(300);assert.ok(a.frames.size>0);
  a.setTime(500);a.view.receive({keyboard:0,mouse:103,available:true});
  assert.equal(a.node('mini-widget').classList.contains('sport-hit'),false,'an empty sample must not stamp again');
  a.frame(2200);assert.equal(a.frames.size,0,'expired marks must not keep animation running');
  for(const poll of a.timers.values())poll();assert.equal(a.frames.size,0,'an idle poll must not restart animation');
  a.setTime(2250);a.view.receive({keyboard:0,mouse:103,available:true});
  assert.equal(a.frames.size,0,'an empty idle reply must not schedule a canvas frame');
  const before=a.operations;
  a.setTime(2500);a.view.receive({keyboard:0,mouse:100000,available:true});a.frame(2550);
  assert.ok(a.operations-before<3000,'a large counter batch must have bounded drawing work');
  a.view.sync({compact:true,sport_enabled:true,visible:false});
  assert.equal(a.frames.size,0);assert.equal(a.timers.size,0);
});

test('reduced motion retains Warp and live rank without scheduling canvas animation',()=>{
  const a=scene(true);a.view.receive({keyboard:0,mouse:0,available:true});
  for(let i=1;i<=4;i++){a.setTime(i*250);a.view.receive({keyboard:0,mouse:i*3,available:true});}
  assert.equal(a.node('mini-sport-grade').textContent,'SS');
  assert.equal(a.node('mini-widget').dataset.sportSkin,'warp');
  assert.equal(a.node('mini-widget').dataset.moving,'false');assert.equal(a.frames.size,0);
});

test('keyboard-only input reaches SS while an unchanged counter adds no held-key repeats',()=>{
  const meter=createMeter();meter.sample({keyboard:90000,mouse:50000},0);
  for(let i=1;i<=4;i++)meter.sample({keyboard:90000+i*3,mouse:50000},i*250);
  assert.equal(meter.state(1000).cps,12);assert.equal(meter.state(1000).grade.rank,'SS');
  assert.equal(meter.state(1000).clicks,12);
  assert.equal(meter.sample({keyboard:90012,mouse:50000},1250),0);
  assert.equal(meter.state(1250).clicks,12);
});

test('keyboard and mouse deltas share one rolling window and either counter rollback resets it',()=>{
  const meter=createMeter();meter.sample({keyboard:100,mouse:200},0);
  const batches=[[102,201],[103,203],[105,204],[106,206]];
  batches.forEach(([keyboard,mouse],i)=>meter.sample({keyboard,mouse},(i+1)*250));
  assert.equal(meter.state(1000).cps,12);assert.equal(meter.state(1000).grade.rank,'SS');
  assert.equal(meter.sample({keyboard:105,mouse:300},1250),0,'mouse growth must not mask keyboard reset');
  assert.equal(meter.state(1250).cps,0);
  assert.equal(meter.sample({keyboard:200,mouse:299},1500),0,'keyboard growth must not mask mouse reset');
  assert.equal(meter.state(1500).cps,0);
  meter.sample({keyboard:202,mouse:299},1750);assert.equal(meter.state(1750).cps,2);
  meter.sample({keyboard:NaN,mouse:310},2000);assert.equal(meter.state(2000).cps,0);
});

test('the production Warp view animates and grades keyboard-only samples',()=>{
  const a=scene();a.view.receive({keyboard:10000,mouse:100,available:true});
  for(let i=1;i<=4;i++){a.setTime(i*250);a.view.receive({keyboard:10000+i*3,mouse:100,available:true});}
  assert.equal(a.node('mini-sport-grade').textContent,'SS');
  assert.equal(a.node('mini-sport-cps').textContent,'12.0');
  assert.equal(a.node('mini-widget').dataset.moving,'true');assert.ok(a.frames.size>0);
});
