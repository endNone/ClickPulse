"""Provision NSIS on a Windows CI runner, verifying each bounded download attempt."""
import subprocess
import sys
import time

from build_windows_installer import compiler_path


def ensure_compiler(attempts=3, retry_seconds=15):
    if attempts < 1:
        raise ValueError('At least one NSIS installation attempt is required.')
    try:
        return compiler_path()
    except RuntimeError:
        pass

    last_error = None
    for attempt in range(1, attempts + 1):
        print(f'Installing NSIS (attempt {attempt}/{attempts})', flush=True)
        try:
            result = subprocess.run(
                ['choco', 'install', 'nsis', '--no-progress', '-y', '--limit-output'],
                check=False,
                timeout=180,
            )
            # Chocolatey can report success after a failed feed request. Check
            # the actual compiler instead of trusting only the process status.
            path = compiler_path()
            if result.returncode == 0:
                return path
            last_error = RuntimeError(f'Chocolatey exited with code {result.returncode}.')
        except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
            last_error = error
        if attempt < attempts:
            print(f'NSIS is not ready; retrying in {retry_seconds}s: {last_error}', flush=True)
            time.sleep(retry_seconds)
    raise RuntimeError(f'NSIS installation failed after {attempts} attempts: {last_error}') from last_error


if __name__ == '__main__':
    if sys.platform != 'win32':
        raise SystemExit('NSIS provisioning is intended only for Windows CI runners.')
    try:
        print(f'NSIS compiler ready: {ensure_compiler()}', flush=True)
    except (ValueError, RuntimeError) as error:
        raise SystemExit(str(error)) from error
