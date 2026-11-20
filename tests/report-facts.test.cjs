'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const F = require('../ui/report-facts.js');
const {cardData} = require('../ui/insights.js');
const H = require('../ui/history.js');
const C = require('../ui/shortcuts-catalog.js');
const locales = require('../ui/locales.json');
const i18n = require('../ui/i18n.js');

const day = '2026-11-17';
const previous = '2026-11-16';
const I = i18n(locales);
function makeCard(hours, start = day, now = '2026-11-18T03:00:00Z', ranking = [], systemZone = 'UTC') {
  const period = start === day ? 'daily' : 'weekly';
  const report = {day, start, period, timezone: 'UTC', hours, ranking};
  return cardData({report, now}, day, 'UTC', systemZone, period, 'macos', start);
}
function section(id, card, depth) {
  return F.section(id, card, I, C, H, depth);
}

test('weekly busiest slot reports at most one decimal without rounding positive activity to zero', () => {
  for (const [value, expected] of [[7008 / 7, '1,001.1'], [9320 / 7, '1,331.4'], [7, '7'], [1 / 31, '<0.1']]) {
    assert.equal(F.clockPeakValue({hour: 16, dailyAverage: value}, I), '16:00–17:00 · ' + expected + ' inputs/day');
  }
});

test('weekly activity counts dates separately and compares individual hours with the whole period', () => {
  const card = makeCard([
    [previous, 7, 30, 10], [day, 7, 20, 10], [day, 20, 10, 20],
    [previous, 11, 0, 0], ['2026-11-12', 9, 800, 0],
  ], previous);
  assert.equal(card.activeHours, 3);
  const rhythm = section('rhythm', card).lines;
  assert(rhythm.includes(I.t('report_fact.active', {count: '3'})));
  assert(rhythm.includes(I.t('report_fact.periods', {bands: '06:00–12:00 70.0% · 18:00–24:00 30.0%'})));
  assert(rhythm.includes('Busiest time slot · 07:00–08:00 · 35 inputs/day'));
  const peak = section('peak', card).lines;
  assert(peak.includes(I.t('report_fact.peak_share', {share: '40.0%'})));
  assert(peak.some(line=>line.includes(previous)&&line.includes('07:00–08:00')&&line.endsWith(I.t('share.shortcut_count', {count: '40'}))));
  assert(!peak.join('\n').includes(I.t('report.clock_peak')));
});

test('partial reports distinguish a future time band from an observed zero, including in a multi-day range', () => {
  const now = day + 'T07:30:00Z';
  const samples = [[previous, 22, 4, 1], [day, 6, 4, 1], [day, 7, 1, 0], [day, 18, 50000, 100]];
  const daily = section('rhythm', makeCard(samples, day, now)).lines;
  assert(daily.includes(I.t('report_fact.periods', {bands: '06:00–12:00 100.0%'})));
  const weekly = makeCard(samples, previous, now);
  assert(section('rhythm', weekly).lines.includes(I.t('report_fact.periods', {bands: '06:00–12:00 54.5% · 18:00–24:00 45.5%'})));
  const peak = section('peak', weekly).lines;
  assert(peak.includes(I.t('report_fact.peak_tied', {share: '45.5%', count: '2'})));
  assert(peak.some(line=>line.includes(I.t('report.peak_tied'))&&line.includes(previous)&&line.includes('22:00–23:00')));
});

test('book analogy uses only keys, skips negligible counts, and preserves tiny nonzero input shares', () => {
  const sample = makeCard([[day, 1, 0, 1], [day, 22, 70000, 40]]);
  assert.deepEqual(section('book', sample).lines, [
    I.t('report_fact.book_part', {book: locales.en['calendar.book_alice'], value: '50.0%'}),
  ]);
  assert(section('rhythm', sample).lines.includes(I.t('report_fact.periods', {bands: '00:00–06:00 <0.1% · 18:00–24:00 >99.9%'})));
  for (const keys of [0, 10]) {
    const mouseHeavy = makeCard([[day, 1, keys, 900000]]);
    assert(!F.available(mouseHeavy, H).includes('book'));
    assert.deepEqual(section('book', mouseHeavy).lines, []);
  }
  assert.deepEqual(section('book', makeCard([[day, 1, 700000, 1]])).lines,
    [I.t('report_fact.book_multiple', {book: locales.en['calendar.book_pride'], value: '1'})]);
});

test('available report highlights respect the data, including empty, mouse-only and timezone-mismatched periods', () => {
  const combo = {platform: 'macos', modifiers: 8, key: 8, count: 1};
  for (const card of [makeCard([]), {loaded: false}]) {
    assert.deepEqual(F.available(card, H), []);
    assert.deepEqual(section('peak', card), {label: '', lines: []});
  }
  const mouse = makeCard([[day, 23, 0, 44]], day, undefined, [combo]);
  assert.deepEqual(F.available(mouse, H), ['mix', 'peak']);
  assert.deepEqual(section('mix', mouse).lines, [I.t('report_prompt.mix', {keyboard: '0.0%', mouse: '100.0%'})]);
  const previousHour = section('peak', mouse).lines.join('\n');
  assert(previousHour.includes(day) && previousHour.includes('23:00–24:00'));
  const keyboard = makeCard([[day, 1, 1, 0]], day, undefined, [combo]);
  assert.deepEqual(F.available(keyboard, H), ['shortcut', 'mix', 'peak']);
  assert.deepEqual(section('ranking', keyboard), {label: '', lines: []});
  const whole = [[day, 1, 170, 0], [day, 9, 4, 2]];
  const foreign = makeCard(whole, day, undefined, [combo], 'Asia/Shanghai');
  assert.deepEqual(F.available(foreign, H), ['mix', 'peak', 'rhythm', 'book']);
  assert.deepEqual(section('shortcut', foreign).lines, []);
  assert.deepEqual(section('mix', makeCard([[day, 1, 1, 15]])).lines,
    [I.t('report_prompt.mix', {keyboard: '6.3%', mouse: '93.7%'})]);
});

test('one shortcut and a bounded ranking contain exactly the requested rows, respecting equal counts', () => {
  const ranking = [[8, 3], [9, 2], [6, 2], [0, 1]].map(([key, count]) =>
    ({platform: 'macos', modifiers: 8, key, count}));
  const hours = [[day, 0, 10, 4], [day, 3, 9900, 26]];
  const card = makeCard(hours, day, undefined, ranking);
  assert.deepEqual(F.available(card, H), ['shortcut', 'ranking', 'mix', 'peak', 'rhythm', 'book']);
  const leader = section('shortcut', card, 3);
  assert.deepEqual(leader.lines, [
    F.shortcutLabel(ranking[0], I, C) + ' · ' + I.t('share.shortcut_count', {count: '3'}),
  ]);
  for (const depth of [2, 3]) {
    const result = section('ranking', card, depth);
    assert.equal(result.label, I.t('report_focus.ranking', {count: String(depth)}));
    assert.equal(result.lines.length, depth);
    assert(!result.lines.join('\n').includes('⌘ + A'));
    assert(result.lines.at(-1).startsWith('=2. '));
    assert(!result.lines.join('\n').includes('⌘ + Z') || depth === 3);
  }
  const two = makeCard(hours, day, undefined, ranking.slice(0, 2));
  const result = section('ranking', two, 3);
  assert.equal(result.label, I.t('report_focus.ranking', {count: '2'}));
  assert.equal(result.lines.length, 2);
  assert.equal(section('mix', card).lines.length, 1);
  for (const id of ['mix', 'peak', 'rhythm', 'book']) {
    assert(!section(id, card).lines.join('\n').includes('⌘ + C'));
  }
});

test('unique references annotate the literal chord with a translated title or the original English', () => {
  const chord = key => ({platform: 'macos', modifiers: 8, key});
  for (const locale of Object.keys(locales)) {
    const I = i18n(locales, locale);
    const label = F.shortcutLabel(chord(8), I, C);
    const copy = C.matches(chord(8))[0];
    assert.equal(label, I.t('report_fact.shortcut', {
      chord: '⌘ + C', scope: C.scope(copy, locale), meaning: copy.title[locale === 'zh-CN' ? 'zh' : locale] || I.t('report_fact.action_copy'),
    }));
    for (const key of [34, 36, 78]) {
      assert.equal(F.shortcutLabel(chord(key), I, C), C.chordParts(chord(key)).join(' + '));
    }
  }
  const japanese = i18n(locales, 'ja');
  assert.equal(F.shortcutLabel(chord(12), japanese, C), japanese.t('report_fact.shortcut', {
    chord: '⌘ + Q', meaning: 'Quit app', scope: 'macOS',
  }));
});

test('all eight translations preserve each report fact and its parameters', () => {
  const keys = table => Object.keys(table).filter(key => key.startsWith('report_fact.')).sort();
  const params = text => (text.match(/\{[a-z_]+\}/g) || []).sort();
  const expected = keys(locales.en);
  assert.ok(expected.length > 0);
  for (const [locale, table] of Object.entries(locales)) {
    assert.deepEqual(keys(table), expected, locale);
    for (const key of expected) {
      assert.ok(table[key].trim());
      assert.deepEqual(params(table[key]), params(locales.en[key]), locale + ' ' + key);
    }
  }
});
