'use strict';
const ClickPulseScatter = (() => {
  const devices = ['keyboard', 'mouse'];
  const clock = hour => String(hour).padStart(2, '0') + ':00';

  // A bucket contains observations from different dates at the same local hour.
  function group(rows, weekday, shown, history) {
    const buckets = Array.from({length: 24}, () => [[], []]);
    let previousDay, dayOfWeek, seed, maximum = 0;
    for (const [day, hour, keys, clicks] of rows) {
      if (day !== previousDay) {
        previousDay = day;
        dayOfWeek = history.weekday(day);
        seed = Math.floor(Date.parse(day + 'T00:00:00Z') / 86400000);
      }
      if (weekday !== -1 && dayOfWeek !== weekday) continue;
      for (let device = 0; device < 2; device++) {
        const count = device === 0 ? keys : clicks;
        if (!shown[device] || count <= 0) continue;
        // Stable jitter separates nearby days without suggesting minute precision.
        const jitter = (((seed * 421 + hour * 61 + device * 17) % 997 + 997) % 997) / 997 - 0.5;
        buckets[hour][device].push({day, hour, device, count, jitter});
        maximum = Math.max(maximum, count);
      }
    }
    for (const pair of buckets) for (const points of pair) points.sort((a, b) => a.count - b.count);
    return {buckets, maximum};
  }

  // Only for the explicitly marked demo. Preserve each synthetic day's totals.
  function demoHours(records) {
    const rows = [];
    for (const {day, counts} of records) {
      const seed = Date.parse(day + 'T00:00:00Z') / 86400000;
      const weights = Array.from({length: 24}, (_, h) => {
        const noise = Math.abs(Math.sin(seed * 19 + h * 37));
        if (h < 7 || h > 22) return noise > .96 ? noise * .13 : 0;
        return (.18 + noise * noise) * (h >= 9 && h <= 18 && h !== 12 ? 4 : 1);
      });
      const sum = weights.reduce((a,b) => a+b, 0);
      let cumulative = 0, keys = 0, clicks = 0;
      weights.forEach((weight, h) => {
        cumulative += weight;
        const nextKeys = Math.round(counts.keyboard * cumulative / sum);
        const nextClicks = Math.round(counts.mouse * cumulative / sum);
        if (nextKeys !== keys || nextClicks !== clicks) rows.push([day,h,nextKeys-keys,nextClicks-clicks]);
        keys = nextKeys; clicks = nextClicks;
      });
    }
    return rows;
  }

  function create({document, window, I, H, send}) {
    const get = id => document.getElementById(id);
    const canvas = get('scatter-plot'), caption = get('scatter-caption'), status = get('scatter-status');
    const weekdays = get('scatter-weekdays'), tooltip = get('scatter-tip');
    const toggles = devices.map(device => get('scatter-' + device));
    const shown = [true,true];
    let weekday = -1, locale = '', requestKey = '', requestId = 0, previousDays = [];
    let past = [], todayRows = [], todayKey = '', graph, hitColumns = [], selected = null;
    let dirty = true, themeKey = '', failureTime = 0;

    function forgetPoint() {
      selected = null;
      tooltip.hidden = true;
    }
    function choose(point) {
      if (!point) { forgetPoint(); return; }
      selected = point;
      tooltip.textContent = I.day(point.day) + ' · ' + clock(point.hour) + ' · ' +
        I.t('common.' + devices[point.device]) + ' ' + I.number(point.count);
      tooltip.hidden = false;
      const half = tooltip.offsetWidth / 2;
      tooltip.style.left = Math.max(half, Math.min(canvas.clientWidth - half, point.x)) + 'px';
      tooltip.style.top = Math.max(tooltip.offsetHeight + 6, point.y - 8) + 'px';
    }
    function draw() {
      const w = canvas.clientWidth, h = canvas.clientHeight;
      if (!w || !h || !graph || !graph.maximum || canvas.hidden) return;
      const dpr = window.devicePixelRatio || 1;
      canvas.width = Math.round(w * dpr); canvas.height = Math.round(h * dpr);
      const ctx = canvas.getContext('2d');
      ctx.scale(canvas.width / w, canvas.height / h);
      const style = window.getComputedStyle(canvas);
      const colors = ['--scatter-key','--scatter-mouse'].map(name => style.getPropertyValue(name).trim());
      const scale = H.countScale(graph.maximum);
      ctx.font = '11px ' + style.fontFamily;
      const left = Math.ceil(ctx.measureText(I.number(scale.ceiling)).width) + 12;
      const right = w - 18, top = 10, base = h - 25, band = (right-left) / 24;
      const y = count => base - (base-top) * count / scale.ceiling;
      ctx.lineWidth = 1;
      ctx.textAlign = 'right'; ctx.textBaseline = 'middle';
      for (const tick of scale.ticks) {
        ctx.fillStyle = style.color; ctx.fillText(I.number(tick), left-8, y(tick));
        ctx.strokeStyle = style.getPropertyValue('--line');
        ctx.beginPath(); ctx.moveTo(left,y(tick)); ctx.lineTo(right,y(tick)); ctx.stroke();
      }
      ctx.textAlign = 'center'; ctx.textBaseline = 'bottom';
      for (let hour = 0; hour <= 24; hour += w < 540 ? 6 : 3) {
        ctx.fillText(clock(hour),left+hour*band,h);
      }
      hitColumns = graph.buckets.map(() => []);
      const both = shown.every(Boolean);
      graph.buckets.forEach((pair, hour) => pair.forEach((points, device) => {
        if (!points.length) return;
        const x = left + band*(hour + (both ? (device === 0 ? .28 : .72) : .5));
        ctx.fillStyle = colors[device]; ctx.globalAlpha = .48;
        for (const point of points) {
          point.x = x + band * (both ? .35 : .65) * point.jitter;
          point.y = y(point.count);
          ctx.beginPath(); ctx.arc(point.x,point.y,2.35,0,2*Math.PI); ctx.fill();
          hitColumns[hour].push(point);
        }
        const mid = (points.length-1)/2;
        const median = (points[Math.floor(mid)].count + points[Math.ceil(mid)].count)/2;
        const half = Math.min(7, band * (both ? .18 : .32));
        ctx.globalAlpha = 1;
        ctx.fillRect(x-half, y(median)-1, half*2, 2);
      }));
      if (selected) choose(selected);
    }
    function render() {
      const previous = selected;
      forgetPoint();
      const pending = past === null;
      graph = pending ? null : group(past.concat(todayRows), weekday, shown, H);
      const empty = !graph || graph.maximum === 0;
      canvas.hidden = empty;
      status.hidden = !empty;
      caption.hidden = empty;
      if (empty) {
        status.textContent = I.t(pending ? failureTime ? 'error.stats' : 'common.loading'
          : shown.some(Boolean) ? 'scatter.empty' : 'scatter.select');
        hitColumns = [];
      } else {
        if (previous) selected = graph.buckets[previous.hour][previous.device].find(point => point.day === previous.day);
        draw();
      }
      dirty = false;
    }
    function labels() {
      locale = I.locale;
      const names = [I.t('scatter.all'), ...I.weekdays()];
      weekdays.replaceChildren(...names.map((name, index) => {
        const button = document.createElement('button');
        button.type = 'button'; button.textContent = name;
        button.setAttribute('aria-pressed', String(weekday === index-1));
        button.addEventListener('click', () => {
          weekday = index-1;
          Array.from(weekdays.children).forEach((b,i) => b.setAttribute('aria-pressed', String(i === index)));
          render();
        });
        return button;
      }));
    }
    toggles.forEach((button, i) => button.addEventListener('click', () => {
      shown[i] = !shown[i];
      button.setAttribute('aria-pressed', String(shown[i]));
      render();
    }));

    function update(snapshot, records, bounds, theme) {
      const today = snapshot.now.slice(0,10);
      const pastDays = records.filter(day => day.day < today && (day.counts.keyboard || day.counts.mouse));
      // Ordinary input changes only today's 24 counters, which are already in the snapshot.
      const key = JSON.stringify([snapshot.database_path, snapshot.demo, I.effectiveTimeZone]);
      const daysChanged = pastDays.length !== previousDays.length || pastDays.some((row,i) => {
        const old = previousDays[i];
        return row.day !== old.day || row.counts.keyboard !== old.counts.keyboard || row.counts.mouse !== old.counts.mouse;
      });
      if (key !== requestKey || daysChanged || (failureTime && Date.now()-failureTime > 15000)) {
        requestKey = key; previousDays = pastDays; requestId++; failureTime = 0; dirty = true;
        if (!pastDays.length) past = [];
        else if (snapshot.demo) past = demoHours(pastDays);
        else {
          past = null;
          send('history_hours',{id:requestId,start:pastDays[0].day,end:pastDays[pastDays.length-1].day,timezone:snapshot.timezone || 'system'});
        }
      }
      const current = today >= bounds.start && today <= bounds.end;
      const hours = current ? snapshot.hours || [] : [];
      const nextToday = JSON.stringify([today,hours]);
      if (nextToday !== todayKey) {
        todayKey = nextToday; dirty = true;
        todayRows = hours.map((counts,h) => [today,h,counts.keyboard,counts.mouse]);
      }
      if (locale !== I.locale) { labels(); dirty = true; }
      if (themeKey !== theme) { themeKey = theme; dirty = true; }
      if (dirty) render();
    }
    canvas.addEventListener('pointermove', event => {
      const rect = canvas.getBoundingClientRect();
      const x = (event.clientX-rect.left)*canvas.clientWidth/rect.width;
      const y = (event.clientY-rect.top)*canvas.clientHeight/rect.height;
      let nearest = null, distance = 64;
      for (const column of hitColumns) {
        if (!column.length || Math.abs(column[0].x-x) > canvas.clientWidth/24 + 8) continue;
        for (const p of column) {
          const squared = (p.x-x)**2 + (p.y-y)**2;
          if (squared <= distance) { distance = squared; nearest = p; }
        }
      }
      choose(nearest);
    });
    canvas.addEventListener('pointerleave', forgetPoint);
    canvas.addEventListener('blur', forgetPoint);
    canvas.addEventListener('focus', () => choose(hitColumns.find(c => c.length)?.[0]));
    canvas.addEventListener('keydown', event => {
      if (!['ArrowLeft','ArrowRight','ArrowUp','ArrowDown','Escape'].includes(event.key)) return;
      event.preventDefault();
      if (event.key === 'Escape') { forgetPoint(); return; }
      const occupied = hitColumns.filter(column => column.length);
      if (!occupied.length) return;
      const col = hitColumns[selected?.hour] || occupied[0];
      if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
        const offset = event.key === 'ArrowLeft' ? -1 : 1;
        choose(occupied[(occupied.indexOf(col)+offset+occupied.length)%occupied.length][0]);
      } else {
        const offset = event.key === 'ArrowUp' ? 1 : -1;
        choose(col[(col.indexOf(selected)+offset+col.length)%col.length]);
      }
    });
    if (window.ResizeObserver) new window.ResizeObserver(draw).observe(canvas);
    return {
      update,
      receive(id, rows) {
        if (id !== requestId) return;
        past = rows;
        failureTime = rows === null ? Date.now() : 0;
        render();
      },
      clear() {
        if (!requestKey) return;
        requestId++; requestKey = ''; previousDays = []; past = []; graph = null; hitColumns = []; dirty = true;
        canvas.width = canvas.height = 0;
        forgetPoint();
      }
    };
  }
  return {create, group, demoHours};
})();
if (typeof module !== 'undefined') module.exports = ClickPulseScatter;
