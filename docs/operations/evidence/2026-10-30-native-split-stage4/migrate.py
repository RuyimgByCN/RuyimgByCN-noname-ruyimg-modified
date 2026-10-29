from pathlib import Path
import importlib.util,subprocess,json,hashlib,re
E=Path('docs/operations/evidence/2026-10-30-native-split-stage4')
spec=importlib.util.spec_from_file_location('layering','scripts/check-native-layering.py')
assert spec is not None and spec.loader is not None
m=importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
S=Path('crates/engine-native/src');K=Path('crates/engine-native-kernel/src');C=Path('crates/engine-native-content/src')
K.mkdir(parents=True);C.mkdir(parents=True)
moves=[]
for name in (*m.KERNEL_MODULES,'content'):
 destination=C if name=='content' else K
 for old in [S/(name+'.rs'),S/name]:
  if not old.exists():continue
  for file in ([old] if old.is_file() else sorted(old.rglob('*'))):
   if file.is_file():moves.append({'old':str(file),'new':str(destination/file.relative_to(S)),'before_sha256':hashlib.sha256(file.read_bytes()).hexdigest()})
  subprocess.run(['git','mv',str(old),str(destination/old.name)],check=True)
(E/'move-manifest.json').write_text(json.dumps(moves,indent=2)+'\n')
modules=', '.join(m.KERNEL_MODULES)
(K/'lib.rs').write_text('//! Deterministic native rule kernel and shared content vocabulary.\n#![deny(unreachable_patterns)]\n\n'+''.join('pub mod '+name+';\n' for name in m.KERNEL_MODULES)+'\npub use machine::LogEvent;\n')
(C/'lib.rs').write_text('//! Built-in native content and content-specific resolution machines.\n#![deny(unreachable_patterns)]\n\npub use engine_native_kernel::{\n    '+modules+',\n};\npub use engine_native_kernel::LogEvent;\n\npub mod content;\n')
p=S/'lib.rs';s=p.read_text()
for name in (*m.KERNEL_MODULES,'content'):s=s.replace('pub mod '+name+';\n','')
s=s.replace('pub mod ai;', 'pub use engine_native_kernel::{\n    '+modules+',\n};\npub use engine_native_content::content;\n\npub mod ai;')
p.write_text(s)
for name,desc,deps in [
 ('kernel','Deterministic native rule kernel and shared vocabulary','blake3 = { workspace = true }\nrule-api = { workspace = true }\nrule-vm = { workspace = true }'),
 ('content','Built-in native content and resolution machines','engine-native-kernel = { path = "../engine-native-kernel" }\nrule-vm = { workspace = true }')]:
 Path(f'crates/engine-native-{name}/Cargo.toml').write_text('[package]\nname = "engine-native-'+name+'"\nversion.workspace = true\nedition.workspace = true\nlicense.workspace = true\nauthors.workspace = true\ndescription = "'+desc+'"\n\n[dependencies]\n'+deps+'\n')
p=Path('Cargo.toml');s=p.read_text().replace('    "crates/engine-native",','    "crates/engine-native",\n    "crates/engine-native-kernel",\n    "crates/engine-native-content",');p.write_text(s)
p=Path('crates/engine-native/Cargo.toml');s=p.read_text().replace('rule-api = { workspace = true }\nrule-vm = { workspace = true }','engine-native-kernel = { path = "../engine-native-kernel" }\nengine-native-content = { path = "../engine-native-content" }');p.write_text(s)
# Existing facade tests still construct VM modules and use the rule ABI.
p.write_text(p.read_text().replace('[dev-dependencies]\n','[dev-dependencies]\nrule-api = { workspace = true }\nrule-vm = { workspace = true }\n'))
changes={
 'state/general_assembly.rs':[('pub(crate) fn apply_general_assembly','pub fn apply_general_assembly')],
 'state.rs':[('pub(crate) use general_assembly::apply_general_assembly','pub use general_assembly::apply_general_assembly')],
 'kernel/equipment.rs':[('pub(crate) fn grant_equip_skills','pub fn grant_equip_skills'),('pub(crate) fn revoke_equip_skills','pub fn revoke_equip_skills')],
 'kernel.rs':[('pub(crate) use equipment::*;','pub use equipment::{grant_equip_skills, revoke_equip_skills};\nuse equipment::commit_equip;'),('pub(crate) use combat_rules::*;','pub(crate) use combat_rules::*;\npub use combat_rules::tao_futai2_banned;')],
 'kernel/combat_rules.rs':[('pub(crate) fn tao_futai2_banned','pub fn tao_futai2_banned')],
 'kernel/sha.rs':[('pub(crate) fn new','pub fn new')],
 'kernel/duel.rs':[('pub(crate) fn new','pub fn new')],
 'kernel/monkey.rs':[('pub(crate)','pub')],
 'kernel/tricks.rs':[('pub(crate)','pub')],
 'kernel/nullify.rs':[('pub(crate) fn wrap_nullify_window','pub fn wrap_nullify_window'),('impl NoopMachine {\n    pub(crate) fn new','impl NoopMachine {\n    pub fn new')],
 'snap/asks/shapes.rs':[('pub(crate) fn ask(','pub fn ask('),('pub(crate) fn hand_card_options','pub fn hand_card_options'),('pub(crate) fn other_player_options','pub fn other_player_options')],
}
for name,edits in changes.items():
 p=K/name;s=p.read_text()
 for old,new in edits:
  assert old in s,(name,old)
  s=s.replace(old,new)
 p.write_text(s)
# Source-reading consumers retain their original input sets, split across two roots.
for p in Path('scripts').glob('*.py'):
 if p.name in ['check-native-layering.py','test_check_native_layering.py']:continue
 s=p.read_text();t=s
 t=t.replace('HOST.parents[1] / "vocabulary','ROOT / "crates/engine-native-kernel/src/vocabulary')
 # report_registered_deviations has no ROOT yet.
 if 'ROOT / "crates/engine-native-kernel/' in t and 'ROOT =' not in t:
  t=t.replace('HOST = Path(__file__).resolve().parent.parent /', 'ROOT = Path(__file__).resolve().parent.parent\nHOST = ROOT /')
 for name in (*m.KERNEL_MODULES,'content'):
  dest='engine-native-content' if name=='content' else 'engine-native-kernel'
  t=t.replace('crates/engine-native/src/'+name, 'crates/'+dest+'/src/'+name)
  t=t.replace('"crates" / "engine-native" / "src" / "'+name+'"','"crates" / "'+dest+'" / "src" / "'+name+'"')
  t=t.replace('"crates" / "engine-native" / "src" / "'+name+'.rs"','"crates" / "'+dest+'" / "src" / "'+name+'.rs"')
 if t!=s:p.write_text(t)
for p in Path('crates/engine-native/tests').rglob('*.rs'):
 s=p.read_text();t=s
 for name in (*m.KERNEL_MODULES,'content'):
  dest='engine-native-content' if name=='content' else 'engine-native-kernel'
  # Strings beginning with src, /src or ../src (include_str) have distinct anchors.
  t=t.replace('"src/'+name, '"../'+dest+'/src/'+name)
  t=t.replace('"/src/'+name, '"/../'+dest+'/src/'+name)
  t=t.replace('"../src/'+name, '"../../'+dest+'/src/'+name)
 if p.name=='nc08_upstream_yunchou_unimplemented.rs':
  t=t.replace('manifest().join("src")','manifest().join("../engine-native-kernel/src")')
 if t!=s:p.write_text(t)
p=Path('.github/workflows/ci.yml');s=p.read_text().replace('cargo check -p engine-native --all-targets','cargo check -p engine-native-kernel -p engine-native-content -p engine-native --all-targets').replace('cargo test -p engine-native -p engine-native-wasm','cargo test -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm').replace('只限定 engine-native（','限定原生三层（');p.write_text(s)
