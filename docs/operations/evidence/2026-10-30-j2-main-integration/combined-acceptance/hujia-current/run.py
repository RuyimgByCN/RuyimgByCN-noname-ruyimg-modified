import runpy,sys,shutil
from pathlib import Path
m=runpy.run_path('/tmp/b-j2-main-run.py');run=m['run'];out=m['OUT'];h=m['TMP']/'hujia-current'
if run('hujia-current-native',['cargo','run','--offline','--manifest-path',str(h/'Cargo.toml'),'--',str(out/'hujia-current/vector.json')]):sys.exit(1)
shutil.copy2(h/'Cargo.lock',out/'hujia-current/Cargo.lock')
if run('hujia-current-actual-wasm',['node',str(h/'verify.mjs'),str(m['ROOT']),str(out/'hujia-current/vector.json'),str(out/'hujia-current/actual-wasm-report.json')]):sys.exit(1)
