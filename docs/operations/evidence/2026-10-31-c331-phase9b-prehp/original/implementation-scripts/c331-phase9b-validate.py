from pathlib import Path
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());w=r/'worktree';p=w/'crates/engine-native-kernel/src/kernel/damage.rs';s=p.read_text();a=s.index('fn finish_damage');print(s[a:a+550]);s=s.replace('''d.settlement.progress = if outcome == crate::event::DamageOutcome::Zero { crate::event::DamageProgress::ZeroPending } else { crate::event::DamageProgress::Stopped };''','''// The active window/child keeps its progress until the root resumes.
''');p.write_text(s)
p=w/'crates/engine-native-kernel/src/kernel/damage_flow.rs';s=p.read_text().replace('''        let mut child = None;''','''        if d.settlement.outcome==O::Cancelled && !matches!(d.settlement.progress,P::Stopped|P::CancelledPending) {
            d.settlement.progress=P::CancelledPending;
        }
        if d.settlement.outcome==O::ReplacedByLoseHp {d.settlement.progress=P::Stopped;}
        let mut child = None;''',1);p.write_text(s)
p=w/'crates/engine-native-kernel/src/snap/validation.rs';s=p.read_text();a=s.index('            if d.settlement.committed_damage ==');b=s.index('            check_opt_pid(d.source',a);s=s[:a]+'''            super::damage_validation::check_settlement(d,b)?;
'''+s[b:];a=s.index('        if let EventData::DamageWindow { root, window } = &f.data {',s.index('pub fn validate_resolution'));b=s.index('        for q in &f.queue',a);s=s[:a]+'''        super::damage_validation::check_frame(f,resolution)?;
'''+s[b:];p.write_text(s)
p=w/'crates/engine-native-kernel/src/snap.rs';s=p.read_text();s+='\nmod damage_validation;\n';p.write_text(s)
