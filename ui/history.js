'use strict';
// Calendar-day arithmetic in UTC avoids skipped or duplicated days at DST changes.
// The day strings themselves have already been assigned in Rust's local timezone.
const ClickPulseHistory = (() => {
  const total = counts => counts.keyboard + counts.mouse;
  const date = day => new Date(day + 'T00:00:00Z');
  const dayString = value => value.toISOString().slice(0, 10);
  const addDays = (day, amount) => {
    const value = date(day);
    value.setUTCDate(value.getUTCDate() + amount);
    return dayString(value);
  };
  const weekday = day => (date(day).getUTCDay() + 6) % 7;
  const difference = (a, b) => Math.round((date(b) - date(a)) / 86400000);

  function bounds(records, today, range) {
    if (range === '365') return { start: addDays(today, -364), end: today };
    if (range === '90') return { start: addDays(today, -89), end: today };
    if (range !== 'all') {
      return { start: range + '-01-01', end: range === today.slice(0, 4) ? today : range + '-12-31' };
    }
    return { start: records.length ? records[0].day : addDays(today, -89), end: today };
  }

  function recordedInRange(records, today, range) {
    const { start, end } = bounds(records, today, range);
    return records.filter(record => record.day >= start && record.day <= end);
  }

  function calendarDays(records, start, end) {
    const byDay = new Map(records.map(record => [record.day, record.counts]));
    const days = [];
    for (let day = start; day <= end; day = addDays(day, 1)) {
      days.push({ day, counts: byDay.get(day) || { keyboard: 0, mouse: 0 } });
    }
    return days;
  }

  function years(records) {
    return [...new Set(records.map(record => record.day.slice(0, 4)))].sort().reverse();
  }

  function sections(days) {
    const result = [];
    for (const day of days) {
      const year = day.day.slice(0, 4);
      if (!result.length || result[result.length - 1].year !== year) result.push({ year, days: [] });
      result[result.length - 1].days.push(day);
    }
    return result;
  }

  // Longer histories use weekly/monthly totals so a complete period remains legible.
  function trend(days) {
    if (days.length <= 90) return days;
    const monthly = days.length > 370;
    const groups = new Map();
    for (const item of days) {
      const period = monthly ? item.day.slice(0, 7) : addDays(item.day, -weekday(item.day));
      if (!groups.has(period)) groups.set(period, {
        day: period, start: item.day, end: item.day,
        grouping: monthly ? 'month' : 'week',
        counts: { keyboard: 0, mouse: 0 }
      });
      const group = groups.get(period);
      group.end = item.day;
      group.counts.keyboard += item.counts.keyboard;
      group.counts.mouse += item.counts.mouse;
    }
    return [...groups.values()];
  }

  function paginate(records, page, size = 30) {
    const rows = records.filter(record => total(record.counts) > 0).slice().reverse();
    const pages = Math.max(1, Math.ceil(rows.length / size));
    const current = Math.max(0, Math.min(page, pages - 1));
    return { rows: rows.slice(current * size, (current + 1) * size), page: current, pages, total: rows.length };
  }

  // Counts use whole, evenly spaced ticks; the stack always fits below the top tick.
  function countScale(maximum) {
    const max = Math.max(0, maximum);
    const magnitude = 10 ** Math.floor(Math.log10(Math.max(1, max / 5)));
    const step = Math.max(1, [1, 2, 5, 10].find(value => value * magnitude >= max / 5) * magnitude);
    const ceiling = Math.max(5, Math.ceil(max / step) * step);
    const ticks = Array.from({length: Math.round(ceiling / step) + 1}, (_, i) => ceiling - i * step);
    return {ceiling, step, ticks};
  }

  function trendLabelIndices(length, limit = 5) {
    const count = Math.min(length, limit);
    return Array.from({length: count}, (_, i) => count === 1 ? 0 : Math.round(i * (length - 1) / (count - 1)));
  }

  // Approximate English-edition lengths, not a measure of written content.
  // References and counting method: docs/literary-references.md.
  const books=Object.freeze([
    {id:'pride',characters:700000},
    {id:'alice',characters:140000},
    {id:'frankenstein',characters:420000},
    {id:'sherlock',characters:560000},
    {id:'two_cities',characters:750000},
    {id:'expectations',characters:990000},
    {id:'jane_eyre',characters:1020000},
    {id:'wuthering',characters:640000},
    {id:'dracula',characters:840000},
    {id:'moby_dick',characters:1220000},
    {id:'treasure',characters:360000},
    {id:'dorian',characters:430000},
    {id:'time_machine',characters:180000},
    {id:'war_worlds',characters:340000},
    {id:'oz',characters:210000},
    {id:'secret_garden',characters:430000},
    {id:'tom_sawyer',characters:390000},
    {id:'huck_finn',characters:570000},
    {id:'eighty_days',characters:370000},
    {id:'monte_cristo',characters:2610000},
  ].map(Object.freeze));

  const phraseCount=5;

  function nextChoice(length, previous, random) {
    const skip=Number.isInteger(previous)&&previous>=0&&previous<length;
    const unit=Number.isFinite(random)?Math.max(0,Math.min(random,1-Number.EPSILON)):0;
    const index=Math.floor(unit*(length-(skip?1:0)));
    return skip&&index>=previous?index+1:index;
  }

  function nextBook(previous=-1, random=Math.random()) {
    return nextChoice(books.length,previous,random);
  }

  function nextPhrase(previous=-1, random=Math.random()) {
    return nextChoice(phraseCount,previous,random);
  }

  function bookComparison(keyboard, index=0) {
    if(!Number.isFinite(keyboard)||keyboard<=0)return null;
    const book=books[index]||books[0],characters=book.characters;
    return {characters,copies:Math.round(keyboard/characters*10)/10,opening:keyboard<characters/10};
  }

  function monthMarkers(days, labelForMonth) {
    if(!days.length)return [];
    const first=days[0].day,padding=weekday(first),markers=[];
    let previous='';
    for(const {day} of days){
      const month=day.slice(5,7);
      if(month===previous)continue;
      previous=month;
      const label=labelForMonth(month),left=Math.floor((difference(first,day)+padding)/7)*15;
      // Reserve enough room for Latin abbreviations and CJK month labels. When
      // the range begins at month-end, omit that fragment's label, not the next month.
      const width=Array.from(label).reduce((n,c)=>n+(c.codePointAt(0)>255?12:7),0)+7;
      if(markers.length && left < markers[markers.length-1].left+markers[markers.length-1].width)markers.pop();
      markers.push({label,left,width});
    }
    return markers;
  }

  return { addDays, weekday, difference, bounds, recordedInRange, calendarDays, monthMarkers, years, sections, trend, paginate, countScale, trendLabelIndices, books, phraseCount, nextBook, nextPhrase, bookComparison };
})();
if (typeof module !== 'undefined') module.exports = ClickPulseHistory;
