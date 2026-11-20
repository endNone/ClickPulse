'use strict';
// Squared rows keep blocks readable while preserving each positive count's
// exact share of the available area. Zero-count keys have no invented area.
function keyTreemap(keys,width,height) {
  if(!Number.isFinite(width)||!Number.isFinite(height)||width<=0||height<=0)return [];
  const ordered=keys.filter(key=>Number.isFinite(key.count)&&key.count>0).slice().sort((a,b)=>b.count-a.count||a.id.localeCompare(b.id));
  const total=ordered.reduce((sum,key)=>sum+key.count,0);
  if(!total)return [];
  const remaining=ordered.map(key=>({key,area:key.count/total*width*height})),result=[];
  let x=0,y=0,w=width,h=height;
  const worst=(row,side)=>{
    if(!row.length)return Infinity;
    const sum=row.reduce((sum,item)=>sum+item.area,0),areas=row.map(item=>item.area);
    return Math.max(side*side*Math.max(...areas)/(sum*sum),sum*sum/(side*side*Math.min(...areas)));
  };
  while(remaining.length){
    const side=Math.min(w,h),row=[remaining.shift()];
    while(remaining.length&&worst([...row,remaining[0]],side)<=worst(row,side))row.push(remaining.shift());
    const area=row.reduce((sum,item)=>sum+item.area,0),vertical=w>=h;
    const thickness=remaining.length?area/side:vertical?w:h;
    let offset=0;
    row.forEach((item,index)=>{
      const length=index===row.length-1?side-offset:item.area/thickness;
      result.push({...item.key,x:vertical?x:x+offset,y:vertical?y+offset:y,w:vertical?thickness:length,h:vertical?length:thickness});
      offset+=length;
    });
    if(vertical){x+=thickness;w-=thickness;}else{y+=thickness;h-=thickness;}
  }
  return result;
}
function createClickPulseKeyboard(platform='macos') {
  const key = (id, label, vk, chars = '', width = 1, legacy = []) => ({ id, label, vk: [].concat(vk), chars: [...chars].map(c => c.codePointAt(0)), width, legacy });
  const letters = (text, codes) => [...text].map((label, i) => key(label, label, codes[i], label + label.toLowerCase()));
  const rows = [
    [key('esc', 'Esc', 53, '', 1.2, [53]), ...[122,120,99,118,96,97,98,100,101,109,103,111].map((vk, i) => key('f' + (i+1), 'F' + (i+1), vk))],
    [key('backtick','`',50,'`~'), ...[18,19,20,21,23,22,26,28,25,29].map((vk,i) => key('digit'+i, '1234567890'[i], vk, '1234567890'[i] + '!@#$%^&*()'[i])), key('minus','−',27,'-_'),key('equal','=',24,'=+'),key('backspace','⌫',51,'',1.65,[51])],
    [key('tab','Tab',48,'',1.5,[48]), ...letters('QWERTYUIOP',[12,13,14,15,17,16,32,34,31,35]),key('leftbracket','[',33,'[{【「'),key('rightbracket',']',30,']}】」'),key('backslash','\\',42,'\\|',1.2)],
    [key('caps','Caps',57,'',1.8,[57]), ...letters('ASDFGHJKL',[0,1,2,3,5,4,38,40,37]),key('semicolon',';',41,';:'),key('quote',"'",39,"'\""),key('return','Return',36,'',2.05,[36])],
    [key('shift-left','Shift',56,'',2.2,[56]), ...letters('ZXCVBNM',[6,7,8,9,11,45,46]),key('comma',',',43,',<《'),key('period','.',47,'.>。》'),key('slash','/',44,'/?'),key('shift-right','Shift',60,'',2.3,[60])],
    [key('fn','Fn',63,'',1,[63]),key('control','Ctrl',[59,62],'',1.05,[59,62]),key('option-left','⌥',58,'',1.1,[58]),key('command-left','⌘',55,'',1.3,[55]),key('space','Space',49,'  ',5,[49,32]),key('command-right','⌘',54,'',1.3,[54]),key('option-right','⌥',61,'',1.1,[61]),key('left','←',123,'',1,[123]),key('down','↓',125,'',1,[125]),key('up','↑',126,'',1,[126]),key('right','→',124,'',1,[124])]
  ];
  const windowsSupplemental=[...[105,107,113,106,64,79,80,90].map((vk,i)=>key('f'+(i+13),'F'+(i+13),vk)),key('numequals','Num =',81),key('numcomma','Num ,',95),key('iso-section','§',10,'§±'),key('volume-up','Vol +',72),key('volume-down','Vol −',73),key('mute','Mute',74)];
  if(platform==='windows'||platform==='linux') {
    for(const row of rows)for(const k of row){if(k.id.startsWith('command-'))k.label='Win';if(k.id.startsWith('option-'))k.label='Alt';if(k.id==='return')k.label='Enter';}
    rows[0].push(key('print-screen','PrtSc',[0x10137,0x10054]),key('scroll-lock','ScrLk',0x10046),key('pause-break','Pause',[0x10245,0x10146]));
    rows[5]=[key('control-left','Ctrl',59,'',1.3),key('command-left','Win',55,'',1.1),key('option-left','Alt',58,'',1.1),key('space','Space',49,'  ',5,[49,32]),key('option-right','Alt',61,'',1.1),key('command-right','Win',54,'',1.1),key('menu','Menu',63,'',1.1),key('control-right','Ctrl',62,'',1.3),key('left','←',123),key('down','↓',125),key('up','↑',126),key('right','→',124)];
    windowsSupplemental.push(key('insert','Ins',114),key('home','Home',115),key('pageup','PgUp',116),key('delete','Del',117),key('end','End',119),key('pagedown','PgDn',121),
      key('numlock','Num',71),key('numdivide','Num ÷',75),key('nummultiply','Num ×',67),key('numminus','Num −',78),key('numplus','Num +',69),
      ...[82,83,84,85,86,87,88,89,91,92].map((vk,i)=>key('num'+i,'Num '+i,vk)),key('numdecimal','Num .',65),key('numenter','Num Enter',76));
  }
  const supplemental = platform==='macos' ? [
    key('help','Help',114),key('home','Home',115),key('pageup','PgUp',116),
    key('delete','Del',117),key('end','End',119),key('pagedown','PgDn',121),
    ...[105,107,113,106,64,79,80,90].map((vk,i)=>key('f'+(i+13),'F'+(i+13),vk)),
    key('numclear','Clear',71),key('numdivide','Num ÷',75),key('nummultiply','Num ×',67),
    key('numminus','Num −',78),key('numplus','Num +',69),key('numequals','Num =',81),
    ...[82,83,84,85,86,87,88,89,91,92].map((vk,i)=>key('num'+i,'Num '+i,vk)),
    key('numdecimal','Num .',65),key('numenter','Num ↵',76),key('numcomma','Num ,',95),
    key('volume-up','Vol +',72),key('volume-down','Vol −',73),key('mute','Mute',74),
    key('menu','Menu',110),key('iso-section','§',10,'§±'),key('jis-yen','¥',93),
    key('jis-underscore','JIS _',94),key('jis-eisu','Eisu',102),key('jis-kana','Kana',104)
  ] : windowsSupplemental;
  if(platform==='linux')for(const row of rows)for(const k of row)if(k.id.startsWith('command-'))k.label='Super';
  const all = [...rows.flat(), ...supplemental];
  // Both keyboard views describe the same visible keys. Keep supplemental
  // history in ranked/totals without including it in this display scope.
  const compactIds = new Set(rows.flat().map(k => k.id));
  const physicalAliases = k=>k.vk.flatMap(vk=>[65536+vk,0x110000+vk,2**40+vk]);
  // Full-width ASCII punctuation/digits describe the same printed key. Do not
  // normalize arbitrary code values: some ranges are old native/hash encodings.
  const characterAliases = k=>k.chars.flatMap(code=>code>=33&&code<=126?[code,code+0xfee0]:[code]);
  const byId = new Map(all.map(k => [k.id, k]));
  const shortCount = new Intl.NumberFormat('en-US',{notation:'compact',maximumFractionDigits:0});
  let numberLocale,formats;
  function compact(value,locale='en',short=false){
    if(short)return shortCount.format(value).replace('K','k');
    if(numberLocale!==locale){
      numberLocale=locale;
      formats={plain:new Intl.NumberFormat(locale),
        thousands:new Intl.NumberFormat(locale,{minimumFractionDigits:1,maximumFractionDigits:1}),
        millions:new Intl.NumberFormat(locale,{minimumFractionDigits:2,maximumFractionDigits:2})};
    }
    return value>=1000000?formats.millions.format(value/1000000)+'M':value>=10000?formats.thousands.format(value/1000)+'k':formats.plain.format(value);
  }

  function distribution(stats = {}, scope = 'history', layout = 'compact') {
    const totals = new Map(all.map(k => [k.id, 0]));
    let total = 0, mapped = 0;
    const allocate = (records, passes) => {
      const remaining = new Map();
      for (const row of records || []) {
        const count = Math.max(0, Number(row.count) || 0);
        remaining.set(Number(row.code), (remaining.get(Number(row.code)) || 0) + count);
        total += count;
      }
      for (const aliases of passes) {
        for (const k of all) {
          for (const code of new Set(aliases(k))) {
            const count = remaining.get(code) || 0;
            totals.set(k.id, totals.get(k.id) + count);
            mapped += count;
            remaining.delete(code);
          }
        }
      }
    };
    const historical = scope === 'history' && stats.history_available;
    if (historical) {
      // Old vk values collided with character codes. Reserve the known special
      // aliases first, then attribute remaining printable characters once.
      allocate(stats.history_before_fix, [physicalAliases, k=>k.legacy, characterAliases]);
      allocate(stats.history_since_fix, [physicalAliases, characterAliases]);
    } else {
      allocate(stats.physical, [k => k.vk]);
    }
    const ranked = all.map(k => ({ ...k, count: totals.get(k.id) })).filter(k => k.count > 0).sort((a,b) => b.count-a.count || a.id.localeCompare(b.id));
    const arrowIds=['up','left','down','right'];
    const fullSize=layout==='full_size';
    const visibleIds=new Set(compactIds);
    const estimated=historical && [...(stats.history_before_fix||[]),...(stats.history_since_fix||[])].some(row=>Number(row.count)>0&&Number(row.code)<2**40);
    const systemIds=platform==='macos'?['f13','f14','f15']:['print-screen','scroll-lock','pause-break'];
    const navigationKey=platform==='macos'?'fn':'insert';
    const clusterKeys=new Set([...arrowIds,...systemIds,navigationKey]);
    const displayRows=rows.map(row=>row.filter(k=>!fullSize||!clusterKeys.has(k.id)).map(k=>({...k,count:totals.get(k.id)})));
    if(fullSize){
      const place=(id,zone,column,row,colSpan=1,rowSpan=1)=>{
        const k=byId.get(id);if(!k)return null;
        visibleIds.add(id);
        return {...k,count:totals.get(id),zone,column,row,colSpan,rowSpan};
      };
      const system=systemIds.map((id,i)=>place(id,'system',i+1,1)).filter(Boolean);
      if(system.length)displayRows.push(system);
      if(platform==='macos')displayRows.push(['f16','f17','f18','f19'].map((id,i)=>place(id,'numpad-function',i+1,1)));
      const nav=[navigationKey,'home','pageup','delete','end','pagedown'].map((id,i)=>place(id,'navigation',i%3+1,Math.floor(i/3)+1));
      if(platform==='macos')nav[0].label='fn';
      nav.push(place('up','navigation',2,4),place('left','navigation',1,5),place('down','navigation',2,5),place('right','navigation',3,5));
      displayRows.push(nav.filter(Boolean));
      const pad=[
        ...(platform==='macos'?[
          ['numclear',1,1],['numequals',2,1],['numdivide',3,1],['nummultiply',4,1],['numminus',4,2],['numplus',4,3]
        ]:[
          ['numlock',1,1],['numdivide',2,1],['nummultiply',3,1],['numminus',4,1],['numplus',4,2,1,2]
        ]),
        ['num7',1,2],['num8',2,2],['num9',3,2],
        ['num4',1,3],['num5',2,3],['num6',3,3],['num1',1,4],['num2',2,4],['num3',3,4],['numenter',4,4,1,2],['num0',1,5,2],['numdecimal',3,5]
      ].map(([id,...position])=>place(id,'numpad',...position)).filter(Boolean);
      displayRows.push(pad);
    }
    const extra=supplemental.filter(k=>!visibleIds.has(k.id)&&totals.get(k.id)>0).map(k=>({...k,count:totals.get(k.id),supplemental:true}));
    for(const k of extra)visibleIds.add(k.id);
    const visibleRanked=ranked.filter(k=>visibleIds.has(k.id));
    const other=total-mapped;
    // Keep unresolved historical counts in the total without inventing a key
    // or changing the physical keyboard layout. Original records stay intact.
    if(extra.length)displayRows.push(extra);
    return {estimated,fullSize,rows:displayRows,ranked,visibleRanked,total,mapped,unmapped:other,other,historical};
  }
  return { distribution, compact, byId, treemap:keyTreemap };
}
const ClickPulseKeyboard=createClickPulseKeyboard(typeof window==='undefined'?'macos':window.clickpulsePlatform);
if (typeof module !== 'undefined') module.exports = {...ClickPulseKeyboard,create:createClickPulseKeyboard};
