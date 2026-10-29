import pathlib,subprocess,sys,os
base=pathlib.Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage4-9rhcfkw2')
env=['env','CARGO_TARGET_DIR='+str(base/'target-after'),'CARGO_BUILD_JOBS=8','CARGO_INCREMENTAL=1']
packages=['-p','engine-native-kernel','-p','engine-native-content','-p','engine-native']
commands=[
 ('check-final',env+['cargo','check','--offline',*packages,'--all-targets']),
 ('native-tests',env+['cargo','test','--offline',*packages]),
 ('test-list-after',env+['cargo','test','--offline',*packages,'--','--list']),
 ('dependent-tests',env+['cargo','test','--offline','-p','engine-native-wasm','-p','content-native-loader','-p','game-server']),
 ('gate1b',env+['RUSTFLAGS=-D warnings','cargo','check','--offline',*packages,'--all-targets']),
]
for name,cmd in commands:
 rc=subprocess.call([sys.executable,str(base/'run.py'),name,str(pathlib.Path.cwd()),*cmd])
 if rc:break
sys.exit(rc)
