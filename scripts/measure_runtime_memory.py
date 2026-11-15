"""Measure the recorder alone, using an isolated database and the native input backend.

This complements the deterministic UI comparison; its figures are not added to UI
figures or presented as a measurement of the combined installed application.
"""
import argparse
import ctypes
import json
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import sys
import time

from measure_memory import memory

RUST = r'''
use std::{path::PathBuf, time::{Duration, Instant}};
fn main() -> anyhow::Result<()> {
    let root = PathBuf::from(std::env::var("CLICKPULSE_MEMORY_PROBE")?);
    let mut runtime = clickpulse::runtime::Runtime::start_owned(&root.join("counts.db"), false)?;
    std::thread::sleep(Duration::from_secs(20));
    let status = runtime.capture_status()?;
    std::fs::write(root.join("ready.json"), serde_json::to_vec_pretty(&status)?)?;
    let limit = Instant::now() + Duration::from_secs(90);
    while !root.join("measured").exists() && Instant::now() < limit {
        std::thread::sleep(Duration::from_millis(100));
    }
    let final_status = runtime.capture_status()?;
    std::fs::write(root.join("final-status.json"), serde_json::to_vec_pretty(&final_status)?)?;
    runtime.shutdown()?;
    Ok(())
}
'''


def prepare(source, output):
    shutil.copytree(source, output, ignore=shutil.ignore_patterns('.git', 'target', 'dist', '.Codex', '__pycache__'))
    path = output/'src/runtime.rs'
    text = path.read_text()
    old = 'path: crate::platform::log_path().ok(),'
    assert text.count(old) == 1, 'Status log must remain isolated'
    path.write_text(text.replace(old, 'path: Some(std::path::PathBuf::from(std::env::var("CLICKPULSE_MEMORY_PROBE").expect("isolated probe directory")).join("runtime.log")),', 1))
    (output/'src/bin').mkdir(exist_ok=True)
    (output/'src/bin/memory_runtime.rs').write_text(RUST)


def run(binary, output, label):
    output.mkdir(parents=True, exist_ok=False)
    with (output/'app.log').open('w') as log:
        app = subprocess.Popen([str(binary.resolve())], env=dict(os.environ, CLICKPULSE_MEMORY_PROBE=str(output.resolve())), stdout=log, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic()+60
            while not (output/'ready.json').exists():
                if app.poll() is not None or time.monotonic()>deadline:
                    raise RuntimeError('Recorder probe did not become ready: '+(output/'app.log').read_text())
                time.sleep(.25)
            samples = []
            for _ in range(10):
                if sys.platform == 'darwin':
                    lib = ctypes.CDLL('/usr/lib/libproc.dylib')
                    data = ctypes.create_string_buffer(4096)
                    if lib.proc_pid_rusage(app.pid, 0, data): raise RuntimeError('Process memory unavailable')
                    values = (ctypes.c_uint64*10).from_buffer(data, 16)
                    sample = {'resident_bytes': values[6], 'measured_bytes': values[7]}
                else:
                    sample = memory(app.pid)
                if sample is None: raise RuntimeError('Recorder disappeared')
                samples.append(sample)
                time.sleep(1)
            report = {'label': label, 'platform': __import__('platform').platform(), 'metric': {'darwin':'physical footprint','win32':'private working set (USS)'}.get(sys.platform,'proportional resident size (PSS)'), 'capture': json.loads((output/'ready.json').read_text()), 'median_bytes': statistics.median(s['measured_bytes'] for s in samples), 'samples':samples, 'note':'Recorder-only process; empty isolated database initially. Native input permission/status reported, never assume capture is active. Do not add to the separate UI measurement.'}
            (output/'measurements.json').write_text(json.dumps(report, indent=2)+'\n')
            (output/'measured').write_text('ok')
            app.wait(timeout=15)
            if app.returncode: raise RuntimeError('Recorder shutdown failed')
            print(json.dumps({k:report[k] for k in ['label','median_bytes','capture']}), flush=True)
        finally:
            if app.poll() is None: app.terminate();app.wait(timeout=15)


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    sub=parser.add_subparsers(dest='command',required=True)
    prep=sub.add_parser('prepare');prep.add_argument('source',type=Path);prep.add_argument('output',type=Path)
    measure=sub.add_parser('run');measure.add_argument('binary',type=Path);measure.add_argument('output',type=Path);measure.add_argument('--label',required=True)
    args=parser.parse_args()
    if args.command=='prepare':prepare(args.source.resolve(),args.output.resolve())
    else:run(args.binary,args.output.resolve(),args.label)
