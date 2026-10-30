import json, os, pathlib, shutil, subprocess, sys, time
root = pathlib.Path(__file__).resolve().parent.parent / 'worktree'
evidence = root.parent / 'evidence'
label, *command = sys.argv[1:]
target = root.parent / 'target'
target.mkdir(exist_ok=True)
# Only this run's target is owned here; never clean any shared cache.
for path in target.glob('**/incremental'):
    shutil.rmtree(path)
free = shutil.disk_usage(root).free
assert free > 15 * 1024**3, free
env = os.environ.copy()
env.update(CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='2', CARGO_TERM_COLOR='never')
started = time.time()
with (evidence / (label + '.log')).open('w') as log:
    log.write(json.dumps({'cwd': str(root), 'command': command, 'platform': sys.platform, 'disk_free_before': free, 'target': str(target), 'incremental': 0, 'jobs': 2})+'\n'); log.flush()
    result = subprocess.run(['python3', '/tmp/noname-run-serialized.py', *command], cwd=root, env=env, stdout=log, stderr=subprocess.STDOUT)
summary = {'command':command, 'exit':result.returncode, 'seconds':time.time()-started,'disk_free_after':shutil.disk_usage(root).free}
(evidence / (label + '.json')).write_text(json.dumps(summary,indent=2)+'\n')
print(summary)
