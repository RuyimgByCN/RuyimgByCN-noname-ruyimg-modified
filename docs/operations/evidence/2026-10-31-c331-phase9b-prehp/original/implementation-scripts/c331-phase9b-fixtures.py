from pathlib import Path
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());w=r/'worktree'
p=w/'crates/engine-native/tests/native_kernel/boss_chujiangwang.rs';s=p.read_text();a=s.index('fn boss_chujiangwang_bingfeng_death_flip_source()');s=s[:a]+s[a:].replace('selection: vec![ChoiceOption::Pass],','selection: if c.spec.prompt=="trigger_confirm" {vec![]}else{vec![ChoiceOption::Pass]},',1);# c borrow must end before submit below
s=s.replace('''                let (cid, obs) = (c.id, c.observer);
                submit_cmd(''','''                let (cid, obs) = (c.id, c.observer);
                let selection=if c.spec.prompt=="trigger_confirm" {vec![]}else{vec![ChoiceOption::Pass]};
                submit_cmd(''',1 if False else 0) if False else s
# Capture selection before the mutable submit; limit replacement to the death test.
a=s.index('fn boss_chujiangwang_bingfeng_death_flip_source()');tail=s[a:].replace('''let (cid, obs) = (c.id, c.observer);
                submit_cmd''','''let (cid, obs) = (c.id, c.observer);
                let selection=if c.spec.prompt=="trigger_confirm" {vec![]}else{vec![ChoiceOption::Pass]};
                submit_cmd''',1).replace('''selection: if c.spec.prompt=="trigger_confirm" {vec![]}else{vec![ChoiceOption::Pass]},''','''selection,''',1);s=s[:a]+tail;p.write_text(s)
p=w/'crates/engine-native/tests/native_kernel/dying.rs';s=p.read_text().replace('''        answer(&mut e, ChoiceOption::Pass);
    }
    assert!(!e.debug_alive(0),''','''        let c=e.open_choice().unwrap().clone();
        submit_cmd(&mut e,c.observer,GameAction::Answer{choice_id:c.id,selection:vec![]});
    }
    assert!(!e.debug_alive(0),''',1);p.write_text(s)
p=w/'crates/engine-native/tests/native_kernel/combat.rs';s=p.read_text().replace('''    // 转移后的 1 不掉血；传导以 3 为中心 → 2（仍链）掉血
''','''    // 1 仍横置，因此会再次成为真实 chain root；拒绝该 child 的移伤后再检查 2。
    while let Some(c)=e.open_choice().cloned() {
        if c.spec.prompt!="trigger_confirm" {break;}
        submit_cmd(&mut e,c.observer,GameAction::Answer{choice_id:c.id,selection:vec![]});
    }
    // 转移后的 1 会作为独立传播承受者受伤；以最终目标3的 changeHp 资格传播。
''',1).replace('''    assert_eq!(hp(&e, 1), 4);
    assert_eq!(hp(&e, 3), 3);''','''    assert_eq!(hp(&e, 1), 3);
    assert_eq!(hp(&e, 3), 3);''',1);p.write_text(s)
p=w/'crates/engine-native/tests/nc08_upstream_duwen.rs';s=p.read_text().replace('/../engine-native-kernel/src/kernel/damage.rs','/../engine-native-kernel/src/kernel/damage_windows.rs');p.write_text(s)
p=w/'crates/engine-native-kernel/src/snap/validation.rs';s=p.read_text().replace('''    // 栈帧：唯一性、水位、父子链（父必须存在且是更早的帧）''','''    if resolution.open_choice.is_some() && resolution.parks.is_empty() {
        return Err(oob("choice_without_machine",0));
    }
    // 栈帧：唯一性、水位、父子链（父必须存在且是更早的帧）''',1);p.write_text(s)
p=w/'crates/engine-native/tests/boundary_snapshot/restore.rs';s=p.read_text();a=s.index('fn restore_rejects_tampered_gate_event_payload()');b=s.index('/// N04-b',a);part=s[a:b];part=part.replace('''    let mut patched = false;''','''    let mut invalid=snap.resolution.stack.iter().find_map(|f|match &f.data{EventData::Damage(d)=>Some(d.clone()),_=>None}).unwrap();
    invalid.target=255;
    let mut patched = false;''',1);aa=part.index('''            // 散财确认闸门''');bb=part.index('\n        }',aa);part=part[:aa]+'''            // 新闸门携带 window/root 引用；伪造带越界ID的旧payload仍先过信任边界校验。
            *event=EventData::Damage(invalid.clone());
            patched=true;'''+part[bb:];part=part.replace('挂起的闸门机器应携带 Damage 事件载荷','实际 OptionalGate 应已被篡改');s=s[:a]+part+s[b:];p.write_text(s)
p=w/'crates/engine-native-kernel/src/snap/damage_validation.rs';s=p.read_text().replace('''        O::Zero => {
            matches!(s.progress, P::ZeroPending | P::Stopped) || s.progress.window().is_some()
        }''','''        O::Zero => matches!(s.progress,P::ZeroPending|P::Stopped),''').replace('''        O::Zero => matches!(s.progress, P::ZeroPending | P::Stopped) || s.progress.window().is_some(),''','''        O::Zero => matches!(s.progress,P::ZeroPending|P::Stopped),''');p.write_text(s)
