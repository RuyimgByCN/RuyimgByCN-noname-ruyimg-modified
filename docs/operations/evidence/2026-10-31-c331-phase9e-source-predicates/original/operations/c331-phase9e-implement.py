from pathlib import Path
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());w=r/'worktree'
def update(p,fn):q=w/p;q.write_text(fn(q.read_text()))
k='crates/engine-native-kernel/src/'
p=k+'event/native_damage.rs'
def effects(s):
 s=s.replace('            | Self::Ciqiu\n','')
 s=s.replace('Self::Zuijiu | Self::Huoshen | Self::NitaiOut | Self::ZhaohuoSource =>','Self::Ciqiu | Self::Zuijiu | Self::Huoshen | Self::NitaiOut | Self::ZhaohuoSource =>')
 s=s.replace("pub fn matches(self, owner: PlayerId, d: &DamageData, ctx: &QueryCtx<'_>) -> bool {", "pub fn matches(self, owner: PlayerId, d: &DamageData, ctx: &QueryCtx<'_>, parent: Option<&Frame>) -> bool {")
 s=s.replace('let sha = d.card.is_some_and(|c| catalog.card_kind(c).is_sha_like());','''let sha = d.card.is_some_and(|card| {
            let kind=parent.and_then(|p|match &p.data {
                EventData::CardUsed(u) if u.card==card => u.use_as,
                _ => None,
            }).unwrap_or_else(||catalog.card_kind(card));
            kind.is_sha_like()
        });''')
 s=s.replace('Self::Zuijiu => source && sha,','''Self::Zuijiu => source && sha && parent.is_some_and(|p| {
                p.stage==Stage::Intrinsic && matches!(&p.data, EventData::CardUsed(u)
                    if !u.cancelled && u.use_as.unwrap_or_else(||catalog.card_kind(u.card)).is_sha_like())
            }),''')
 s=s.replace('Self::Chilongya => source && fire && equip(CardKind::Chilongya),','Self::Chilongya => source && fire && !d.from_chain && equip(CardKind::Chilongya),')
 return s
update(p,effects)
for p in ['crates/engine-native-content/src/content/standard/native_damage.rs','crates/engine-native-content/src/content/host.rs']:
 update(p,lambda s:s.replace('effect.matches(owner,d,ctx)','effect.matches(owner,d,ctx,None)'))
p=k+'kernel/damage_windows.rs'
def windows(s):
 start=s.index('            if effect == crate::event::NativeDamageEffect::Ciqiu {');end=s.index('            if effect == crate::event::NativeDamageEffect::Duwen',start)
 s=s[:start]+s[end:]
 body='''            if effect == crate::event::NativeDamageEffect::Ciqiu {
                let source=d.source.expect("admitted source role");
                amount += 1;
                if amount>=i64::from(ctx.core.player(d.target).hp) {
                    ctx.core.turn.ciqiu_dying.push(d.target);
                    let def=ctx.catalog.skill_by_name(crate::vocabulary::host::WUDIAN_CIQIU_KEY).expect("immutable Ciqiu definition");
                    ctx.core.skills.remove(&(source,def));
                }
            }
'''
 s=s.replace('        Window::Begin1 => {\n','        Window::Begin1 => {\n'+body)
 # The admitted source unit already checked the Sha family and exact equipment state.
 s=s.replace('if d.nature == crate::event::DamageNature::Fire {','if d.nature == crate::event::DamageNature::Fire && !d.from_chain {',1)
 old='''                    let hold = ctx.core.player(d.target).hutian_hold.len() as i32;
                    if hold > 0 {
                        let hp = ctx.core.player(d.target).hp;
                        let cap = hp - hold;
                        if i64::from(cap) < amount {
                            amount = i64::from(cap.max(0));
                        }
                    }'''
 new='''                    let hold = ctx.core.player(d.target).hutian_hold.len();
                    if hold > 0 {
                        let Ok(hold)=i64::try_from(hold) else {ctx.fail_damage_amount();return;};
                        let cap=i64::from(ctx.core.player(d.target).hp)-hold;
                        amount=amount.min(cap);
                    }'''
 assert old in s;s=s.replace(old,new);return s
update(p,windows)
p=k+'kernel/damage.rs'
def post(s):
 a=s.index('    if effect == crate::event::NativeDamageEffect::Xixue {');b=s.index('    if effect == crate::event::NativeDamageEffect::Qinglong',a)
 s=s[:a]+'''    if effect == crate::event::NativeDamageEffect::Xixue {
        let source=d.source.expect("admitted source role");
        ctx.recover(source,d.amount as u32,None);
    }
'''+s[b:];return s
update(p,post)
# Shared eligibility derives the live canonical root and its actual immediate parent.
p=k+'resolution/trigger_order.rs'
def eligibility(s):
 s=s.replace('''    event: &EventData,
    kind: EventKind,
) -> bool {
    if let EventData::Damage(d) = event {''','''    frames: &[Frame],
    frame: &Frame,
) -> bool {
    let event=crate::event::damage_event(frames,frame);
    let kind=event_kind(frame);
    if let EventData::Damage(d) = event {''',1)
 old='''        && def.behavior.filter_in_window(
            &QueryCtx {
                state: core,
                catalog,
            },
            q.owner,
            event,
            kind,
        )'''
 new='''        && if let (Some(effect),EventData::Damage(d))=(def.behavior.native_damage_effect(kind),event) {
            let root=match frame.data {EventData::DamageWindow{root,..}=>frames.iter().find(|f|f.id==root),EventData::Damage(_)=>Some(frame),_=>None};
            let parent=root.and_then(|root|root.parent.and_then(|id|{
                let index=frames.iter().position(|f|f.id==root.id)?;
                index.checked_sub(1).and_then(|i|frames.get(i)).filter(|p|p.id==id)
            }));
            effect.matches(q.owner,d,&QueryCtx{state:core,catalog},parent)
        } else {
            def.behavior.filter_in_window(&QueryCtx{state:core,catalog},q.owner,event,kind)
        }'''
 assert old in s;s=s.replace(old,new)
 s=s.replace('''                crate::event::damage_event(frames, frame),
                event_kind(frame),''','''                frames,
                frame,''')
 s=s.replace('eligible(core, catalog, &q, payload, event_kind(f))','eligible(core, catalog, &q, &res.stack, f)')
 s=s.replace('''            let payload = crate::event::damage_event(&res.stack, f);
''','')
 s=s.replace('''                    crate::event::damage_event(&res.stack, frame),
                    event_kind(frame),''','''                    &res.stack,
                    frame,''')
 return s
update(p,eligibility)
p=k+'resolution.rs'
def callers(s):
 s=s.replace('''            crate::event::damage_event(&ctx.res.stack, frame),
            trigger_order::event_kind(frame),''','''            &ctx.res.stack,
            frame,''')
 s=s.replace('''        &actual_event,
        trigger_order::event_kind(res.frame(frame_id).expect("frame")),''','''        &res.stack,
        res.frame(frame_id).expect("frame"),''')
 s=s.replace('''        let event = crate::event::damage_event(&res.stack, frame);
''','')
 s=s.replace('trigger_order::eligible(core, catalog, &q, event, kind)','trigger_order::eligible(core, catalog, &q, &res.stack, frame)')
 return s
update(p,callers)
# Cause is already persisted as Frame.parent + from_chain; bind them instead of adding metadata.
p=k+'snap/damage_validation.rs'
def validation(s):
 old='''    if let EventData::Damage(d) = &f.data {
        if !matches!(f.stage, Stage::Intrinsic | Stage::Done)'''
 new='''    if let EventData::Damage(d) = &f.data {
        let index=res.stack.iter().position(|p|p.id==f.id).expect("current frame");
        let immediate=index.checked_sub(1).and_then(|i|res.stack.get(i));
        if f.parent!=immediate.map(|p|p.id) {
            return Err(SnapshotError::BadTag("damage_immediate_parent",0));
        }
        let chain=immediate.is_some_and(|p|matches!(&p.data,EventData::Damage(parent)
            if p.stage==Stage::Intrinsic && matches!(parent.settlement.progress,P::Chain{..})));
        if d.from_chain!=chain {
            return Err(SnapshotError::BadTag("damage_chain_cause",0));
        }
        if !matches!(f.stage, Stage::Intrinsic | Stage::Done)'''
 assert old in s;s=s.replace(old,new);return s
update(p,validation)
print('Updated finite predicates, genuine parent lookup and restore cause binding')
