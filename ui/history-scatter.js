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

  // Historical buckets are sorted once. Today's point follows equal counts,
  // keeping the median and keyboard traversal identical to a stable full sort.
  function withToday(points, point) {
    if (!point) return points;
    let low = 0, high = points.length;
    while (low < high) {
      const middle = Math.floor((low + high) / 2);
      if (points[middle].count <= point.count) low = middle + 1;
      else high = middle;
    }
    const combined = points.slice();
    combined.splice(low, 0, point);
    return combined;
  }

  // Only for the explicitly marked demo. Preserve each synthetic day's totals.
  function demoHours(records,lastHour=23) {
    const rows = [];
    for (const {day, counts} of records) {
      const seed = Date.parse(day + 'T00:00:00Z') / 86400000;
      const weights = Array.from({length: 24}, (_, h) => {
        if(h>lastHour)return 0;
        const noise = Math.abs(Math.sin(seed * 19 + h * 37));
        if (h < 7 || h > 22) return noise > .96 ? noise * .13 : 0;
        return (.18 + noise * noise) * (h >= 9 && h <= 18 && h !== 12 ? 4 : 1);
      });
      if(!weights.some(Boolean))weights[lastHour]=1;
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

  // Boundary hours are half-open: 08:00–09:00 contains only the 08:00 bucket.
  function createHourRange(document, onChange) {
    const track = document.getElementById('scatter-track');
    const from = document.getElementById('scatter-from'), to = document.getElementById('scatter-to');
    const period = document.getElementById('scatter-period');
    const range = {start:0, end:24};
    let drag = null;
    const clamp = (value, low, high) => Math.min(high, Math.max(low, value));

    function sync() {
      const {start, end} = range;
      track.dataset.full = String(start === 0 && end === 24);
      track.style.setProperty('--range-left', start / 24 * 100 + '%');
      track.style.setProperty('--range-right', (24 - end) / 24 * 100 + '%');
      period.textContent = clock(start) + ' – ' + clock(end);
      for (const [handle, value, min, max] of [[from,start,0,end-1], [to,end,start+1,24]]) {
        handle.setAttribute('aria-valuenow', String(value));
        handle.setAttribute('aria-valuetext', clock(value));
        handle.setAttribute('aria-valuemin', String(min));
        handle.setAttribute('aria-valuemax', String(max));
      }
    }
    function set(start, end) {
      if (start === range.start && end === range.end) return;
      range.start = start; range.end = end;
      sync();
      onChange();
    }
    for (const handle of [from, to]) {
      handle.addEventListener('keydown', event => {
        const left = handle === from, value = left ? range.start : range.end;
        const min = left ? 0 : range.start + 1, max = left ? range.end - 1 : 24;
        const change = {ArrowLeft:-1, ArrowDown:-1, ArrowRight:1, ArrowUp:1, PageDown:-3, PageUp:3};
        let next;
        if (event.key === 'Home') next = min;
        else if (event.key === 'End') next = max;
        else if (typeof change[event.key] === 'number') next = clamp(value + change[event.key], min, max);
        else return;
        event.preventDefault();
        if (left) set(next, range.end); else set(range.start, next);
      });
    }
    function pointerHour(event) {
      const rect = track.getBoundingClientRect();
      return clamp((event.clientX - rect.left) * 24 / rect.width, 0, 24);
    }
    track.addEventListener('pointerdown', event => {
      if (drag || event.button !== 0 || event.isPrimary === false) return;
      const origin = pointerHour(event);
      let mode = 'select';
      if (event.target === from || event.target === to) {
        mode = event.target === from ? 'start' : 'end';
        event.target.focus();
      } else if ((range.start > 0 || range.end < 24) && origin >= range.start && origin < range.end) {
        mode = 'pan';
      }
      drag = {id:event.pointerId, mode, origin, ...range};
      track.setPointerCapture(event.pointerId);
      event.preventDefault();
    });
    track.addEventListener('pointermove', event => {
      if (!drag || event.pointerId !== drag.id) return;
      const {mode, start, end, origin} = drag;
      const hour = pointerHour(event), delta = Math.round(hour - origin);
      if (mode === 'start') set(clamp(start + delta, 0, end - 1), end);
      else if (mode === 'end') set(start, clamp(end + delta, start + 1, 24));
      else if (mode === 'pan') {
        const offset = clamp(delta, -start, 24 - end);
        set(start + offset, end + offset);
      } else {
        const first = Math.round(origin), last = Math.round(hour);
        if (first !== last) set(Math.min(first, last), Math.max(first, last));
      }
    });
    const finish = event => { if (drag && event.pointerId === drag.id) drag = null; };
    for (const type of ['pointerup','pointercancel','lostpointercapture']) track.addEventListener(type, finish);
    sync();
    return range;
  }

  function create({document, window, I, H, send}) {
    const get = id => document.getElementById(id);
    const canvas = get('scatter-plot'), status = get('scatter-status');
    const weekdays = get('scatter-weekdays'), tooltip = get('scatter-tip');
    const toggles = devices.map(device => get('scatter-' + device));
    const shown = [true,true];
    let weekday = -1, locale = '', requestKey = '', requestId = 0, previousDays = [];
    let past = [], todayRows = [], todayKey = '', graph, hitColumns = [], selected = null;
    let dirty = true, themeKey = '', failureTime = 0;
    let peaks = [], columnWidth = 0, lastFrame = '';
    let pastGraph = null, liveBuckets = null;
    const background = document.createElement('canvas');
    const range = createHourRange(document, () => { forgetPoint(); renderRange(); });
    const overview = Array.from({length:24}, () => document.createElement('i'));
    get('scatter-peaks').replaceChildren(...overview);

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
    function releaseBackground() {
      lastFrame = '';
      if (background.width) background.width = background.height = 0;
    }
    function draw(changedHours) {
      const w = canvas.clientWidth, h = canvas.clientHeight;
      if (!w || !h || !graph?.maximum || canvas.hidden) return;
      const dpr = window.devicePixelRatio || 1, pw = Math.round(w*dpr), ph = Math.round(h*dpr);
      const {start,end} = range, hours = end-start;
      const scale = H.countScale(Math.max(...peaks.slice(start,end)));
      const frame = [w,h,dpr,start,end,scale.ceiling,scale.step,themeKey,locale].join(':');
      const redrawPast = frame !== lastFrame;
      if (!redrawPast && changedHours && !changedHours.some(h => h >= start && h < end)) {
        if (selected) choose(selected);
        return;
      }
      if (canvas.width !== pw) canvas.width = pw;
      if (canvas.height !== ph) canvas.height = ph;
      const ctx = canvas.getContext('2d'), style = window.getComputedStyle(canvas);
      const colors = ['--scatter-key','--scatter-mouse'].map(name => style.getPropertyValue(name).trim());
      const font = '11px ' + style.fontFamily;
      ctx.font = font;
      const left = Math.ceil(ctx.measureText(I.number(scale.ceiling)).width) + 12;
      const right = w-18, top = 10, base = h-25, band = (right-left)/hours;
      columnWidth = band;
      const both = shown.every(Boolean);
      const x = (hour, device) => left+band*(hour-start+(both ? (device === 0 ? .28 : .72) : .5));
      const y = count => base-(base-top)*count/scale.ceiling;
      function dot(context, point) {
        point.x = x(point.hour,point.device) + band*(both ? .35 : .65)*point.jitter;
        point.y = y(point.count);
        context.beginPath(); context.arc(point.x,point.y,2.35,0,2*Math.PI); context.fill();
      }
      if (redrawPast) {
        background.width = pw; background.height = ph;
        const bg = background.getContext('2d');
        bg.scale(pw/w,ph/h); bg.font = font;
        bg.textAlign = 'right'; bg.textBaseline = 'middle';
        bg.fillStyle = style.color; bg.strokeStyle = style.getPropertyValue('--line');
        for (const tick of scale.ticks) {
          bg.fillText(I.number(tick),left-8,y(tick));
          bg.beginPath(); bg.moveTo(left,y(tick)); bg.lineTo(right,y(tick)); bg.stroke();
        }
        bg.textAlign = 'center'; bg.textBaseline = 'bottom';
        const step = [1,2,3,4,6,8,12,24].find(value => value*band >= 54) || 24;
        for (let hour = start; hour < end; hour += step) {
          if (hour !== start && (end-hour)*band < 48) break;
          bg.fillText(clock(hour),left+(hour-start)*band,h);
        }
        bg.fillText(clock(end),right,h);
        bg.globalAlpha = .48;
        for (let hour = start; hour < end; hour++) {
          pastGraph.buckets[hour].forEach((points,device) => {
            bg.fillStyle = colors[device];
            for (const point of points) dot(bg,point);
          });
        }
        hitColumns = Array.from({length:24},()=>[]);
        lastFrame = frame;
      }

      // The axes and historical dots are static until their scale or data changes.
      ctx.setTransform(1,0,0,1,0,0);
      ctx.clearRect(0,0,pw,ph);
      ctx.drawImage(background,0,0);
      ctx.setTransform(pw/w,0,0,ph/h,0,0);
      for (let hour = start; hour < end; hour++) {
        const pair = graph.buckets[hour];
        if (redrawPast || !changedHours || changedHours.includes(hour)) hitColumns[hour] = pair[0].concat(pair[1]);
        pair.forEach((points,device) => {
          if (!points.length) return;
          ctx.fillStyle = colors[device];
          const live = liveBuckets[hour][device][0];
          if (live) {
            ctx.globalAlpha = .48; dot(ctx,live); ctx.globalAlpha = 1;
          }
          const mid = (points.length-1)/2;
          const median = (points[Math.floor(mid)].count + points[Math.ceil(mid)].count)/2;
          const half = Math.min(7,band*(both ? .18 : .32));
          ctx.fillRect(x(hour,device)-half,y(median)-1,half*2,2);
        });
      }
      if (selected) choose(selected);
    }
    function renderRange(changedHours) {
      const pending = past === null;
      const empty = !peaks.slice(range.start, range.end).some(Boolean);
      get('scatter-range').hidden = !graph?.maximum && range.start === 0 && range.end === 24;
      canvas.hidden = empty;
      status.hidden = !empty;
      if (empty) {
        status.textContent = I.t(pending ? failureTime ? 'error.stats' : 'common.loading'
          : 'scatter.empty');
        hitColumns = [];
        releaseBackground();
      } else {
        draw(changedHours);
      }
    }
    function render() {
      const previous = selected;
      forgetPoint();
      if (past === null) {
        pastGraph = liveBuckets = graph = null;
        peaks = [];
      }
      const rebuild = !pastGraph;
      const changedHours = [];
      if (past !== null) {
        if (rebuild) {
          lastFrame = '';
          pastGraph = group(past, weekday, shown, H);
          graph = {buckets: pastGraph.buckets.slice(), maximum: 0};
        }
        const next = group(todayRows, weekday, shown, H).buckets;
        for (let hour = 0; hour < 24; hour++) {
          const pair = next[hour];
          if (!rebuild && pair.every((points, device) => {
            const old = liveBuckets[hour][device][0], point = points[0];
            return old?.day === point?.day && old?.count === point?.count;
          })) {
            next[hour] = liveBuckets[hour];
            continue;
          }
          changedHours.push(hour);
          const combined = pastGraph.buckets[hour].map((points, device) => withToday(points, pair[device][0]));
          graph.buckets[hour] = combined;
          peaks[hour] = Math.max(...combined.map(points => points.length ? points[points.length-1].count : 0));
        }
        liveBuckets = next;
        graph.maximum = Math.max(...peaks);
      }
      overview.forEach((bar, hour) => {
        const height = (peaks[hour] ? 100 * peaks[hour] / graph.maximum : 0) + '%';
        if (bar.style.height !== height) bar.style.height = height;
      });
      if (previous && graph) selected = graph.buckets[previous.hour][previous.device].find(point => point.day === previous.day);
      renderRange(rebuild ? undefined : changedHours);
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
          if (weekday === index-1) return;
          weekday = index-1;
          pastGraph = null;
          Array.from(weekdays.children).forEach((b,i) => b.setAttribute('aria-pressed', String(i === index)));
          render();
        });
        return button;
      }));
    }
    function syncDevices() {
      const onlyOne = shown.filter(Boolean).length === 1;
      toggles.forEach((button, i) => {
        button.setAttribute('aria-pressed', String(shown[i]));
        button.setAttribute('aria-disabled', String(onlyOne && shown[i]));
      });
    }
    syncDevices();
    toggles.forEach((button, i) => button.addEventListener('click', () => {
      if (shown[i] && shown.filter(Boolean).length === 1) return;
      shown[i] = !shown[i];
      pastGraph = null;
      syncDevices();
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
        requestKey = key; previousDays = pastDays; requestId++; failureTime = 0; dirty = true; pastGraph = null;
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
        if (!column.length || Math.abs(column[0].x-x) > columnWidth + 8) continue;
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
        const up = event.key === 'ArrowUp';
        const current = col.indexOf(selected);
        const next = current < 0 ? (up ? 0 : col.length-1) : (current+(up ? 1 : -1)+col.length)%col.length;
        choose(col[next]);
      }
    });
    if (window.ResizeObserver) new window.ResizeObserver(() => draw()).observe(canvas);
    return {
      update,
      receive(id, rows) {
        if (id !== requestId) return;
        past = rows;
        pastGraph = null;
        failureTime = rows === null ? Date.now() : 0;
        render();
      },
      clear() {
        if (!requestKey) return;
        requestId++; requestKey = ''; previousDays = []; past = []; pastGraph = liveBuckets = graph = null;
        hitColumns = []; peaks = []; releaseBackground(); dirty = true;
        canvas.width = canvas.height = 0;
        forgetPoint();
      }
    };
  }
  return {create, group, demoHours};
})();
if (typeof module !== 'undefined') module.exports = ClickPulseScatter;
