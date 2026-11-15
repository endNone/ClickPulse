"""Isolated UI memory comparison. Requires psutil only on Windows.

Prepare a disposable source copy, build it, then run its executable. The probe
uses the normal UI actions with demo data and cannot touch recording/history.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import sys
import time

HOOK = r'''
    // Measurement copy only; never part of the distributed App.
    if let Ok(directory) = std::env::var("CLICKPULSE_MEMORY_PROBE") {
        assert!(demo, "Memory measurements require isolated demo data");
        let proxy = event_loop.create_proxy();
        let (open_id, mini_id, hide_id, quit_id) = (show.id().clone(), mini.id().clone(), hide.id().clone(), quit.id().clone());
        std::thread::spawn(move || {
            let path = std::path::PathBuf::from(directory);
            std::fs::write(path.join("pid"), std::process::id().to_string()).unwrap();
            if __LIFECYCLE__ { std::fs::write(path.join("lifecycle"), "true").unwrap(); }
            for (name, action, seconds) in [
                ("background", "", 35),
                ("full_en", "open", 35),
                ("full_zh", "language", 35),
                ("mini", "mini", 35),
                ("hidden_10m", "hide", 610),
                ("reopened", "open", 35),
            ] {
                let started = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
                std::fs::write(path.join(format!("{name}.started")), started.to_string()).unwrap();
                let menu = match action { "open" => Some(&open_id), "mini" => Some(&mini_id), "hide" => Some(&hide_id), _ => None };
                if let Some(id) = menu { let _ = proxy.send_event(UiEvent::Menu(MenuEvent { id: id.clone() })); }
                if action == "language" {
                    let body = r#"{"action":"language","language":"zh-CN"}"#.to_owned();
                    __SEND_IPC__
                }
                std::fs::write(path.join("stage"), name).unwrap();
                std::thread::sleep(Duration::from_secs(seconds));
                std::fs::write(path.join(format!("{name}.ready")), "ready").unwrap();
                while !path.join(format!("{name}.measured")).exists() { std::thread::sleep(Duration::from_millis(100)); }
            }
            let _ = proxy.send_event(UiEvent::Menu(MenuEvent { id: quit_id }));
        });
    }
'''


def prepare(source, output):
    shutil.copytree(source, output, ignore=shutil.ignore_patterns('.git', 'target', 'dist', '.Codex', '__pycache__'))
    path = output/'src/desktop.rs'
    text = path.read_text()
    lifecycle = 'generation: u64, body: String' in text or 'generation: u64,\n        body: String' in text
    anchor = '    let mut next_refresh = Instant::now();'
    assert text.count(anchor) == 1
    send = 'let generation = std::fs::read_to_string(path.join("generation")).unwrap().parse().unwrap(); let _ = proxy.send_event(UiEvent::Ipc { generation, body });' if lifecycle else 'let _ = proxy.send_event(UiEvent::Ipc(body));'
    hook = HOOK.replace('__LIFECYCLE__', str(lifecycle).lower()).replace('__SEND_IPC__', send)
    text = text.replace(anchor, hook+'\n'+anchor)
    text = text.replace('        (clickpulse::presentation::Preferences::default(), None)', '        ({ let mut p = clickpulse::presentation::Preferences::default(); p.language = clickpulse::localization::LanguagePreference::English; p }, None)')
    ready = '''                        Request::Ready {} => {
                            if let Ok(directory) = std::env::var("CLICKPULSE_MEMORY_PROBE") {
                                let path = std::path::Path::new(&directory);
                                let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
                                std::fs::write(path.join("ui-ready"), ms.to_string()).unwrap();
                                __GENERATION__
                            }'''
    text = text.replace('                        Request::Ready {} => {', ready.replace('__GENERATION__', 'std::fs::write(path.join("generation"), generation.to_string()).unwrap();' if lifecycle else ''))
    if lifecycle:
        view = output/'src/desktop/webview.rs'
        body = view.read_text().replace('        self.inner.take();', '''        self.inner.take();
        if let Ok(directory) = std::env::var("CLICKPULSE_MEMORY_PROBE") {
            std::fs::write(std::path::Path::new(&directory).join("disposed"), self.generation.to_string()).unwrap();
        }''')
        body = body.replace('clickpulse::platform::data_dir()?.join("webview")', 'std::path::PathBuf::from(std::env::var("CLICKPULSE_MEMORY_PROBE").expect("probe directory")).join("webview")')
        view.write_text(body)
    else:
        text = text.replace('clickpulse::platform::data_dir()?.join("webview")', 'std::path::PathBuf::from(std::env::var("CLICKPULSE_MEMORY_PROBE").expect("probe directory")).join("webview")')
    path.write_text(text)
    path = output/'src/desktop/window.rs'
    text = path.read_text().replace('        _target.show_application();', '        if std::env::var_os("CLICKPULSE_MEMORY_PROBE").is_none() { _target.show_application(); }').replace('    window.set_focus();', '    if std::env::var_os("CLICKPULSE_MEMORY_PROBE").is_none() { window.set_focus(); }')
    path.write_text(text)


def processes():
    if sys.platform == 'win32':
        import psutil
        return {p.pid: p.info for p in psutil.process_iter(['ppid', 'name'])}
    rows = subprocess.check_output(['ps', '-eo', 'pid=,ppid=,comm='], text=True).splitlines()
    return {int(parts[0]): {'ppid': int(parts[1]), 'name': parts[2]} for row in rows if len(parts := row.strip().split(None, 2)) == 3}


def memory(pid):
    if sys.platform == 'win32':
        import psutil
        try:
            full = psutil.Process(pid).memory_full_info()
            return {'resident_bytes': full.rss, 'private_commit_bytes': full.private, 'measured_bytes': full.uss}
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            return None
    try:
        lines = Path(f'/proc/{pid}/smaps_rollup').read_text().splitlines()
        fields = {'Rss:', 'Pss:', 'Private_Clean:', 'Private_Dirty:'}
        values = {parts[0][:-1]: int(parts[1])*1024 for line in lines if len(parts := line.split()) >= 2 and parts[0] in fields}
        return {'resident_bytes': values['Rss'], 'private_bytes': values['Private_Clean']+values['Private_Dirty'], 'measured_bytes': values['Pss']}
    except (FileNotFoundError, PermissionError, ProcessLookupError):
        return None


def run(binary, output, label):
    output.mkdir(parents=True, exist_ok=False)
    env = dict(os.environ, CLICKPULSE_MEMORY_PROBE=str(output.resolve()))
    log = (output/'app.log').open('w')
    app = subprocess.Popen([str(binary.resolve()), '--demo', '--background'], env=env, stdout=log, stderr=subprocess.STDOUT)
    known = {app.pid}
    report = {'label': label, 'platform': sys.platform, 'system': __import__('platform').platform(), 'demo': True, 'metric': 'private working set (USS)' if sys.platform == 'win32' else 'proportional resident size (PSS)', 'states': {}}
    try:
        for stage in ['background', 'full_en', 'full_zh', 'mini', 'hidden_10m', 'reopened']:
            deadline = time.monotonic()+750
            while not (output/(stage+'.ready')).exists():
                if app.poll() is not None:
                    raise RuntimeError('App exited before '+stage+': '+(output/'app.log').read_text())
                if time.monotonic()>deadline:
                    raise RuntimeError('Timed out at '+stage)
                time.sleep(1)
            lifecycle = (output/'lifecycle').exists()
            if not (lifecycle and stage == 'background') and not (output/'ui-ready').exists():
                raise RuntimeError('WebView never acknowledged the loaded interface')
            if lifecycle and stage == 'hidden_10m' and not (output/'disposed').exists():
                raise RuntimeError('Hidden renderer was not disposed')
            if lifecycle and stage == 'reopened' and int((output/'generation').read_text()) < 2:
                raise RuntimeError('Renderer did not recreate and acknowledge Ready')
            samples = []
            for _ in range(5):
                table = processes()
                while True:
                    children = {pid for pid, row in table.items() if row['ppid'] in known}
                    if children <= known: break
                    known |= children
                rows = [{'pid': pid, 'name': table[pid]['name'], **m} for pid in known if pid in table and (m := memory(pid))]
                samples.append({'processes': rows, 'main_bytes': sum(p['measured_bytes'] for p in rows if p['pid'] == app.pid), 'webview_bytes': sum(p['measured_bytes'] for p in rows if p['pid'] != app.pid), 'total_bytes': sum(p['measured_bytes'] for p in rows)})
                time.sleep(1)
            report['states'][stage] = {'samples': samples, **{'median_'+k: statistics.median(s[k] for s in samples) for k in ['main_bytes', 'webview_bytes', 'total_bytes']}}
            if lifecycle and stage == 'background' and any(sample['webview_bytes'] for sample in samples):
                raise RuntimeError('Background launch unexpectedly created renderer processes')
            if lifecycle and stage == 'reopened':
                report['reopen_ready_ms'] = int((output/'ui-ready').read_text()) - int((output/'reopened.started').read_text())
            (output/'measurements.json').write_text(json.dumps(report, indent=2)+'\n')
            (output/(stage+'.measured')).write_text('ok')
            print(stage, round(report['states'][stage]['median_total_bytes']/2**20, 1), 'MiB', flush=True)
        app.wait(timeout=30)
        if app.returncode: raise RuntimeError('Probe failed to exit cleanly')
    finally:
        if app.poll() is None: app.terminate(); app.wait(timeout=15)
        log.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    prep = sub.add_parser('prepare');prep.add_argument('source', type=Path);prep.add_argument('output', type=Path)
    measure = sub.add_parser('run');measure.add_argument('binary', type=Path);measure.add_argument('output', type=Path);measure.add_argument('--label', required=True)
    args = parser.parse_args()
    if args.command == 'prepare': prepare(args.source.resolve(), args.output.resolve())
    else: run(args.binary, args.output.resolve(), args.label)
