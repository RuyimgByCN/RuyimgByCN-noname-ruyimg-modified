from pathlib import Path
for p in Path('scripts').glob('*.py'):
 if p.name in ['check_doc_line_refs.py','test_check_doc_line_refs.py','check-native-layering.py','test_check_native_layering.py']:continue
 t=p.read_text();s=t
 for name in ['keys','behavior','behavior_list','tags','tag_decode','constants','metadata']:
  t=t.replace(f'crates/engine-native/src/content/host/{name}',f'crates/engine-native/src/vocabulary/host/{name}')
 t=t.replace('"src" / "content" / "host" / "keys.rs"','"src" / "vocabulary" / "host" / "keys.rs"')
 # Every remaining HOST-based use of keys refers to the new vocabulary.
 t=t.replace('(HOST / "keys.rs")','(HOST.parents[1] / "vocabulary/host/keys.rs")').replace('KEYS = HOST / "keys.rs"','KEYS = HOST.parents[1] / "vocabulary/host/keys.rs"')
 if p.name=='report_filter_meta_mismatch.py':
  t=t.replace('def meta_events(base: Path = HOST)', 'def meta_events(base: Path = HOST.parents[1] / "vocabulary/host")')
 if p.name=='report_event_arms.py':
  t=t.replace('HOST.rglob("metadata', '(HOST.parents[1] / "vocabulary/host").rglob("metadata').replace('HOST.glob("metadata', '(HOST.parents[1] / "vocabulary/host").glob("metadata').replace('meta_dir = HOST / "metadata"','meta_dir = HOST.parents[1] / "vocabulary/host/metadata"').replace('(HOST / "metadata.rs")','(HOST.parents[1] / "vocabulary/host/metadata.rs")')
 if p.name in ['report_unimplemented_hosts.py','report_registered_deviations.py']:
  t=t.replace('HOST.rglob("*.rs")','list(HOST.rglob("*.rs")) + list((HOST.parents[1] / "vocabulary/host").rglob("*.rs"))')
 if p.name=='host_key_reachability.py':
  t=t.replace('ROOT / "crates/engine-native/src/content/host.rs"','ROOT / "crates/engine-native/src/vocabulary/card_skills.rs"').replace('`content/host.rs` 的 `CardKind','`vocabulary/card_skills.rs` 的 `CardKind')
 if t!=s:p.write_text(t)
# Runtime/compile-time source scanners in tests, preserving docs-only historical citations.
for p in Path('crates/engine-native/tests').rglob('*.rs'):
 t=p.read_text();lines=t.splitlines(keepends=True);out=[]
 for line in lines:
  if not line.lstrip().startswith('//'):
   for name in ['behavior','behavior_list','keys','tags','tag_decode','constants','metadata']:
    line=line.replace(f'/src/content/host/{name}',f'/src/vocabulary/host/{name}').replace(f'"content/host/{name}',f'"vocabulary/host/{name}')
   line=line.replace('/src/content/standard/names.rs','/src/vocabulary/names.rs')
  out.append(line)
 t=''.join(out)
 if p.name=='nc08_upstream_yunchou_guihan.rs':
  t=t.replace('src.join("keys.rs")','manifest().join("src/vocabulary/host/keys.rs")')
 if p.name=='nc08_registered_defect_guards.rs':
  # read() routes metadata guards to the definition's new home.
  t=t.replace('let path = host_dir().join(rel);','let root = if rel.starts_with("metadata") {\n        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/vocabulary/host")\n    } else {\n        host_dir()\n    };\n    let path = root.join(rel);')
  # First recursive scan is for metadata only; the later effect scan stays content.
  t=t.replace('let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/content/host");','let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/vocabulary/host");',1)
 if t!=p.read_text():p.write_text(t)
# Imports formerly used by metadata.
p=Path('crates/engine-native/src/content/host.rs');t=p.read_text().replace('use crate::catalog::TriggerMeta;\n','').replace('use crate::event::{EventKind, Timing};\n','');p.write_text(t)
p=Path('crates/engine-native/src/content/standard.rs');t=p.read_text().replace('CardDef, CardKind, Catalog, Faction, QueryCtx','CardDef, CardKind, Catalog, QueryCtx');p.write_text(t)
