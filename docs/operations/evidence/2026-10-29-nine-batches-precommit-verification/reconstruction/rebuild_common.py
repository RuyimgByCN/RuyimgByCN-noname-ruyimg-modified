from pathlib import Path
import json,re,shutil,subprocess
SNAP=Path('../final-files')
EVIDENCE=['2026-10-28-b8-gate1b','2026-10-29-nc04-cards-given-v326','2026-10-29-gt014-import-deep-validation','2026-10-29-nc07-archive-hash-fields','2026-10-29-adr0786-j1','2026-10-29-nc08-core-pack-statistics','2026-10-29-o6-test-grouping','2026-10-29-native-split-stage1-2','2026-10-29-rust-effective-1000']
def copy(p):
 t=Path(p);t.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(SNAP/p,t);t.touch()
def evidence(n):
 p='docs/operations/evidence/'+EVIDENCE[n-1];shutil.copytree(SNAP/p,p,dirs_exist_ok=True)
def replace(p,old,new):
 t=Path(p);s=t.read_text();assert s.count(old)==1,(p,old,s.count(old));t.write_text(s.replace(old,new))
def version(v):
 p='crates/engine-native/src/content/standard.rs';s=Path(p).read_text();f=(SNAP/p).read_text();intro=f[f.index('/// v326：'):f.index('pub const STANDARD_RULESET_VERSION:')]
 if v<328:intro=intro[:intro.index('/// v'+str(v+1)+'：')]
 start=s.find('/// v326：');start=start if start>=0 else s.index('pub const STANDARD_RULESET_VERSION:')
 end=s.index(';',s.index('pub const STANDARD_RULESET_VERSION:'))+1
 Path(p).write_text(s[:start]+intro+f'pub const STANDARD_RULESET_VERSION: u64 = {v};'+s[end:])
 for name in ['nc04_vm_resume','nc07_lianyaohu_hash','pa12_tag656_version','nc04_cards_given_event','nc07_archive_hash_fields']:
  t=Path('crates/engine-native/tests/'+name+'.rs')
  if not t.exists():continue
  s=t.read_text();s=re.sub(r'catalog == (325|326|327|328)',f'catalog == {v}',s);s=re.sub(r'catalog: (325|326|327|328)',f'catalog: {v}',s);s=re.sub(r'\^ (325|326|327|328)',f'^ {v}',s);s=re.sub(r'(standard_catalog\(\)\.ruleset_version, )(325|326|327|328)',rf'\g<1>{v}',s);s=re.sub(r'(assert_eq!\(snapshot.ruleset_version, )(325|326|327|328)(\);)',rf'\g<1>{v}\3',s);t.write_text(s)
 p='crates/engine-native/tests/replay_scenarios.rs';s=Path(p).read_text();f=(SNAP/p).read_text()
 for line in (SNAP/'docs/operations/evidence'/EVIDENCE[{326:1,327:3,328:4}[v]]/f'golden-hex-v{v}.txt').read_text().splitlines():
  _,name,h=line.split();pat=r'("'+name+r'",[\s\S]*?)"[a-f0-9]{64}"';s,c=re.subn(pat,lambda m:m[1]+'"'+h+'"',s,count=1);assert c==1,name
 marker='// ADR 0786 J1 v328';a=f.index(marker);b=f.index('// NC04 cards_given（2026-10-28）');new=f[a:b]
 if v<328:new=new[new.index('// NC07 字段批 v327'):] if v==327 else new[new.index('// NC04 cards_given v326'):]
 a=s.find(marker)
 if a<0:a=s.find('// NC07 字段批 v327')
 if a<0:a=s.find('// NC04 cards_given v326')
 if a<0:a=s.index('// NC04 cards_given（2026-10-28）')
 b=s.index('// NC04 cards_given（2026-10-28）');s=s[:a]+new+s[b:];Path(p).write_text(s)
