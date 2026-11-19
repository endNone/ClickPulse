const test = require('node:test');
const assert = require('node:assert/strict');
const H = require('../ui/history.js');
const records = [
  {day:'2026-02-05',counts:{keyboard:100,mouse:20}},
  {day:'2026-06-03',counts:{keyboard:30,mouse:10}},
  {day:'2026-10-07',counts:{keyboard:55,mouse:9}}
];
const count = days => days.reduce((sum,d)=>sum+d.counts.keyboard+d.counts.mouse,0);

test('literary comparison is stable, approximate and never formats an empty or tiny count as zero books',()=>{
  for(const value of [0,-1,NaN,Infinity])assert.equal(H.bookComparison(value),null);
  assert.equal(H.bookComparison(1).opening,true);
  assert.equal(H.bookComparison(69999).opening,true);
  assert.deepEqual(H.bookComparison(70000),{characters:700000,copies:.1,opening:false});
  assert.equal(H.bookComparison(700000).copies,1);
  assert.equal(H.bookComparison(1995607).copies,2.9);
  assert.equal(H.bookComparison(7000000).copies,10);
});

test('twenty books use individual lengths and random selection never immediately repeats',()=>{
  assert.equal(H.books.length,20);
  assert.equal(new Set(H.books.map(book=>book.id)).size,20);
  const seen=new Set();
  H.books.forEach((book,index)=>{
    assert.ok(book.characters>=100000);
    assert.equal(H.bookComparison(book.characters,index).copies,1);
    assert.equal(H.bookComparison(book.characters*2.5,index).copies,2.5);
    assert.equal(H.bookComparison(1,index).opening,true);
    seen.add(H.nextBook(-1,(index+.5)/20));
    const next=new Set(Array.from({length:19},(_,i)=>H.nextBook(index,(i+.5)/19)));
    assert.equal(next.size,19);assert.equal(next.has(index),false);
  });
  assert.equal(seen.size,20);
  for(const value of [0,1,NaN,Infinity,-1])assert.ok(H.nextBook(19,value)>=0&&H.nextBook(19,value)<19);
});

test('phrases are independent of books and do not immediately repeat',()=>{
  assert.equal(H.phraseCount,5);
  const combinations=new Set();
  for(let b=0;b<H.books.length;b++)for(let p=0;p<H.phraseCount;p++)
    combinations.add(H.nextBook(-1,(b+.5)/20)+':'+H.nextPhrase(-1,(p+.5)/5));
  assert.equal(combinations.size,100);
  for(let p=0;p<5;p++){
    const next=new Set(Array.from({length:4},(_,i)=>H.nextPhrase(p,(i+.5)/4)));
    assert.equal(next.size,4);assert.equal(next.has(p),false);
  }
});

test('new capture never hides imported history from all-history calendar',()=>{
  const bounds=H.bounds(records,'2026-10-07','all');
  assert.equal(bounds.start,'2026-02-05');
  const days=H.calendarDays(records,bounds.start,bounds.end);
  assert.equal(count(days),224);
  assert.equal(days.filter(d=>count([d])>0).length,3);
  assert.equal(days.at(-1).day,'2026-10-07');
  assert.equal(count(H.trend(days)),224);
});

test('recent range is optional and labeled separately from full history',()=>{
  const recent=H.recordedInRange(records,'2026-10-07','90');
  assert.deepEqual(recent.map(d=>d.day),['2026-10-07']);
  assert.equal(H.bounds(records,'2026-10-07','90').start,'2026-07-10');
});

test('past year always contains 365 calendar dates and pads time before recording with zeroes',()=>{
  for(const today of ['2026-11-16','2026-01-01','2024-02-29','2024-03-01']){
    const bounds=H.bounds([],today,'365');
    const records=[{day:H.addDays(bounds.start,-1),counts:{keyboard:900,mouse:900}},
      {day:today,counts:{keyboard:20,mouse:10}},
      {day:H.addDays(today,1),counts:{keyboard:800,mouse:800}}];
    const days=H.calendarDays(records,bounds.start,bounds.end);
    assert.equal(days.length,365);assert.equal(days.at(-1).day,today);
    assert.ok(days.slice(0,-1).every(day=>count([day])===0));assert.equal(count(days),30);
    assert.equal(H.calendarDays([],bounds.start,bounds.end).length,365);
    if(today==='2024-03-01')assert.ok(days.some(day=>day.day==='2024-02-29'));
  }
});

test('year partitions, leap dates, and zero-activity gaps are retained',()=>{
  const days=H.calendarDays([], '2024-02-28','2024-03-02');
  assert.deepEqual(days.map(d=>d.day),['2024-02-28','2024-02-29','2024-03-01','2024-03-02']);
  assert.equal(H.calendarDays([], '2026-03-07','2026-03-09').length,3);
  const crossing=H.calendarDays([], '2025-12-31','2026-01-02');
  assert.deepEqual(H.sections(crossing).map(s=>s.year),['2025','2026']);
  assert.deepEqual(H.years(records),['2026']);
});

test('year selection and pagination expose every recorded day without losing totals',()=>{
  const all=[...records,{day:'2027-01-01',counts:{keyboard:1,mouse:1}}];
  assert.equal(count(H.recordedInRange(all,'2027-01-02','2026')),224);
  const a=H.paginate(records,0,2), b=H.paginate(records,1,2);
  assert.equal(a.total,3);assert.equal(a.pages,2);
  assert.deepEqual([...a.rows,...b.rows].map(d=>d.day),records.slice().reverse().map(d=>d.day));
  assert.equal(H.paginate([],10).page,0);
});

test('count axes use readable whole ticks without clipping or duplicate labels',()=>{
  assert.deepEqual(H.countScale(22100),{ceiling:25000,step:5000,ticks:[25000,20000,15000,10000,5000,0]});
  for(const maximum of [0,1,3,6,11,49,99,100,101,999,1000,1001,9999,10000,10001,25000,25001,1234567,9999999]){
    const scale=H.countScale(maximum);
    assert.ok(scale.ceiling>=maximum);
    assert.ok(scale.ticks.length>=2&&scale.ticks.length<=6);
    assert.equal(scale.ticks.at(-1),0);
    assert.equal(new Set(scale.ticks).size,scale.ticks.length);
    assert.ok(scale.ticks.every(Number.isInteger));
    assert.ok(scale.ticks.slice(1).every((value,i)=>scale.ticks[i]-value===scale.step));
  }
});

test('date ticks are evenly sampled, ordered and include the actual first and last periods',()=>{
  assert.deepEqual(H.trendLabelIndices(0),[]);
  assert.deepEqual(H.trendLabelIndices(1),[0]);
  for(const length of [2,4,5,14,90,371]){
    const indices=H.trendLabelIndices(length);
    assert.equal(indices[0],0);assert.equal(indices.at(-1),length-1);
    assert.equal(indices.length,Math.min(5,length));
    assert.ok(indices.slice(1).every((value,i)=>value>indices[i]));
  }
  const weeks=H.trend(H.calendarDays(records,'2026-02-05','2026-06-01'));
  assert.equal(weeks[0].day,'2026-02-02');assert.equal(weeks[0].start,'2026-02-05');
  assert.equal(weeks[0].grouping,'week');assert.equal(count(weeks),120);
  const months=H.trend(H.calendarDays(records,'2025-12-01','2027-01-01'));
  assert.equal(months[0].grouping,'month');assert.equal(months[0].day,'2025-12');
  assert.equal(count(months),224);
});

test('calendar month labels do not overlap for ranges starting at month-end',()=>{
 const H=require('../ui/history.js');
 for(const start of ['2026-10-29','2026-05-28'])for(const labels of [['Oct','Nov','Dec'],['10月','11月','12月']]){
   const days=Array.from({length:90},(_,i)=>({day:H.addDays(start,i)}));
   const markers=H.monthMarkers(days,m=>labels[Number(m)%3]);
   for(let i=1;i<markers.length;i++)assert(markers[i].left>=markers[i-1].left+markers[i-1].width);
   assert(markers.length>=2);
 }
});

test('hourly scatter keeps dates distinct and filters weekdays, devices and zero counts', () => {
  const {group, demoHours} = require('../ui/history-scatter.js');
  const rows = [['2026-11-07',9,10,0], ['2026-11-08',9,0,11], ['2026-11-14',9,7,2]];
  const saturday = group(rows,5,[true,true],H);
  assert.deepEqual(saturday.buckets[9][0].map(p=>p.count),[7,10]);
  assert.deepEqual(saturday.buckets[9][1].map(p=>p.day),['2026-11-14']);
  assert.equal(saturday.maximum,10);
  assert.equal(group(rows,-1,[false,true],H).maximum,11);
  assert.equal(group(rows,0,[true,true],H).maximum,0);
  const simulated = demoHours([{day:'2026-11-07',counts:{keyboard:1837,mouse:419}}]);
  assert.equal(simulated.reduce((sum,row)=>sum+row[2],0),1837);
  assert.equal(simulated.reduce((sum,row)=>sum+row[3],0),419);
});
