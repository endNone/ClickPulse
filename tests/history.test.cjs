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
