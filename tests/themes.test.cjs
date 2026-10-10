const test=require('node:test');
const assert=require('node:assert/strict');
const T=require('../ui/themes.js');

test('all four themes change full, heatmap and mini colors without introducing layout tokens',()=>{
  const colors=[];
  for(const name of T.names){
    const values={},root={dataset:{},style:{setProperty(key,value){values[key]=value;}}};
    T.apply(root,name);assert.equal(root.dataset.theme,name);
    for(const key of ['--bg','--navy','--blue','--heat-4','--mini-end'])assert.equal(typeof values[key],'string');
    assert.equal(Object.keys(values).some(key=>/width|height|font|gap/.test(key)),false);
    colors.push(values['--blue']);
  }
  assert.equal(new Set(colors).size,4);assert.equal(T.get('unsupported').name,'blue');
  assert.equal(T.get('constructor').accent,T.get('blue').accent);
  assert.equal(T.get('__proto__').name,'blue');
});
test('theme controls and mini labels retain readable foreground/background pairs',()=>{
  for(const name of T.names){
    const p=T.get(name),tokens=T.tokens(name);
    assert.ok(T.contrast(p.accent,'#ffffff')>=4.5,name+' primary');
    assert.ok(T.contrast(p.strong,'#ffffff')>=4.5,name+' mini entry');
    assert.ok(T.contrast(p.selected,'#ffffff')>=4.5,name+' mini entry hover');
    assert.ok(T.contrast(tokens['--control-text'],'#ffffff')>=4.5,name+' control');
    assert.ok(T.contrast(tokens['--nav-text'],p.navy)>=4.5,name+' sidebar');
    assert.ok(T.contrast(tokens['--mini-device-text'],tokens['--mini-end'])>=4.5,name+' mini');
  }
});
test('heatmap text remains readable at every intensity in every theme',()=>{
  for(const name of T.names){
    const palette=T.get(name),from=T.rgb(palette.empty),to=T.rgb(palette.full);
    for(let step=0;step<=100;step++){
      const color='#'+from.map((v,i)=>Math.round(v+(to[i]-v)*step/100).toString(16).padStart(2,'0')).join('');
      assert.ok(T.contrast(color,T.textFor(color))>=4.5,`${name}/${step}`);
    }
  }
});
