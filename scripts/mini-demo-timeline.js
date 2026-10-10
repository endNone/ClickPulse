/* Deterministic documentation-only input events; no OS input or live database. */
(() => {
  const frameMs = 80;
  const events = [
    {frame: 19, type: 'drag', keys: 0, clicks: 1},
    {frame: 49, type: 'review', keys: 0, clicks: 1},
    {frame: 61, type: 'focus', keys: 0, clicks: 1},
  ];
  let next = 68;
  const cadence = [2, 2, 3, 1, 2, 2, 3];
  const sentence = 'Make activity easier to understnad.';
  for (let index = 0; index < sentence.length; index++) {
    const char = sentence[index];
    events.push({frame: next, type: 'type', char, keys: /[A-Z]/.test(char) ? 2 : 1, clicks: 0});
    next += cadence[index % cadence.length] + (char === ' ' ? 3 : 0);
  }
  next += 8; // Notice the transposed letters, then correct the word.
  for (let index = 0; index < 4; index++, next += 2)
    events.push({frame: next, type: 'backspace', keys: 1, clicks: 0});
  next += 3;
  for (const char of 'and.') {
    events.push({frame: next, type: 'type', char, keys: 1, clicks: 0});
    next += 2;
  }
  const saveFrame = next + 7;
  events.push({frame: saveFrame, type: 'save', keys: 2, clicks: 0}); // Cmd + S.
  const finishFrame = saveFrame + 21;
  events.push({frame: finishFrame, type: 'finish', keys: 0, clicks: 1});
  const maxFrame = finishFrame + 26;
  function stateAt(frame) {
    if (!Number.isInteger(frame) || frame < 0 || frame > maxFrame)
      throw new RangeError('Unknown documentation frame');
    let text = '', keys = 0, clicks = 0, lastKeyFrame = -Infinity;
    for (const event of events) {
      if (event.frame > frame) break;
      keys += event.keys;
      clicks += event.clicks;
      if (event.type === 'type') text += event.char;
      if (event.type === 'backspace') text = text.slice(0, -1);
      if (event.keys) lastKeyFrame = event.frame;
    }
    const click = events.find(event => event.clicks && event.frame <= frame && frame - event.frame < 3);
    return {frame, text, keys, clicks, lastKeyFrame, click,
      reviewed: frame >= 49, focused: frame >= 61, saved: frame >= saveFrame,
      finished: frame >= finishFrame, dragging: frame >= 19 && frame < 35};
  }
  const timeline = {frameMs, maxFrame, frameCount: maxFrame + 1,
    durationMs: (maxFrame + 1) * frameMs, saveFrame, finishFrame, events, stateAt};
  if (typeof module !== 'undefined') module.exports = timeline;
  if (typeof window !== 'undefined') window.ClickPulseMiniDemo = timeline;
})();
