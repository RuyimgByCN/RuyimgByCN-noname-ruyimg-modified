import pathlib,subprocess,sys
root=pathlib.Path.cwd();e=root.parent/'evidence/a330';binary='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target/debug/game-server'
for argv in [['node',str(e/'build-runner.mjs'),'apps/web-client',str(e/'ui-dist-final')],['node',str(e/'ui-real.mjs'),str(e/'browser-final'),str(e/'ui-dist-final'),binary]]:
 print('RUN',argv,flush=True);r=subprocess.run(argv)
 if r.returncode:sys.exit(r.returncode)
