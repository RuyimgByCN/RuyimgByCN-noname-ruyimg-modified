from pathlib import Path
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());w=r/'worktree';p=w/'crates/engine-native-kernel/src/event/damage.rs';s=p.read_text();s=s.replace('''    #[default]
    Before,
    HpCommitted,
    TargetDamage,
    DyingPending,
    DyingDone,
    Source,
    End,
    After,''','''    #[default]
    Start,
    Before,
    Begin,
    Begin1,
    Begin2,
    Begin3,
    Begin4,
    SubmitHp,
    HpCommitted { chain_eligible: bool },
    TargetDamage { chain_eligible: bool },
    DyingPending { chain_eligible: bool },
    DyingDone { chain_eligible: bool },
    Source { chain_eligible: bool },
    End { chain_eligible: bool },
    After { chain_eligible: bool },''');s=s.replace('''    ZeroPending,
    Stopped,''','''    ZeroPending,
    CancelledPending,
    Stopped,''');s=s.replace('''pub enum DamageWindowKind {
    Source,''','''pub enum DamageWindowKind {
    Before,
    Begin,
    Begin1,
    Begin2,
    Begin3,
    Begin4,
    Target,
    Source,''');s=s.replace('''        match self {
            Self::Source => EventKind::DamageSource,''','''        match self {
            Self::Before => EventKind::DamageBefore,
            Self::Begin => EventKind::DamageBegin,
            Self::Begin1 => EventKind::DamageBegin1,
            Self::Begin2 => EventKind::DamageBegin2,
            Self::Begin3 => EventKind::DamageBegin3,
            Self::Begin4 => EventKind::DamageBegin4,
            Self::Target => EventKind::Damage,
            Self::Source => EventKind::DamageSource,''',1);s=s.replace('''            Self::Cancelled => 4,''','''            Self::Cancelled => 4,
            Self::Before => 5,
            Self::Begin => 6,
            Self::Begin1 => 7,
            Self::Begin2 => 8,
            Self::Begin3 => 9,
            Self::Begin4 => 10,
            Self::Target => 11,''',1).replace('''            4 => Self::Cancelled,''','''            4 => Self::Cancelled,
            5 => Self::Before,
            6 => Self::Begin,
            7 => Self::Begin1,
            8 => Self::Begin2,
            9 => Self::Begin3,
            10 => Self::Begin4,
            11 => Self::Target,''',1)
a=s.index('''        let (tag, remaining) = match &self.progress''');b=s.index('''        out
    }
}''',a);s=s[:a]+'''        out.extend(self.progress.canonical_bytes());
'''+s[b:];s+='''
impl DamageProgress {
    pub fn tag(&self) -> u8 {
        match self {
            Self::Before=>0,Self::HpCommitted{..}=>1,Self::TargetDamage{..}=>2,
            Self::DyingPending{..}=>3,Self::DyingDone{..}=>4,Self::Source{..}=>5,
            Self::End{..}=>6,Self::After{..}=>7,Self::Chain{..}=>8,Self::Done=>9,
            Self::ZeroPending=>10,Self::Stopped=>11,Self::Start=>12,Self::Begin=>13,
            Self::Begin1=>14,Self::Begin2=>15,Self::Begin3=>16,Self::Begin4=>17,
            Self::SubmitHp=>18,Self::CancelledPending=>19,
        }
    }
    pub fn chain_eligible(&self)->Option<bool> {
        match self {
            Self::HpCommitted{chain_eligible}|Self::TargetDamage{chain_eligible}|
            Self::DyingPending{chain_eligible}|Self::DyingDone{chain_eligible}|
            Self::Source{chain_eligible}|Self::End{chain_eligible}|Self::After{chain_eligible}=>Some(*chain_eligible),
            _=>None,
        }
    }
    pub fn canonical_bytes(&self)->Vec<u8> {
        let mut out=vec![self.tag()];
        if let Some(eligible)=self.chain_eligible(){out.push(u8::from(eligible));}
        if let Self::Chain{remaining}=self {
            out.extend_from_slice(&(remaining.len() as u32).to_le_bytes());out.extend_from_slice(remaining);
        }
        out
    }
    pub fn is_pre_hp(&self)->bool {
        matches!(self,Self::Start|Self::Before|Self::Begin|Self::Begin1|Self::Begin2|Self::Begin3|Self::Begin4|Self::SubmitHp)
    }
    pub fn window(&self)->Option<DamageWindowKind> {
        use DamageWindowKind as W;
        Some(match self {
            Self::Before=>W::Before,Self::Begin=>W::Begin,Self::Begin1=>W::Begin1,
            Self::Begin2=>W::Begin2,Self::Begin3=>W::Begin3,Self::Begin4=>W::Begin4,
            Self::TargetDamage{..}=>W::Target,Self::Source{..}=>W::Source,
            Self::End{..}=>W::End,Self::After{..}=>W::After,
            _=>return None,
        })
    }
}
''';p.write_text(s)
p=w/'crates/engine-native-kernel/src/event.rs';s=p.read_text().replace('''    DamageCancelled,
}''','''    DamageCancelled,
    DamageBegin,
    DamageBegin1,
    DamageBegin2,
    DamageBegin3,
    DamageBegin4,
}''',1);p.write_text(s)
p=w/'crates/engine-native-kernel/src/archive/values.rs';s=p.read_text().replace('''DamageCancelled => 27,''','''DamageCancelled => 27,
        DamageBegin => 28, DamageBegin1 => 29, DamageBegin2 => 30, DamageBegin3 => 31, DamageBegin4 => 32,''').replace('''27 => DamageCancelled,''','''27 => DamageCancelled,
        28 => DamageBegin,29=>DamageBegin1,30=>DamageBegin2,31=>DamageBegin3,32=>DamageBegin4,''');p.write_text(s)
p=w/'crates/engine-native-kernel/src/archive/damage.rs';s=p.read_text();a=s.index('    let (tag, remaining)');b=s.index('\n}',a);s=s[:a]+'''    w.u8(s.progress.tag());
    if let Some(eligible)=s.progress.chain_eligible(){w.boolean(eligible);}
    if let P::Chain{remaining}=&s.progress {w.len(remaining.len());for p in remaining {w.u8(*p);}}
'''+s[b:];
for variant in ['HpCommitted','TargetDamage','DyingPending','DyingDone','Source','End','After']:
 s=s.replace('=> P::'+variant+',','=> P::'+variant+'{chain_eligible:r.boolean()?},')
s=s.replace('''        11 => P::Stopped,''','''        11 => P::Stopped,
        12=>P::Start,13=>P::Begin,14=>P::Begin1,15=>P::Begin2,16=>P::Begin3,17=>P::Begin4,18=>P::SubmitHp,19=>P::CancelledPending,''');p.write_text(s)
# Keep each source-qualified native pre-window separate; no shadow amount survives a yield.
p=w/'crates/engine-native-kernel/src/kernel/damage.rs';s=p.read_text();start=s.index('    // 绝情');hp=s.index('    let submitted = amount as u32;');chunks={}
markers=['    // 绝情','    // 招祸','    // 火神','    // 拟态（Boss 梼杌，规则 v217）：锁定——防止','    let mut amount = i64::from(d.amount);','    // 赤龙牙','    // 回合伤害统计','    // 灵渠（Boss 玄武','    // 脉轮','    let Ok(amount)=i32::try_from(amount)']
idx=[s.index(x)for x in markers];chunks['before']=s[idx[0]:idx[1]];chunks['begin4cancel']=s[idx[1]:idx[2]]+s[idx[3]:idx[4]];chunks['begin1cancel']=s[idx[2]:idx[3]];chunks['begin1mod']=s[idx[4]+len(markers[4]):idx[5]]+s[idx[6]:idx[7]];chunks['beginmod']=s[idx[5]:idx[6]]+s[idx[8]:idx[9]];chunks['begin4mod']=s[idx[7]:idx[8]]
pre='''//! Source-qualified native damage transitions; ordinary window queues follow each commit.
use super::*;
use crate::event::DamageWindowKind as Window;
pub(super) fn commit_damage_window(ctx:&mut Ctx<'_>, root:EventId, window:Window) {
    let mut d=ctx.res.frame(root).expect("damage root").damage().clone();
    if d.prevented || d.amount<=0 || d.settlement.outcome!=crate::event::DamageOutcome::Running || d.settlement.notrigger {return;}
    let mut amount=i64::from(d.amount);
    match window {
'''
for name,body in [('Before',chunks['before']),('Begin',chunks['beginmod']),('Begin1',chunks['begin1cancel']+chunks['begin1mod']),('Begin4',chunks['begin4cancel']+chunks['begin4mod'])]:
 body=body.replace('frame_id','root');body=body.replace('''                ctx.emit(EventData::LoseHp {
                    player: d.target,
                    amount: d.amount as u32,
                });
                finish_damage(ctx, root, d, crate::event::DamageOutcome::ReplacedByLoseHp);''','''                finish_damage(ctx, root, d.clone(), crate::event::DamageOutcome::ReplacedByLoseHp);
                ctx.emit(EventData::LoseHp {player:d.target,amount:d.amount as u32});''').replace('''            ctx.emit(EventData::LoseHp {
                player: d.target,
                amount: 1,
            });
            finish_damage(ctx, root, d, crate::event::DamageOutcome::ReplacedByLoseHp);''','''            finish_damage(ctx, root, d.clone(), crate::event::DamageOutcome::ReplacedByLoseHp);
            ctx.emit(EventData::LoseHp {player:d.target,amount:1});''')
 pre+='        Window::'+name+'=>{\n'+body+'\n},\n'
pre+='''        _=>return,
    }
    let Ok(amount)=i32::try_from(amount) else {
        ctx.res.fault=Some(crate::machine::VmFault{skill:"intrinsic_damage".into(),owner:d.target,event:"Damage".into(),kind:"DamageAmountOverflow".into(),detail:"canonical source amount exceeds i32".into()});return;
    };
    d.amount=amount;ctx.res.frame_mut(root).unwrap().data=EventData::Damage(d);
}
''';(p.parent/'damage_windows.rs').write_text(pre)
s=s[:start]+'''    let amount=d.amount;
'''+s[hp:];s=s.replace('''d.settlement.progress = crate::event::DamageProgress::HpCommitted;''','''let chain_eligible = d.nature != crate::event::DamageNature::Normal && !d.from_chain && ctx.core.player(d.target).linked;
    d.settlement.progress = crate::event::DamageProgress::HpCommitted { chain_eligible };''',1)
# Unlink only the damaged recipient during its own HP transition, even a chained recipient.
needle='''    d.settlement.hp_loss = if p.infinite_hp() { 0 } else { hp_loss };''';s=s.replace(needle,needle+'''
    if d.nature != crate::event::DamageNature::Normal && ctx.core.player(d.target).linked {
        ctx.core.player_mut(d.target).linked=false;
        ctx.log(LogEvent::Linked{player:d.target,linked:false});
    }
''',1);p.write_text(s)
p=w/'crates/engine-native-kernel/src/kernel.rs';s=p.read_text();a=s.index('        EventData::Damage(d) =>');b=s.index('        EventData::LoseHp',a);s=s[:a]+'''        EventData::DamageWindow { root, window } => {
            damage_windows::commit_damage_window(ctx,root,window);
            let d=ctx.res.frame(root).expect("damage root").damage().clone();
            if window==crate::event::DamageWindowKind::End && d.amount>0 && !d.prevented && !d.settlement.notrigger && d.settlement.outcome==crate::event::DamageOutcome::Running {post_damage(ctx,d);}
        },
'''+s[b:];s=s.replace('mod damage_flow;','mod damage_windows;\nmod damage_flow;',1);p.write_text(s)
p=w/'crates/engine-native-kernel/src/resolution.rs';s=p.read_text().replace('''    let stage = match kind {
        EventKind::Damage => Stage::BeforeTriggers,''','''    let stage = match &data {
        EventData::Damage(_) => Stage::Intrinsic,
        _ => match kind {''',1).replace('''        _ => Stage::Commit,
    };
    res.stack.push''','''        _ => Stage::Commit,
        },
    };
    res.stack.push''',1).replace('''        (EventKind::Damage, Stage::AfterTriggers) => Stage::Intrinsic,''','''        (EventKind::Damage, Stage::AfterTriggers) => Stage::Done,''',1);p.write_text(s)
