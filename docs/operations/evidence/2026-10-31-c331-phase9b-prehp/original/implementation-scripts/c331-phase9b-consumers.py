from pathlib import Path
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());w=r/'worktree';p=w/'crates/engine-native-kernel/src/machine.rs';s=p.read_text();a=s.index('    /// 当前帧的伤害载荷只读访问');s=s[:a]+'''    /// Damage handlers read their checked root payload while their queue retains the child frame.
    pub fn current_event_frame_id(&self)->crate::ids::EventId {
        let id=self.current_frame_id();
        match &self.res.frame(id).expect("current frame").data {
            EventData::DamageWindow{root,..}=>*root,
            _=>id,
        }
    }

'''+s[a:];a=s.index('        let id = self.current_frame_id();',s.index('    pub fn current_damage(&self)'));b=s.index('        match self.res.frame(root)',a);s=s[:a]+'''        let root=self.current_event_frame_id();
'''+s[b:];a=s.index('        let id = self.current_frame_id();',s.index('    pub fn current_damage_mut(&mut self)'));b=s.index('        self.res',a);s=s[:a]+'''        let id=self.current_event_frame_id();
'''+s[b:];p.write_text(s)
# All these callers inspect or mutate the current event payload. Queue/park/source identity keeps current_frame_id.
for p in (w/'crates/engine-native-content/src/content/host/step').glob('*.rs'):
 s=p.read_text()
 if 'ctx.current_frame_id()'in s:s=s.replace('ctx.current_frame_id()','ctx.current_event_frame_id()');p.write_text(s)
p=w/'crates/engine-native-content/src/content/host/step/swd_damage_reactions.rs';s=p.read_text();s=s.replace('''                        let stage = ctx.res.frame(ctx.current_event_frame_id()).expect("frame").stage;
                        if stage == crate::event::Stage::AfterTriggers {''','''                        let after = !ctx.current_damage().expect("damage root").settlement.progress.is_pre_hp();
                        if after {''');p.write_text(s)
p=w/'crates/engine-native-kernel/src/kernel/damage_flow.rs';s=p.read_text().replace('''            if d.prevented {
                d.settlement.outcome''','''            if d.prevented {
                ctx.log(LogEvent::DamagePrevented{target:d.target});
                d.settlement.outcome''',1);p.write_text(s)
p=w/'crates/engine-native-kernel/src/kernel/damage_windows.rs';s=p.read_text();s=s.replace('''                        ctx.recover(d.target, 1, None);
                        finish_damage(ctx, root, d, crate::event::DamageOutcome::Cancelled);''','''                        finish_damage(ctx, root, d.clone(), crate::event::DamageOutcome::Cancelled);
                        ctx.recover(d.target, 1, None);''');p.write_text(s)
p=w/'crates/engine-native/tests/native_kernel/boss_xuanwu.rs';s=p.read_text().replace('''assert_eq!(r.state_core().player(1).lingqu_count, 3, "灵渠计数入存档");''','''assert_eq!(r.state_core().player(1).lingqu_count, 1, "仅正数 damageEnd 的灵渠计数入存档");''');p.write_text(s)
