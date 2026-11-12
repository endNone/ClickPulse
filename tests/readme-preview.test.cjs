const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const {execFileSync} = require('node:child_process');

test('documentation preview shows the current package version and current controls', () => {
  const root = path.resolve(__dirname, '..');
  const output = fs.mkdtempSync(path.join(os.tmpdir(), 'clickpulse-preview-'));
  try {
    execFileSync(process.execPath, [path.join(root, 'scripts/prepare-readme-preview.cjs'), output]);
    const version = fs.readFileSync(path.join(root, 'Cargo.toml'), 'utf8').match(/^version = "([^"]+)"/m)[1];
    const html = fs.readFileSync(path.join(output, 'index.html'), 'utf8');
    assert.ok(html.includes('>'+version+'<'), 'the visible version must match Cargo.toml');
    assert.ok(!html.includes('__VERSION__'));
    assert.ok(!html.includes('Export CSV'));
    assert.ok(!html.includes('Hourly totals; partial hours at each end'));
  } finally {
    fs.rmSync(output, {recursive:true, force:true});
  }
});
