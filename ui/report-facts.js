'use strict';
const ClickPulseReportFacts = (() => {
  const commonActions = new Map([
    ['Copy', 'copy'], ['Paste', 'paste'], ['Cut', 'cut'], ['Undo', 'undo'],
    ['Redo', 'redo'], ['Select all', 'select_all'], ['Save', 'save'],
    ['Find', 'find'], ['Print', 'print'],
  ]);

  function shortcutLabel(row, I, C) {
    const chord = C.chordParts(row).join(' + ');
    // A keypad alias can explain an entry in the encyclopedia, but is not
    // evidence of the user's intended action. Keep the physical chord visible.
    const exact = C.matches(row).filter(entry =>
      entry.platform === row.platform && entry.modifiers === row.modifiers && entry.key === row.key);
    if (exact.length !== 1) return chord;
    const entry = exact[0];
    let meaning = entry.title[I.locale === 'zh-CN' ? 'zh' : I.locale];
    if (!meaning) {
      const action = commonActions.get(entry.title.en);
      // The catalog supplies English reference titles for entries that have
      // no local translation. Keep its original wording in that case.
      meaning = action ? I.t('report_fact.action_' + action) : C.title(entry, I.locale);
    }
    return I.t('report_fact.shortcut', {chord, meaning, scope: C.scope(entry, I.locale)});
  }

  // Keep very small nonzero slices distinct from an empty interval.
  function percent(count, total, I, tenths = Math.round(1000 * count / total)) {
    if (count > 0 && tenths === 0) return '<' + I.percent(.001, 1);
    if (count < total && tenths === 1000) return '>' + I.percent(.999, 1);
    return I.percent(tenths / 1000, 1);
  }
  function hourSpan(hour, end = hour + 1) {
    const clock = value => String(value).padStart(2, '0') + ':00';
    return clock(hour) + '–' + clock(end);
  }
  function clockPeakValue({hour, dailyAverage}, I) {
    const count = dailyAverage > 0 && dailyAverage < 0.05 ? '<' + I.number(0.1) : I.number(Math.round(dailyAverage * 10) / 10);
    return hourSpan(hour) + ' · ' + I.t('report.daily_average', {count});
  }

  function bookReference(keyboard, H) {
    // Use the longest book that fits, or the shortest for a fractional comparison.
    let shortest = 0, fits = -1;
    H.books.forEach((book, index) => {
      if (book.characters < H.books[shortest].characters) shortest = index;
      if (book.characters <= keyboard && (fits < 0 || book.characters > H.books[fits].characters)) fits = index;
    });
    const index = fits < 0 ? shortest : fits;
    const comparison = H.bookComparison(keyboard, index);
    return {id: H.books[index].id, ...comparison};
  }

  const focusIds = ['shortcut', 'ranking', 'mix', 'peak', 'rhythm', 'book'];
  function canInclude(id, card, H) {
    if (!card?.loaded || card.keyboard + card.mouse === 0) return false;
    switch (id) {
      case 'shortcut': return card.keyboard > 0 && card.ranking.length > 0;
      case 'ranking': return card.keyboard > 0 && card.ranking.length >= 2;
      case 'mix': return true;
      case 'peak': return !!card.peak;
      case 'rhythm': return card.activeHours >= 2;
      // A comparison below one tenth of one percent is too small to be meaningful.
      case 'book': return card.keyboard > 0 && H.books.some(book => card.keyboard >= book.characters / 1000);
      default: return false;
    }
  }
  function available(card, H) {
    return focusIds.filter(id => canInclude(id, card, H));
  }

  function section(id, card, I, C, H, depth = 2) {
    if (!canInclude(id, card, H)) return {label: '', lines: []};
    const total = card.keyboard + card.mouse;
    const count = id === 'ranking' ? Math.min(card.ranking.length, depth === 3 ? 3 : 2) : 1;
    const label = I.t('report_focus.' + id, {count: I.number(count)});
    if (id === 'shortcut' || id === 'ranking') {
      const rows = card.ranking.slice(0, count);
      const lines = rows.map((row, index) => {
        const rank = card.ranking.findIndex(other => other.count === row.count) + 1;
        const tied = rank < index + 1 || card.ranking[index + 1]?.count === row.count;
        const prefix = tied || id === 'ranking' ? (tied ? '=' : '') + I.number(rank) + '. ' : '';
        return prefix + shortcutLabel(row, I, C) + ' · ' + I.t('share.shortcut_count', {count: I.number(row.count)});
      });
      return {label, lines};
    }
    if (id === 'mix') {
      // Round one side and compute the other as its complement so the pair sums to 100%.
      const keyboardTenths = Math.round(1000 * card.keyboard / total);
      return {label, lines: [I.t('report_prompt.mix', {
        keyboard: percent(card.keyboard, total, I, keyboardTenths),
        mouse: percent(card.mouse, total, I, 1000 - keyboardTenths),
      })]};
    }
    if (id === 'book') {
      const reference = bookReference(card.keyboard, H);
      const book = I.t('calendar.book_' + reference.id);
      const text = card.keyboard < reference.characters
        ? I.t('report_fact.book_part', {book, value: percent(card.keyboard, reference.characters, I)})
        : I.t('report_fact.book_multiple', {book, value: I.number(reference.copies)});
      return {label, lines: [text]};
    }
    if (id === 'peak') {
      const {hour, day, total: peakTotal} = card.peak;
      const period = day + ' · ' + hourSpan(hour);
      const share = percent(peakTotal, total, I);
      const tied = card.ties > 1;
      const times = I.t('share.shortcut_count', {count: I.number(peakTotal)});
      return {label, lines: [
        I.t(tied ? 'report.peak_tied' : 'report.peak') + ' · ' + period + ' · ' + times,
        tied ? I.t('report_fact.peak_tied', {share, count: I.number(card.ties)}) : I.t('report_fact.peak_share', {share}),
      ]};
    }
    // The remaining module describes the clock-time distribution across the selected dates.
    const bands = [];
    for (let start = 0; start < 24; start += 6) {
      const count = card.hours.slice(start, start + 6).reduce((sum, value) => sum + value, 0);
      if (count > 0) bands.push(hourSpan(start, start + 6) + ' ' + percent(count, total, I));
    }
    const lines = [I.t('report_fact.active', {count: I.number(card.activeHours)}), I.t('report_fact.periods', {bands: bands.join(' · ')})];
    if (card.weekly && card.clockPeak) {
      lines.push(I.t('report.clock_peak') + ' · ' + clockPeakValue(card.clockPeak, I));
    }
    return {label, lines};
  }

  return {available, section, shortcutLabel, clockPeakValue};
})();
if (typeof module !== 'undefined') module.exports = ClickPulseReportFacts;
