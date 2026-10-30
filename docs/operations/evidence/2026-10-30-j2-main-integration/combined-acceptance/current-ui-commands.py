import runpy,sys
m=runpy.run_path('/tmp/b-j2-main-run.py');run=m['run'];tmp=m['TMP'];target=m['TARGET']
for name,cmd in [
 ('b-ui-bundle',['node',str(tmp/'runners/build-b-ui.mjs')]),
 ('b-ui-real-chrome',['node',str(tmp/'runners/run-b-ui.mjs'),'chrome','/Applications/Google Chrome.app/Contents/MacOS/Google Chrome','current329']),
 ('b-ui-real-wkwebview',['node',str(tmp/'runners/run-b-ui.mjs'),'wkwebview',target+'/debug/noname-next-desktop','current329']),
]:
 if run(name,cmd):sys.exit(1)
