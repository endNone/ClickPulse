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

  // A playful keystroke-length comparison, never a measure of typed text.
  // Pride and Prejudice's English novel body is rounded to700,000 characters;
  // source and counting method are documented in docs/data-and-privacy.md.
  function bookComparison(keyboard) {
    if(!Number.isFinite(keyboard)||keyboard<=0)return null;
    const characters=700000;
    return {characters,copies:Math.round(keyboard/characters*10)/10,opening:keyboard<characters/10};
  }

  return { addDays, weekday, difference, bounds, recordedInRange, calendarDays, years, sections, trend, paginate, countScale, trendLabelIndices, bookComparison };
})();
if (typeof module !== 'undefined') module.exports = ClickPulseHistory;
