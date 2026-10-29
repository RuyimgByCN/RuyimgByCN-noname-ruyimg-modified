import pathlib,sys,subprocess,hashlib,json
base=pathlib.Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage4-9rhcfkw2')
phase=sys.argv[1];root=base/phase
rel='crates/engine-native/src/content/standard.rs' if phase=='before' else 'crates/engine-native-content/src/content/standard.rs'
p=root/rel;original=p.read_bytes()
old='pub fn standard_catalog() -> Catalog {\n    let (cards, skills) = standard_catalog_parts();'
new='pub fn standard_catalog() -> Catalog {\n    let (mut cards, skills) = standard_catalog_parts();\n    cards.shrink_to_fit();'
assert original.decode().count(old)==1
try:
 p.write_text(original.decode().replace(old,new))
 packages=['-p','engine-native'] if phase=='before' else ['-p','engine-native-kernel','-p','engine-native-content','-p','engine-native']
 rc=subprocess.call([sys.executable,str(base/'run.py'),phase+'-warm',str(root),'env','CARGO_TARGET_DIR='+str(base/('target-'+phase)),'CARGO_BUILD_JOBS=8','CARGO_INCREMENTAL=1','/usr/bin/time','-p','cargo','test','--offline',*packages,'--no-run'])
finally:p.write_bytes(original)
E=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-native-split-stage4')
(E/(phase+'-edit.json')).write_text(json.dumps({'path':rel,'old':old,'new':new,'restored':p.read_bytes()==original,'sha256':hashlib.sha256(original).hexdigest()},indent=2)+'\n')
sys.exit(rc)
