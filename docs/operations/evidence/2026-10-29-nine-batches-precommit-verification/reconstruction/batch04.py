from rebuild_common import *
p='crates/engine-native/src/engine/hash.rs';s=subprocess.check_output(['git','show','HEAD:'+p]).decode();f=(SNAP/p).read_text()
a=f.index('        // NC07（规则 v327');b=f.index('        push_u8(&mut h, p.paotai_lv);',a);s=s.replace('        push_u8(&mut h, p.paotai_lv);',f[a:b]+'        push_u8(&mut h, p.paotai_lv);')
a=f.index('\n    // NC07（规则 v327')+1;b=f.index('    push_u32(&mut h, c.turn.phase_number);',a);s=s.replace('    push_u32(&mut h, c.turn.phase_number);',f[a:b]+'    push_u32(&mut h, c.turn.phase_number);');Path(p).write_text(s)
for name in ['nc07_archive_hash_fields','nc07_archive_hash_guard']:copy('crates/engine-native/tests/'+name+'.rs')
p='crates/engine-native/tests/nc07_archive_hash_guard.rs';replace(p,'    // 摘要编码分布在 engine/hash.rs（状态）与 engine/hash/machine.rs（机器快照）。\n    let hash = [include_str!("../src/engine/hash.rs"), include_str!("../src/engine/hash/machine.rs")].concat();','    let hash = include_str!("../src/engine/hash.rs");')
evidence(4);version(327)
