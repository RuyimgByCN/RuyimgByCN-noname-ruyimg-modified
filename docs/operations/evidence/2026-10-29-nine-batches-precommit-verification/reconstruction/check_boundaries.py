import subprocess,json,re,hashlib
from pathlib import Path
base='95f51867a3648321da1cc92149b60e2bbfaae748';root=Path(__file__).resolve().parent
commits=subprocess.check_output(['git','rev-list','--reverse',base+'..HEAD']).decode().splitlines();expected=[325,326,326,327,328,328,328,328,328];rows=[]
for i,h in enumerate(commits,1):
 files=set(subprocess.check_output(['git','ls-tree','-r','--name-only',h]).decode().splitlines())
 def text(p):return subprocess.check_output(['git','show',h+':'+p]).decode()
 version=int(re.search(r'pub const STANDARD_RULESET_VERSION: u64 = (\d+)',text('crates/engine-native/src/content/standard.rs'))[1]);assert version==expected[i-1]
 assert 'pub const ARCHIVE_FORMAT_VERSION: u16 = 105;' in text('crates/engine-native/src/archive.rs')
 if i>=2:
  evidence={326:'2026-10-29-nc04-cards-given-v326',327:'2026-10-29-nc07-archive-hash-fields',328:'2026-10-29-adr0786-j1'}[version]
  golden=text('docs/operations/evidence/'+evidence+'/golden-hex-v'+str(version)+'.txt')
  replay=text('crates/engine-native/tests/replay_scenarios.rs')
  for line in golden.splitlines():
   _,name,value=line.split();assert re.search('"'+name+'",[\\s\\S]*?"'+value+'"',replay),name
 assert ('commit_pending: Vec<u8>' in text('crates/engine-native/src/snap/types.rs'))==(i>=5)
 assert ('pub fn restore(&self, catalog: &Catalog)' in text('crates/engine-native/src/snap/restore.rs'))==(i>=8)
 assert ('crates/engine-native/src/content/hooks.rs' in files)==(i>=8)
 assert ('crates/engine-native/src/rvm_bridge/machine.rs' in files)==(i>=9)
 assert ('crates/engine-native/src/engine/hash/machine.rs' in files)==(i>=9)
 assert ('scripts/rust_effective_lines.py' in files)==(i>=9)
 assert ('crates/engine-native/tests/groups/kernel.rs' in files)==(i>=7)
 if i<9:
  assert 'fn push_machine' in text('crates/engine-native/src/engine/hash.rs')
  assert 'impl SkillMachine for VmMachine' in text('crates/engine-native/src/rvm_bridge.rs')
 if i>=5 and i<9:assert 'restore_commit_paused' in text('crates/engine-native/src/rvm_bridge.rs')
 if i==4:assert 'hash/machine.rs' not in text('crates/engine-native/tests/nc07_archive_hash_guard.rs')
 rows.append({'batch':i,'commit':h,'rules':version,'archive':105,'native_abi':753,'boundary_assertions':'passed'})
(root/'boundaries.json').write_text(json.dumps(rows,indent=2));print(json.dumps(rows,indent=2))
