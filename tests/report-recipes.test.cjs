'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const {registry, createPicker, describe, isCompatible} = require('../ui/report-recipes.js');
const languages = ['en', 'zh-CN', 'zh-TW', 'ja', 'ko', 'fr', 'de', 'es'];
const dimensions = Object.keys(registry);
const daily = {kind: 'day', total: 86, sparse: false};
const weekly = {kind: 'week', total: 159, sparse: false};

function randomFrom(seed) {
  return () => {
    seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
    return seed / 4294967296;
  };
}

test('report recipe library covers each artistic choice in all eight languages', () => {
  assert.deepEqual(Object.values(registry).map(section => Object.keys(section).length), [21, 7, 8, 5]);
  for (const dimension of dimensions) {
    const labelSets = Object.fromEntries(languages.map(lang => [lang, new Set()]));
    for (const [id, entry] of Object.entries(registry[dimension])) {
      const words = dimension === 'medium' ? entry.words : entry;
      assert.deepEqual(Object.keys(words), languages);
      for (const lang of languages) {
        const {label, instruction} = words[lang];
        assert.ok(label.trim(), [dimension, id, lang].join(' '));
        assert.ok(instruction.length > label.length, [dimension, id, lang].join(' '));
        assert.ok(!labelSets[lang].has(label), 'duplicate picker label ' + label);
        labelSets[lang].add(label);
        if (dimension === 'palette') assert.equal(instruction.match(/#[0-9A-F]{6}/g).length, 4);
      }
    }
  }
});

test('the density limit belongs to the medium and survives changes to charts, colors or layout', () => {
  const budgets = new Set(Object.values(registry.medium).map(medium => medium.budget));
  assert.deepEqual(budgets, new Set([2, 3, 4]));
  assert.equal(registry.medium.album.budget, 2);
  assert.equal(registry.medium.transit.budget, 3);
  assert.equal(registry.medium.newsprint.budget, 4);

  const picker = createPicker(randomFrom(4));
  for (let i = 0; i < Object.keys(registry.medium).length; i++) {
    const {medium} = picker.next(daily);
    const budget = registry.medium[medium].budget;
    for (const dimension of ['chart', 'palette', 'layout']) {
      assert.equal(registry.medium[picker.next(daily, dimension).medium].budget, budget);
    }
    assert.equal(registry.medium[picker.adapt({...daily, sparse:true}).medium].budget, budget);
  }
});

test('the record sleeve describes time labels without requesting a track list', () => {
  const trackLists = /track ?list|曲目[表單单]|曲目[一表]|トラックリスト|트랙 목록|liste des pistes|Titelliste|lista de pistas/i;
  for (const {instruction} of Object.values(registry.medium.album.words)) {
    assert.doesNotMatch(instruction, trackLists);
  }
});

test('every registered chart, palette and composition is used by a medium', () => {
  for (const [dim, prop] of [['chart','charts'], ['palette','palettes'], ['layout','layouts']]) {
    const used = new Set();
    for (const m of Object.values(registry.medium)) {
      assert.equal(new Set(m[prop]).size, m[prop].length);
      m[prop].forEach(id => used.add(id));
    }
    assert.deepEqual(used, new Set(Object.keys(registry[dim])));
  }
});

test('a full reroll uses every medium once per cycle and changes the three other axes when possible', () => {
  for (let seed = 0; seed < 10; seed++) {
    for (const context of [daily, weekly, {...weekly,sparse:true}]) {
      const picker = createPicker(randomFrom(seed));
      let last;
      for (let cycle = 0; cycle < 3; cycle++) {
        const seen = new Set();
        for (let i = 0; i < 21; i++) {
          const recipe = picker.next(context);
          assert.ok(isCompatible(recipe, context));
          assert.ok(Object.isFrozen(recipe));
          assert.ok(!seen.has(recipe.medium), recipe.medium);
          seen.add(recipe.medium);
          if (last) {
            for (const axis of dimensions.filter(dim => !context.sparse || dim !== 'chart')) {
              assert.notEqual(recipe[axis], last[axis], axis);
            }
          }
          last = recipe;
        }
        assert.equal(seen.size, 21);
      }
    }
  }
});

test('an unfinished day has a chart when it has observations and a clock is reserved for one day', () => {
  const recipe = {medium:'blueprint', chart:'clock', palette:'indigo', layout:'split'};
  assert.equal(isCompatible(recipe, {...daily,partial:true}), true);
  assert.equal(isCompatible(recipe, weekly), false);
  assert.equal(isCompatible(recipe, {...weekly,partial:true}), false);
  assert.equal(isCompatible(recipe, {kind:'range',sparse:false}), false);
  assert.equal(isCompatible(recipe, {...daily,sparse:true}), false);
  const {en} = registry.chart.clock;
  assert.match(en.instruction, /proportional/);
  assert.match(en.instruction, /future or missing/);
});

test('adapt preserves an unchanged brief and only replaces an incompatible chart', () => {
  const picker = createPicker(randomFrom(1));
  assert.equal(picker.current(), null);
  const dense = picker.adapt(daily);
  assert.equal(picker.adapt({...daily,total:9999,partial:true}), dense);
  const sparse = picker.adapt({...daily,sparse:true});
  assert.equal(sparse.chart, 'ledger');
  for (const dim of ['medium','layout','palette']) assert.equal(sparse[dim], dense[dim]);
  assert.equal(picker.canChange({...daily,sparse:true}, 'chart'), false);
  assert.equal(picker.next({...daily,sparse:true}, 'chart'), sparse);
  const filled = picker.adapt({...weekly,partial:true});
  assert.ok(isCompatible(filled, weekly));
  assert.notEqual(filled.chart, 'ledger');
  for (const dim of ['medium','layout','palette']) assert.equal(filled[dim], dense[dim]);
  assert.equal(picker.current(), filled);
});

test('single-axis edits preserve the other three axes and find compatible media throughout the whole registry', () => {
  const picker = createPicker(randomFrom(9314));
  picker.adapt(daily);
  for (let i = 0; i < 24; i++) {
    for (const dimension of ['chart','medium','palette','medium','layout','medium']) {
      const before = picker.current();
      const hasAlternative = Object.keys(registry[dimension]).some(id =>
        id !== before[dimension] && isCompatible({...before,[dimension]:id}, daily));
      assert.equal(picker.canChange(daily, dimension), hasAlternative, dimension);
      const after = picker.next(daily, dimension);
      if (hasAlternative) assert.notEqual(after[dimension], before[dimension]);
      else assert.equal(after, before);
      for (const other of dimensions.filter(dim => dim !== dimension)) assert.equal(after[other], before[other]);
    }
  }
});

test('recipe descriptions follow script and regional language settings and expose the actual choice', () => {
  const recipe = {medium:'risograph', chart:'columns', palette:'indigo', layout:'masthead'};
  const full = describe(recipe, 'zh-CN');
  for (const dimension of dimensions) {
    assert.equal(full[dimension].id, recipe[dimension]);
    assert.ok(full[dimension].instruction.length >= 15);
  }
  assert.deepEqual(describe(recipe, 'zh-Hant-HK'), describe(recipe, 'zh-TW'));
  assert.deepEqual(describe(recipe, 'zh_CN'), full);
  assert.deepEqual(describe(recipe, 'fr-FR'), describe(recipe, 'fr'));
  assert.deepEqual(describe(recipe, 'it'), describe(recipe, 'en'));
});

test('unknown choices cannot validate a recipe or enable an unrelated picker control', () => {
  const picker = createPicker(() => 0);
  const r = picker.adapt(weekly);
  for (const dim of dimensions) {
    assert.equal(isCompatible({...r,[dim]:'__proto__'}, weekly), false);
  }
  assert.equal(isCompatible(null, daily), false);
  assert.equal(picker.canChange(weekly, 'surprise'), false);
  assert.equal(picker.next(weekly, 'surprise'), r);
});
