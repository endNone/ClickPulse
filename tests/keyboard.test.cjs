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

test('compact layouts keep extended counts visible without duplicating keys',()=>{
  const stats={physical:[{code:0,count:5},{code:115,count:1000},{code:86,count:7},{code:999,count:3}]};
  for(const platform of ['macos','windows']){
    const model=K.create(platform),view=model.distribution(stats,'history','compact');
    assert.deepEqual(view.visibleRanked.map(key=>key.id),['home','num4','A']);
    const visibleRows=view.rows.filter(row=>!row.some(key=>key.supplemental));
    assert.equal(visibleRows.length,6);
    assert.ok(visibleRows[5].some(key=>key.id==='space'));
    assert.ok(!visibleRows.flat().some(key=>['home','num4'].includes(key.id)));
    assert.equal(view.ranked.length,3);assert.equal(view.total,1015);
    assert.equal(view.mapped,1012);assert.equal(view.unmapped,3);
    assert.equal(view.ranked.reduce((sum,key)=>sum+key.count,0)+view.unmapped,view.total);
    const historical=model.distribution({history_available:true,history_since_fix:stats.physical.map(row=>({...row,code:2**40+row.code}))},'history','compact');
    assert.deepEqual(historical.visibleRanked,view.visibleRanked);
    assert.equal(historical.total,view.total);
  }
});

test('Windows modified PrintScreen and Pause aliases retain old history and combine with new captures once',()=>{
  const windows=K.create('windows');
  const physical=[{code:0x10054,count:2},{code:0x10137,count:3},{code:0x10146,count:5},{code:0x10245,count:7}];
  for(const stats of [{physical},{physical,history_available:true,history_since_fix:physical.map(row=>({...row,code:2**40+row.code}))}]){
    const view=windows.distribution(stats),byKey=counts(view);
    assert.equal(byKey.get('print-screen'),5);assert.equal(byKey.get('pause-break'),12);
    assert.equal(view.total,17);assert.equal(view.mapped,17);assert.equal(view.unmapped,0);
    assert.deepEqual(view.visibleRanked.map(key=>key.id),['pause-break','print-screen']);
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

test('Linux uses PC physical identities and Super labels',()=>{
  const linux=require('../ui/keyboard.js').create('linux');
  const distribution=linux.distribution({physical:[{code:0,count:3},{code:55,count:2},{code:62,count:1},{code:0x10137,count:1},{code:0x200e0,count:1}]});
  assert.equal(distribution.total,8);assert.equal(distribution.mapped,7);assert.equal(distribution.unmapped,1);
  assert.equal(linux.byId.get('command-left').label,'Super');
  assert.equal(linux.byId.get('control-right').vk[0],62);
});

test('only the full-size choice exposes the navigation cluster and numpad, conserving all counts',()=>{
  for(const platform of ['macos','windows','linux']){
    const model=K.create(platform);
    assert.equal(model.distribution({physical:[{code:0,count:1}]}).fullSize,false);
    assert.equal(model.distribution({physical:[{code:123,count:1}]}).fullSize,false);
    assert.equal(model.distribution({physical:[{code:123,count:1}]},'history','compact').fullSize,false);
    const stats={physical:[{code:82,count:10},{code:72,count:4},{code:10,count:3},{code:999,count:2}]};
    assert.equal(model.distribution(stats).fullSize,false);
    const view=model.distribution(stats,'history','full_size');
    assert.equal(view.fullSize,true);
    const cells=view.rows.flat();assert.equal(new Set(cells.map(k=>k.id)).size,cells.length);
    assert.equal(cells.find(k=>k.id==='num0').colSpan,2);
    assert.equal(cells.find(k=>k.id==='numplus').rowSpan,platform==='macos'?1:2);
    assert.equal(cells.find(k=>k.id==='numenter').rowSpan,2);
    assert.equal(cells.filter(k=>k.zone==='navigation').length,10);
    assert.deepEqual(view.rows[0].map(k=>k.id),['esc',...Array.from({length:12},(_,i)=>'f'+(i+1))]);
    assert.deepEqual(cells.filter(k=>k.zone==='system').map(k=>k.id),platform==='macos'?['f13','f14','f15']:['print-screen','scroll-lock','pause-break']);
    assert.equal(cells.reduce((sum,k)=>sum+k.count,0)+view.other,19);
    assert.equal(view.other,2);
    assert.equal(model.distribution({},'history','full_size').fullSize,true);
  }
});


test('Mac full-size frame moves the physical Fn key beside Home and keeps Help distinct',()=>{
  const physical=[{code:63,count:7},{code:114,count:3}];
  const history={history_available:true,history_since_fix:physical.map(k=>({...k,code:2**40+k.code}))};
  for(const stats of [{physical},history]){
    const full=K.distribution(stats,'history','full_size').rows.flat();
    const fn=full.filter(k=>k.id==='fn');
    assert.equal(fn.length,1);
    assert.deepEqual([fn[0].zone,fn[0].column,fn[0].row,fn[0].label,fn[0].count],['navigation',1,1,'fn',7]);
    assert.equal(full.find(k=>k.id==='home'&&k.row===1).column,2);
    const help=full.find(k=>k.id==='help');
    assert.equal(help.supplemental,true);
    assert.equal(help.count,3);
    const compact=K.distribution(stats,'history','compact').rows[5];
    assert.equal(compact[0].id,'fn');
    assert.equal(compact[0].count,7);
  }
  const pc=K.create('windows').distribution({physical:[{code:114,count:3}]},'history','full_size');
  const insert=pc.rows.flat().find(k=>k.zone==='navigation'&&k.row===1&&k.column===1);
  assert.deepEqual([insert.id,insert.count],['insert',3]);
});

test('Mac numeric keypad places equals beside Clear and gives plus and minus separate rows',()=>{
  for(const physical of [[],[{code:81,count:1},{code:104,count:3}]]){
    const view=K.distribution({physical},'history','full_size');
    const pad=view.rows.flat().filter(k=>k.zone==='numpad');
    assert.deepEqual(pad.filter(k=>k.row===1).map(k=>[k.id,k.column]),[['numclear',1],['numequals',2],['numdivide',3],['nummultiply',4]]);
    assert.deepEqual(pad.filter(k=>k.column===4).map(k=>[k.id,k.row,k.rowSpan]),[['nummultiply',1,1],['numminus',2,1],['numplus',3,1],['numenter',4,2]]);
    const occupied=new Set();
    for(const key of pad)for(let row=key.row;row<key.row+key.rowSpan;row++)for(let col=key.column;col<key.column+key.colSpan;col++){
      const position=row+':'+col;assert.ok(!occupied.has(position),position);occupied.add(position);
    }
    assert.equal(occupied.size,20);
    assert.equal(view.rows.flat().filter(k=>k.id==='numequals').length,1);
    assert.equal(pad.find(k=>k.id==='numequals').count,physical.length?1:0);
    assert.equal(view.total,physical.length?4:0);
    if(physical.length)assert.equal(view.visibleRanked.find(k=>k.id==='jis-kana').count,3);
  }
});

test('Mac extended function keys occupy the hardware row once, including zero-count keys',()=>{
  const codes=[105,107,113,106,64,79,80],ids=codes.map((_,i)=>'f'+(13+i));
  for(const physical of [[],codes.map((code,i)=>({code,count:i+1})).concat({code:90,count:8})]){
    const view=K.distribution({physical},'history','full_size'),cells=view.rows.flat();
    assert.deepEqual(cells.filter(k=>k.zone==='system').map(k=>[k.id,k.column,k.row]),[['f13',1,1],['f14',2,1],['f15',3,1]]);
    assert.deepEqual(cells.filter(k=>k.zone==='numpad-function').map(k=>[k.id,k.column,k.row]),[['f16',1,1],['f17',2,1],['f18',3,1],['f19',4,1]]);
    for(const [index,id] of ids.entries()){
      const matches=cells.filter(k=>k.id===id);assert.equal(matches.length,1);
      assert.deepEqual(matches[0].vk,[codes[index]]);assert.equal(matches[0].count,physical.length?index+1:0);
      assert.equal(matches[0].supplemental,undefined);
    }
    const f20=cells.filter(k=>k.id==='f20');assert.equal(f20.length,physical.length?1:0);
    if(f20.length){assert.equal(f20[0].supplemental,true);assert.equal(f20[0].count,8);}
    assert.equal(cells.reduce((n,k)=>n+k.count,0),view.total);
    assert.equal(new Set(cells.map(k=>k.id)).size,cells.length);
  }
});

test('PC top row preserves system-key identities, blank numpad header and platform modifiers',()=>{
  for(const platform of ['windows','linux']){
    const model=K.create(platform),view=model.distribution({physical:[{code:0x10137,count:10},{code:0x10046,count:20},{code:0x10245,count:30}]},'history','full_size');
    assert.deepEqual(view.rows.flat().filter(k=>k.zone==='system').map(k=>[k.label,k.column,k.count]),[['PrtSc',1,10],['ScrLk',2,20],['Pause',3,30]]);
    assert.equal(view.rows.flat().some(k=>k.zone==='numpad-function'),false);
    assert.equal(view.mapped,60);assert.equal(view.other,0);
    for(const side of ['left','right'])assert.equal(model.byId.get('command-'+side).label,platform==='linux'?'Super':'Win');
    assert.equal(model.byId.get('menu').label,'Menu');
  }
});

test('keycap counts use the active locale and reuse numeric formatters',()=>{
  const fs=require('node:fs'),vm=require('node:vm');let constructed=0;
  const context=vm.createContext({module:{exports:{}},Intl:{NumberFormat:function(...args){constructed++;return new Intl.NumberFormat(...args);}}});
  vm.runInContext(fs.readFileSync('ui/keyboard.js','utf8'),context);
  const keyboard=context.module.exports.create('macos');
  assert.equal(keyboard.compact(1234,'de'),new Intl.NumberFormat('de').format(1234));
  const first=constructed;
  for(let i=0;i<1000;i++)keyboard.compact(i,'de');
  assert.equal(constructed,first);
  assert.equal(keyboard.compact(12345,'fr'),'12,3k');
  assert.equal(keyboard.compact(1234,'fr'),new Intl.NumberFormat('fr').format(1234));
  assert.equal(keyboard.compact(1234567,'de'),'1,23M');
  const beforeShort=constructed;
  for(const locale of Object.keys(require('../ui/locales.json'))){
    for(const [value,label] of [[0,'0'],[999,'999'],[1000,'1k'],[10000,'10k'],[280000,'280k'],[999999,'1M']]){
      assert.equal(keyboard.compact(value,locale,true),label,locale);
    }
  }
  assert.equal(constructed,beforeShort);
});
