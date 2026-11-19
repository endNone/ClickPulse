import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]


def load_script(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class MemoryToolsTests(unittest.TestCase):
    def test_prepare_preserves_utf8_sources_under_a_gbk_default(self):
        ui = load_script('measure_memory')
        with patch.dict(sys.modules, {'measure_memory': ui}):
            recorder = load_script('measure_runtime_memory')
        original_open = Path.open

        def gbk_open(path, mode='r', buffering=-1, encoding=None, errors=None, newline=None):
            if 'b' not in mode and encoding in (None, 'locale'):
                encoding = 'gbk'
            return original_open(path, mode, buffering, encoding, errors, newline)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'source'
            files = ['src/desktop.rs', 'src/desktop/webview.rs', 'src/desktop/window.rs',
                     'src/runtime.rs', 'ui/app.js']
            marker = '// UTF-8 source: 键鼠统计 🔵\n'
            for name in files:
                path = source / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes((ROOT / name).read_bytes() + marker.encode('utf-8'))
            # UTF-8 mode resolves an omitted encoding before Path.open runs.
            # Force only that default to locale, then emulate a GBK machine;
            # explicitly requested UTF-8 must still pass through unchanged.
            with patch('io.text_encoding', side_effect=lambda encoding, *_: encoding or 'locale'), \
                    patch.object(Path, 'open', gbk_open):
                with self.assertRaises(UnicodeDecodeError):
                    (source / 'ui/app.js').read_text()
                ui.prepare(source, root / 'ui')
                recorder.prepare(source, root / 'recorder')
            for folder in ['ui', 'recorder']:
                for name in files:
                    self.assertIn(marker, (root / folder / name).read_text(encoding='utf-8'))
            self.assertIn('ProbeAssets', (root / 'ui/src/desktop.rs').read_text(encoding='utf-8'))
            prepared = (root / 'ui/src/desktop.rs').read_text(encoding='utf-8')
            for call in ['_target.show_application();', 'window.set_focus();']:
                self.assertIn('if std::env::var_os("CLICKPULSE_MEMORY_PROBE").is_none() { '+call+' }', prepared)

            self.assertIn('CLICKPULSE_MEMORY_PROBE', (root / 'recorder/src/bin/memory_runtime.rs').read_text(encoding='utf-8'))


if __name__ == '__main__':
    unittest.main()
