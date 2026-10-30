from pathlib import Path
import json,subprocess,time
out=Path('/tmp/pa09-webview-5b7ad3ab').resolve()
started=time.monotonic()
while not (out/'native-build.done').exists():
    if time.monotonic()-started>1250: raise SystemExit('native build status timeout')
    time.sleep(2)
assert (out/'native-build.done').read_text()=='0', 'native build failed; do not launch another binary'
with (out/'evidence/run-webview.log').open('w') as log:
    result=subprocess.run(['python3','/tmp/noname-run-serialized.py','python3',str(out/'harness/run-webview.py')],cwd=out,stdout=log,stderr=subprocess.STDOUT)
(out/'evidence/run-webview-result.json').write_text(json.dumps({'exit':result.returncode,'wall_including_build_wait_and_lock_seconds':time.monotonic()-started},indent=2))
