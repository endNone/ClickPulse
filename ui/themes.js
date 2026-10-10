'use strict';
const ClickPulseThemes=(() => {
  const palettes={
    blue:{accent:'#2563eb',hover:'#1d4ed8',strong:'#2459a2',navy:'#102543',selected:'#1c3c68',navHover:'#193456',background:'#f3f6fc',empty:'#edf4ff',full:'#1f53a0',mouse:'#67c9f4'},
    teal:{accent:'#0f8177',hover:'#096a62',strong:'#17685f',navy:'#102d2e',selected:'#1b5051',navHover:'#173d3e',background:'#f2f8f7',empty:'#eaf7f4',full:'#14665f',mouse:'#64c9bc'},
    violet:{accent:'#7c3aed',hover:'#6d28d9',strong:'#6639a2',navy:'#251e3c',selected:'#493568',navHover:'#352849',background:'#f6f3fc',empty:'#f3edff',full:'#60319c',mouse:'#b49ae8'},
    graphite:{accent:'#52667e',hover:'#3b4e66',strong:'#344b66',navy:'#182330',selected:'#34465b',navHover:'#253446',background:'#f4f6f8',empty:'#edf1f6',full:'#34485f',mouse:'#95aabd'}
  };
  const rgb=hex=>[1,3,5].map(i=>parseInt(hex.slice(i,i+2),16));
  const mix=(a,b,amount)=>'#'+rgb(a).map((value,i)=>Math.round(value+(rgb(b)[i]-value)*amount).toString(16).padStart(2,'0')).join('');
  const get=name=>{const selected=Object.prototype.hasOwnProperty.call(palettes,name)?name:'blue';return {...palettes[selected],name:selected};};
  function tokens(name){
    const p=get(name),soft=mix(p.accent,'#ffffff',.94),muted=mix(p.accent,'#ffffff',.68);
    return {
      '--bg':p.background,'--blue':p.accent,'--cyan':p.mouse,'--navy':p.navy,
      '--accent-hover':p.hover,'--accent-strong':p.strong,'--accent-soft':soft,'--accent-border':mix(p.accent,'#ffffff',.76),
      '--focus':mix(p.accent,'#ffffff',.3),'--control-text':mix(p.strong,'#172b4a',.28),'--control-border':mix(p.accent,'#ffffff',.78),'--control-hover':mix(p.accent,'#ffffff',.95),'--control-hover-border':mix(p.accent,'#ffffff',.55),
      '--nav-selected':p.selected,'--nav-hover':p.navHover,'--nav-text':mix(p.navy,'#ffffff',.75),'--nav-muted':mix(p.navy,'#ffffff',.6),'--nav-icon':mix(p.accent,'#ffffff',.55),'--nav-line':mix(p.navy,'#ffffff',.14),
      '--featured-start':mix(p.strong,p.navy,.4),'--featured-end':p.strong,
      '--key-border':p.accent+'26','--key-bottom':p.accent+'38',
      '--heat-0':p.empty,'--heat-1':mix(p.full,p.empty,.76),'--heat-2':mix(p.full,p.empty,.56),'--heat-3':mix(p.full,p.empty,.3),'--heat-4':p.full,
      '--mini-start':mix(p.navy,p.accent,.12),'--mini-end':p.navy,'--mini-edge':mix(p.navy,p.accent,.35)+'80','--mini-muted':muted,'--mini-hover':mix(p.navy,p.accent,.4)+'70','--mini-selected':p.accent+'24',
      '--mini-status-bg':p.accent+'1c','--mini-dot':mix(p.accent,'#ffffff',.3),'--mini-device-bg':muted+'0c','--mini-device-border':muted+'1c','--mini-device-text':mix(p.accent,'#ffffff',.82),'--mini-device-icon':mix(p.accent,'#ffffff',.6)
    };
  }
  function apply(root,name){
    const p=get(name);root.dataset.theme=p.name;
    for(const[key,value]of Object.entries(tokens(p.name)))root.style.setProperty(key,value);
    return p;
  }
  const luminance=color=>rgb(color).map(c=>{const v=c/255;return v<=.04045?v/12.92:((v+.055)/1.055)**2.4;}).reduce((n,v,i)=>n+v*[.2126,.7152,.0722][i],0);
  const contrast=(a,b)=>(Math.max(luminance(a),luminance(b))+.05)/(Math.min(luminance(a),luminance(b))+.05);
  const textFor=color=>contrast(color,'#ffffff')>=4.5?'#ffffff':contrast(color,'#172b4a')>=4.5?'#172b4a':'#000000';
  return {get,tokens,apply,rgb,contrast,textFor,names:Object.keys(palettes)};
})();
if(typeof module!=='undefined')module.exports=ClickPulseThemes;
