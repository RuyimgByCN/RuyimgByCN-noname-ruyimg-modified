from pathlib import Path
import subprocess,sys
root=Path.cwd();e=root.parent/'evidence/a330-corrections';binary='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target/debug/game-server'
if not (e/'ui-dist/index.html').exists():
 r=subprocess.run(['node',str(root.parent/'evidence/a330/build-runner.mjs'),'apps/web-client',str(e/'ui-dist')]);
 if r.returncode:sys.exit(r.returncode)
for skew,name in [(60000,'fast'),(-60000,'slow')]:
 argv=['node',str(e/'ui-skew.mjs'),str(e/('browser-'+name)),str(e/'ui-dist'),binary,str(skew)]
 print('RUN',argv,flush=True);r=subprocess.run(argv)
 if r.returncode:sys.exit(r.returncode)
