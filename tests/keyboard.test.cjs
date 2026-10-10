const { test } = require('node:test');
const assert = require('node:assert/strict');
const K = require('../ui/keyboard.js');
const counts = view => new Map(view.rows.flat().map(key => [key.id, key.count]));

test('frequency blocks conserve relative area and cover the canvas once across shapes and skewed usage',()=>{
  for(const [width,height]of [[640,279],[360,279],[1200,279],[279,640]]){
    for(const weights of [[100],[8,4,2,1],[200000,100000,9000,100,2,1],Array.from({length:70},(_,i)=>i+1)]){
      const keys=weights.map((count,i)=>({id:String(i),count})),sum=weights.reduce((a,b)=>a+b,0);
      const blocks=K.treemap(keys,width,height);
      assert.equal(blocks.length,keys.length);
      assert.equal(new Set(blocks.map(key=>key.id)).size,keys.length);
      for(const block of blocks){
        assert.ok(block.w>0&&block.h>0);
        assert.ok(block.x>=-1e-7&&block.y>=-1e-7&&block.x+block.w<=width+1e-7&&block.y+block.h<=height+1e-7);
        assert.ok(Math.abs(block.w*block.h/(width*height)-block.count/sum)<1e-9);
      }
      for(let i=0;i<blocks.length;i++)for(let j=i+1;j<blocks.length;j++){
        const a=blocks[i],b=blocks[j];
        const overlapW=Math.min(a.x+a.w,b.x+b.w)-Math.max(a.x,b.x),overlapH=Math.min(a.y+a.h,b.y+b.h)-Math.max(a.y,b.y);
        assert.ok(overlapW<1e-7||overlapH<1e-7,'blocks must not overlap');
      }
      assert.ok(Math.abs(blocks.reduce((area,key)=>area+key.w*key.h,0)-width*height)<1e-7);
      assert.deepEqual(keys,weights.map((count,i)=>({id:String(i),count})));
    }
  }
});
test('frequency blocks exclude zero or invalid counts without inventing a remainder key',()=>{
  const keys=[{id:'A',count:1},{id:'B',count:0},{id:'C',count:-3},{id:'D',count:NaN},{id:'E',count:Infinity}];
  assert.deepEqual(K.treemap(keys,100,50),[{id:'A',count:1,x:0,y:0,w:100,h:50}]);
  assert.deepEqual(K.treemap([],100,50),[]);assert.deepEqual(K.treemap(keys,0,50),[]);
});

test('encoded special keys are recognized before the old repair date without double counting',()=>{
  const view=K.distribution({history_available:true,history_before_fix:[
    {code:65536+49,count:1563},{code:65536+51,count:981},{code:65536+36,count:321},
    {code:65536+123,count:35},{code:0x110000+55,count:5}
  ],physical:[{code:49,count:999}]});
  const byKey=counts(view);
  assert.equal(byKey.get('space'),1563);assert.equal(byKey.get('backspace'),981);
  assert.equal(byKey.get('return'),321);assert.equal(byKey.get('left'),35);
  assert.equal(byKey.get('command-left'),5);
  assert.equal(view.total,2905);assert.equal(view.mapped,2905);assert.equal(view.other,0);
});
test('full-width labels and Chinese punctuation share the corresponding printed keys',()=>{
  const view=K.distribution({history_available:true,history_since_fix:[
    {code:'Ａ'.codePointAt(0),count:2},{code:'１'.codePointAt(0),count:3},
    {code:'！'.codePointAt(0),count:4},{code:'，'.codePointAt(0),count:5},
    {code:'。'.codePointAt(0),count:6},{code:'【'.codePointAt(0),count:7}
  ]});
  const byKey=counts(view);
  assert.equal(byKey.get('A'),2);assert.equal(byKey.get('digit0'),7);
  assert.equal(byKey.get('comma'),5);assert.equal(byKey.get('period'),6);
  assert.equal(byKey.get('leftbracket'),7);assert.equal(view.total,27);assert.equal(view.other,0);
});
test('Mac extended keys retain identities while unresolved counts stay outside the layout',()=>{
  const view=K.distribution({physical:[{code:115,count:2},{code:116,count:3},
    {code:107,count:4},{code:86,count:5},{code:21,count:6},{code:999,count:7}]},'physical');
  const byKey=counts(view);
  assert.equal(byKey.get('home'),2);assert.equal(byKey.get('pageup'),3);
  assert.equal(byKey.get('f14'),4);assert.equal(byKey.get('num4'),5);
  assert.equal(byKey.get('digit3'),6);assert.equal(byKey.has('other'),false);
  assert.equal(view.mapped,20);assert.equal(view.other,7);assert.equal(view.total,27);
  assert.equal(view.rows.flat().reduce((sum,key)=>sum+key.count,0)+view.unmapped,view.total);
  assert.equal(view.ranked.some(key=>key.id==='other'),false);
});

test('visible key scope excludes hidden Mac extensions while keeping Windows navigation and all stored counts',()=>{
  const stats={physical:[{code:0,count:5},{code:115,count:1000},{code:86,count:7},{code:999,count:3}]};
  for(const platform of ['macos','windows']){
    const model=K.create(platform),view=model.distribution(stats);
    assert.deepEqual(view.visibleRanked.map(key=>key.id),platform==='macos'?['A']:['home','num4','A']);
    assert.equal(view.ranked.length,3);assert.equal(view.total,1015);
    assert.equal(view.mapped,1012);assert.equal(view.unmapped,3);
    assert.equal(view.ranked.reduce((sum,key)=>sum+key.count,0)+view.unmapped,view.total);
    const historical=model.distribution({history_available:true,history_since_fix:stats.physical.map(row=>({...row,code:2**40+row.code}))});
    assert.deepEqual(historical.visibleRanked,view.visibleRanked);
    assert.equal(historical.total,view.total);
  }
});

test('Windows layout distinguishes modifiers, navigation and number pad without duplicate counts',()=>{
  const windows=K.create('windows');
  const view=windows.distribution({physical:[{code:59,count:2},{code:62,count:3},{code:55,count:4},{code:58,count:5},{code:114,count:6},{code:86,count:7},{code:123,count:8},{code:76,count:9}]});
  const byId=counts(view);
  assert.equal(view.total,44);assert.equal(view.mapped,44);assert.equal(view.unmapped,0);
  assert.equal(byId.get('control-left'),2);assert.equal(byId.get('control-right'),3);
  assert.equal(byId.get('num4'),7);assert.equal(byId.get('left'),8);
  assert.equal(windows.byId.get('command-left').label,'Win');
  assert.equal(windows.byId.get('option-left').label,'Alt');
  assert.equal(windows.byId.get('return').label,'Enter');
  assert.equal(windows.byId.has('fn'),false);
});
test('native physical history has exact attribution while old encodings remain estimated',()=>{
  const windows=K.create('windows');
  const exact=windows.distribution({history_available:true,history_since_fix:[{code:2**40+76,count:11}],physical:[{code:76,count:11}]});
  assert.equal(exact.total,11);assert.equal(exact.mapped,11);assert.equal(exact.estimated,false);
  assert.equal(K.distribution({history_available:true,history_before_fix:[{code:97,count:3}]}).estimated,true);
});

test('historical character and special namespaces stay separate without adding physical totals', () => {
  const view = K.distribution({history_available:true,
    history_before_fix:[{code:97,count:20},{code:53,count:3}],
    history_since_fix:[{code:97,count:10},{code:53,count:5},{code:65536+97,count:7},{code:0x110000+53,count:2}],
    physical:[{code:0,count:999}]});
  const byKey = counts(view);
  assert.equal(byKey.get('A'),30);
  assert.equal(byKey.get('f6'),7);
  assert.equal(byKey.get('esc'),5);
  assert.equal(byKey.get('digit4'),5);
  assert.equal(view.total,47);
  assert.equal(view.mapped,47);
});

test('old ambiguous codes are assigned once and unresolved history preserves totals without adding keys', () => {
  const view = K.distribution({history_available:true,history_before_fix:[{code:49,count:11},{code:48,count:4},{code:40000,count:8}]});
  assert.equal(counts(view).get('space'),11);
  assert.equal(counts(view).get('digit0'),0);
  assert.equal(counts(view).get('tab'),4);
  assert.equal(view.mapped+view.unmapped,view.total);
  assert.equal(view.unmapped,8);
  assert.equal(view.rows.length,6);
  assert.equal(counts(view).has('other'),false);
});

test('physical view uses complete virtual keys and never mixes in character history', () => {
  const view = K.distribution({history_available:true,history_since_fix:[{code:97,count:900}],physical:[{code:0,count:8},{code:97,count:4},{code:51,count:2},{code:999,count:3}]},'physical');
  const byKey = counts(view);
  assert.equal(byKey.get('A'),8);
  assert.equal(byKey.get('f6'),4);
  assert.equal(byKey.get('backspace'),2);
  assert.equal(view.total,17);
  assert.equal(view.unmapped,3);
  assert.equal(view.historical,false);
});

test('shifted characters and duplicate input rows conserve counts', () => {
  const view = K.distribution({history_available:true,history_since_fix:[{code:49,count:5},{code:33,count:2},{code:49,count:3},{code:65,count:7},{code:97,count:11}]});
  assert.equal(counts(view).get('digit0'),10);
  assert.equal(counts(view).get('A'),18);
  assert.equal(view.total,28);
  assert.equal(view.mapped,28);
});

test('App physical records extend the historical total once across the handoff', () => {
  const view = K.distribution({history_available:true,
    history_before_fix:[{code:97,count:100}],
    history_since_fix:[{code:97,count:20},{code:2**40,count:8},{code:2**40+53,count:2}],
    physical:[{code:0,count:28},{code:53,count:2}]});
  assert.equal(counts(view).get('A'),128);
  assert.equal(counts(view).get('esc'),2);
  assert.equal(view.total,130);
  assert.equal(view.mapped,130);
});
