from pathlib import Path
import re,json,hashlib,tarfile,subprocess,datetime
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());w=r/'worktree';ev=r/'evidence/phase9d'
def read(p):return (w/p).read_text()
def put(p,s):(w/p).write_text(s)
base='crates/engine-native-kernel/src/'
p=base+'event/damage.rs';s=read(p);s=s[:s.index('/// Existing native declarations')];put(p,s)
p=base+'event.rs';s=read(p).replace('pub use damage::*;','pub use damage::*;\nmod native_damage;\npub use native_damage::*;');put(p,s)
# Each entry is a genuine catalog-owned or pinned helper source unit, not a provider registry.
groups={'Before':['Jueqing','Wuxin','Jidong','Chengxin'],'Begin':['Chilongya','Xingjunyan','Liangou','Feiren','Ciqiu','Duwen','MailunOut3','MailunIn3','MailunOut4','MailunIn4','Hutian'],'Begin1':['Zuijiu','Huoshen','NitaiOut'],'Begin4':['Zhaohuo','NitaiIn','Bufo','LingquCancel'],'End':['Shuchui','Paotai','Xixue','Qinglong','Yinzei','Zhangwu','Zlfanshi','Zhue','Abi','Yuance','Lingqu','Jiding','Renxing','Cuidui','Duqu'],'After':['Huanxia','Jidian','Minwan']}
helpers={'Jidong':'ow/jidong2','Chengxin':'xianjian/chengxin2','Chilongya':'card-swd/chilongya','Xingjunyan':'card-swd/xingjunyan','Liangou':'ow/liangou2','MailunOut3':'swd/mailun31','MailunIn3':'swd/mailun32','MailunOut4':'swd/mailun41','MailunIn4':'swd/mailun42','Hutian':'swd/hutian2','Shuchui':'card-swd/shuchui2','Paotai':'ow/paotai3','Xixue':'card-swd/xixue','Qinglong':'card-swd/qinglonglingzhu','Huanxia':'swd/huanxia_expire'}
variants=sum(groups.values(),[])
text='''//! Finite identities of actual native damage source units in the locked catalog.
use super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, TriggerMeta};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeDamageEffect { '''+', '.join(variants)+''' }
impl NativeDamageEffect {
    pub fn window(self) -> DamageWindowKind { match self {\n'''
for kind,vs in groups.items():text+='        '+ '|'.join('Self::'+v for v in vs)+' => DamageWindowKind::'+kind+',\n'
text+='''    } }
    pub fn helper_name(self) -> Option<&'static str> { match self {\n'''
for k,v in helpers.items():text+=f'        Self::{k} => Some("native:liuli/{v}"),\n'
text+='''        _ => None,
    } }
    pub fn meta(self) -> TriggerMeta {
        TriggerMeta { event: self.window().event_kind(), timing: Timing::Normal,
            priority: match self { Self::Wuxin => 10, Self::Hutian => -55, Self::LingquCancel => -11, _ => 0 },
            order_selectable: !matches!(self, Self::Liangou | Self::Shuchui | Self::Huanxia),
            optional: false, extra_events: vec![] }
    }
    pub fn from_host(host: crate::vocabulary::host::HostBehavior) -> Option<Self> {
        use crate::vocabulary::host::HostBehavior as H;
        match host { H::WudianFeiren => Some(Self::Feiren), H::WudianCiqiu => Some(Self::Ciqiu), H::WudianDuwen => Some(Self::Duwen), _ => None }
    }
    pub fn matches(self, owner: PlayerId, d: &DamageData, ctx: &QueryCtx<'_>) -> bool {
        let core=ctx.state; let catalog=ctx.catalog; let p=core.player(owner);
        let source = d.source == Some(owner); let target = d.target == owner;
        let fire = d.nature == DamageNature::Fire;
        let sha = d.card.is_some_and(|c| catalog.card_kind(c).is_sha_like());
        let equip=|kind|p.equips.iter().any(|c|catalog.card_kind(*c)==kind);
        if !p.alive { return false; }
        match self {
            Self::Jueqing => source && !target,
            Self::Wuxin => target,
            Self::Zuijiu => source && sha,
            Self::Jidong => target && p.jidong_active,
            Self::Chengxin => target && p.chengxin_active,
            Self::Chilongya => source && fire && equip(CardKind::Chilongya),
            Self::Xingjunyan => (source || target) && sha && equip(CardKind::Xingjunyan),
            Self::Liangou => target && core.turn.liangou_hook == Some(owner) && !core.turn.liangou_bonus_used,
            Self::Feiren => source && sha && !d.from_chain && d.card.is_some_and(|c|catalog.cards[c as usize].suit==crate::catalog::Suit::Spade),
            Self::Ciqiu => source && sha && core.player(d.target).hp==core.player(d.target).max_hp as i32,
            Self::Duwen => source && p.hand.len()==core.player(d.target).hand.len(),
            Self::MailunOut3 => source && p.mailun_mode==3 && !p.mailun_out_used,
            Self::MailunOut4 => source && p.mailun_mode==4 && !p.mailun_out_used,
            Self::MailunIn3 => target && p.mailun_mode==3 && !p.mailun_in_used,
            Self::MailunIn4 => target && p.mailun_mode==4 && !p.mailun_in_used,
            Self::Hutian => target && !p.hutian_hold.is_empty() && i64::from(p.hp)-i64::from(d.amount)<p.hutian_hold.len() as i64,
            Self::Huoshen => target && fire,
            Self::NitaiOut => target && fire && core.turn.current != owner,
            Self::NitaiIn => target && core.turn.current == owner,
            Self::Zhaohuo => target && fire,
            Self::Bufo | Self::LingquCancel => target && d.amount>1,
            Self::Shuchui => source && core.turn.current==owner && core.turn.shuchui_armed && sha,
            Self::Paotai => target && p.paotai_lv>0,
            Self::Xixue => source && !target && sha && p.hp<p.max_hp as i32 && equip(CardKind::Xixueguizhihuan),
            Self::Qinglong => source && !target && d.nature!=DamageNature::Normal && core.player(d.target).alive && equip(CardKind::Qinglonglingzhu) && (!core.player(d.target).hand.is_empty()||!core.player(d.target).equips.is_empty()),
            Self::Yinzei => target && p.hand.is_empty() && p.hp>0 && d.source.is_some_and(|s|s!=owner&&core.player(s).alive&&!core.player(s).hand.is_empty()),
            Self::Zhangwu => target && d.source.is_some_and(|s|s!=owner&&core.player(s).alive),
            Self::Zlfanshi | Self::Lingqu => target,
            Self::Zhue => d.source.is_some_and(|s|s!=owner&&core.player(s).alive),
            Self::Abi => target && d.source.is_some_and(|s|core.player(s).alive),
            Self::Yuance => d.source.is_some_and(|s|s!=d.target&&core.player(s).alive&&core.player(s).hp>0)&&core.player(d.target).alive&&core.player(d.target).hp>0,
            Self::Jiding => !target&&core.is_boss_side(owner)==core.is_boss_side(d.target)&&d.source.is_some_and(|s|s!=owner&&core.player(s).alive&&core.is_boss_side(s)!=core.is_boss_side(owner)),
            Self::Renxing => core.turn.current!=owner,
            Self::Cuidui => source && !target && core.player(d.target).alive && catalog.skill_by_name(crate::vocabulary::names::BOSS_ZHONGDU).is_some_and(|def|!core.has_skill_any_state(d.target,def)),
            Self::Duqu => target && d.source.is_some_and(|s|s!=owner&&core.player(s).alive),
            Self::Huanxia => source && d.card.is_some_and(|card|core.turn.huanxia_track.last().is_some_and(|(c,_)|*c==card)),
            Self::Jidian => source,
            Self::Minwan => source && !target && core.turn.current==owner,
        }
    }
}
'''
put(base+'event/native_damage.rs',text)
p=base+'catalog.rs';s=read(p).replace('fn native_damage_effect(&self)', 'fn native_damage_effect(&self, _kind: crate::event::EventKind)');put(p,s)
# Genuine boss catalog declarations and helper definitions.
p='crates/engine-native-content/src/content/standard/native_damage.rs'
put(p,'''//! Native declarations enter the ordinary source-window competition.
use super::*;
use crate::event::NativeDamageEffect;
pub(super) struct NativeDamageBehavior { pub name: &'static str, pub effects: &'static [NativeDamageEffect] }
impl SkillBehavior for NativeDamageBehavior {
    fn native_damage_effect(&self, kind: EventKind) -> Option<NativeDamageEffect> { self.effects.iter().copied().find(|e|e.window().event_kind()==kind) }
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool { false }
    fn filter_in_window(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData, kind: EventKind) -> bool {
        matches!(event, EventData::Damage(d) if self.native_damage_effect(kind).is_some_and(|effect|effect.matches(owner,d,ctx)))
    }
    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> { NoopBehavior.start(owner,event) }
    fn start_in_window(&self, ctx: &QueryCtx<'_>, owner: PlayerId, _event: &EventData, kind: EventKind) -> Box<dyn SkillMachine> {
        Box::new(crate::kernel::NativeDamageSkill { owner, def:ctx.catalog.skill_by_name(self.name).expect("immutable declaration"), window:self.native_damage_effect(kind).expect("declared window").window() })
    }
}
pub(super) fn configure(skills: &mut Vec<SkillDef>) {
    use NativeDamageEffect as E;
    for skill in skills.iter_mut() {
        let effects: &'static [E] = match skill.name {
'''+''.join(f'            {name} => &[{", ".join("E::"+e for e in effects)}],\n' for name,effects in {'JUE_QING':['Jueqing'],'BOSS_WUXIN':['Wuxin'],'BOSS_ZUIJIU':['Zuijiu'],'BOSS_HUOSHEN':['Huoshen'],'BOSS_NITAI':['NitaiOut','NitaiIn'],'BOSS_ZHAOHUO':['Zhaohuo'],'BOSS_BUFO':['Bufo'],'BOSS_LINGQU':['Lingqu','LingquCancel'],'BOSS_YINZEI':['Yinzei'],'BOSS_ZHANGWU':['Zhangwu'],'BOSS_ZLFANSHI':['Zlfanshi'],'BOSS_ZHUE':['Zhue'],'BOSS_ABI':['Abi'],'BOSS_YUANCE':['Yuance'],'BOSS_JIDING':['Jiding'],'BOSS_RENXING':['Renxing'],'BOSS_CUIDUI':['Cuidui'],'BOSS_DUQU':['Duqu'],'BOSS_JIDIAN':['Jidian'],'BOSS_MINWAN':['Minwan']}.items())+'''            _ => continue,
        };
        let mut meta=effects[0].meta(); meta.extra_events=effects[1..].iter().map(|e|e.window().event_kind()).collect();
        skill.trigger=Some(meta);
        skill.behavior=Box::new(NativeDamageBehavior{name:skill.name,effects});
    }
'''+''.join(f'    {{ let effect=E::{e}; skills.push(SkillDef {{name:effect.helper_name().unwrap(), trigger:Some(effect.meta()), limits:Default::default(), behavior:Box::new(NativeDamageBehavior{{name:effect.helper_name().unwrap(),effects:&[E::{e}]}})}}); }}\n'for e in helpers)+''' }
''')
p='crates/engine-native-content/src/content/standard/boss_skills.rs';s=read(p);s=re.sub(r'effect: (crate::event::NativeDamageEffect::\w+)',r'effects: &[\1]',s);put(p,s)
p='crates/engine-native-content/src/content/standard.rs';s=read(p).replace('skills.extend(boss_skills::skills());','skills.extend(boss_skills::skills());\n    native_damage::configure(&mut skills);');put(p,s)
# Native host declarations preserve their existing non-damage behavior.
p='crates/engine-native-content/src/content/host.rs';s=read(p).replace('impl SkillBehavior for HostSkillBehavior {','''impl SkillBehavior for HostSkillBehavior {
    fn native_damage_effect(&self, kind: crate::event::EventKind) -> Option<crate::event::NativeDamageEffect> {
        crate::event::NativeDamageEffect::from_host(self.behavior).filter(|e| e.window().event_kind()==kind)
    }
    fn filter_in_window(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData, kind: crate::event::EventKind) -> bool {
        if let Some(effect)=self.native_damage_effect(kind) { return matches!(event, EventData::Damage(d) if effect.matches(owner,d,ctx)); }
        self.filter(ctx,owner,event)
    }
    fn start_in_window(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData, kind: crate::event::EventKind) -> Box<dyn crate::catalog::SkillMachine> {
        if let Some(effect)=self.native_damage_effect(kind) {
            return Box::new(crate::kernel::NativeDamageSkill { owner, def:ctx.catalog.skill_by_name(&self.name).expect("host declaration"), window:effect.window() });
        }
        self.start(owner,event)
    }
''').replace('trigger: behavior.trigger_meta(),','trigger: crate::event::NativeDamageEffect::from_host(behavior).map(|e|e.meta()).or_else(||behavior.trigger_meta()),');put(p,s)
# Delegate the old source units to exactly one selected queued body.
p=base+'kernel/damage_windows.rs';s=read(p).replace('pub(super) fn commit_damage_window(ctx: &mut Ctx<\'_>, root: EventId, window: Window)',"pub(super) fn apply_native_damage(ctx: &mut Ctx<'_>, root: EventId, window: Window, effect: crate::event::NativeDamageEffect)")
# Comment starts identify existing semantic units; preserve their complete bodies/error handling.
labels=[('急冻冰封','Jidong'),('诚信免伤','Chengxin'),('赤龙牙','Chilongya'),('星君眼','Xingjunyan'),('链钩','Liangou'),('飞刃','Feiren'),('刺猬','Ciqiu'),('毒瘟','Duwen'),('脉轮','Mailun'),('护天护体','Hutian'),('火神','Huoshen'),('拟态（Boss 梼杌，规则 v217）：锁定——你于回合外','NitaiOut'),('招祸','Zhaohuo'),('拟态（Boss 梼杌，规则 v217）：锁定——防止','NitaiIn'),('灵渠（Boss 玄武，规则 v237）：受到的大于','Bufo'),('boss_lingqu cancel','LingquCancel')]
# Split each window arm into complete top-level comment-delimited units.
for window in ['Before','Begin','Begin1','Begin4']:
 start=s.index('        Window::'+window+' => {')
 body_start=s.index('\n',start)+1
 next_arm=re.search(r'^        (?:Window::\w+|_) =>',s[body_start:],re.M)
 end=body_start+next_arm.start()
 body=s[body_start:end]; suffix='        }\n'; assert body.endswith(suffix);body=body[:-len(suffix)]
 starts=[]
 for label,effect in labels:
  m=re.search(r'^            // '+re.escape(label),body,re.M)
  if m:starts.append((m.start(),effect))
 starts.sort();parts=[]
 for i,(pos,effect) in enumerate(starts):
  part=body[pos:starts[i+1][0] if i+1<len(starts) else len(body)]
  if effect=='Mailun':
   part='''            if matches!(effect, crate::event::NativeDamageEffect::MailunOut3 | crate::event::NativeDamageEffect::MailunOut4) {
                let src=d.source.expect("source role");
                amount += if effect==crate::event::NativeDamageEffect::MailunOut3 {-1} else {1};
                ctx.core.player_mut(src).mailun_out_used=true;
            }
            if matches!(effect, crate::event::NativeDamageEffect::MailunIn3 | crate::event::NativeDamageEffect::MailunIn4) {
                amount += if effect==crate::event::NativeDamageEffect::MailunIn3 {-1} else {1};
                ctx.core.player_mut(d.target).mailun_in_used=true;
            }
'''
  else:part='            if effect == crate::event::NativeDamageEffect::'+effect+' {\n'+part+'            }\n'
  # Xingjunyan has both source and target roles in the pinned declaration; filter qualified its owner.
  if effect=='Xingjunyan':
   part='''            if effect == crate::event::NativeDamageEffect::Xingjunyan { amount += 1; }
'''
  parts.append(part)
 s=s[:body_start]+''.join(parts)+suffix+s[end:]
put(p,s)
# End/After bodies: wrap each existing top-level semantic unit. Restrict global callbacks to admitted owner.
for file,func,units in [('damage.rs','post_damage',['Shuchui','Paotai','Xixue','Qinglong','Yinzei','Zhangwu','Zlfanshi','Zhue','Abi','Yuance','Lingqu','Jiding','Renxing','Cuidui','Duqu']),('damage_after.rs','post_damage_after',['Huanxia','Jidian','Minwan'])]:
 p=base+'kernel/'+file;s=read(p);start=s.index('pub(super) fn '+func);body_start=s.index('\n',start)+1
 end=s.index('\n}',body_start)
 body=s[body_start:end];starts=[m.start()for m in re.finditer(r'^    // (?!同|伤|后|来源|色|极点|step)',body,re.M)]
 # Top-level comments span continuation lines: use code boundaries by first comment after previous code.
 lines=body.splitlines(keepends=True);chunks=[];chunk='';seen_code=False
 for line in lines:
  if line.startswith('    //') and seen_code and chunk.rstrip().endswith(('}', ';')):
   chunks.append(chunk);chunk='';seen_code=False
  chunk+=line
  if line.strip() and not line.lstrip().startswith('//'):seen_code=True
 if chunk:chunks.append(chunk)
 assert len(chunks)==len(units),(file,len(chunks),len(units))
 wrapped=''
 for effect,chunk in zip(units,chunks):
  if effect in ['Zhue','Yuance','Jiding','Renxing']:
   # Existing global iteration is now one source unit per admitted owner.
   chunk=re.sub(r'let holders: Vec<PlayerId> = ctx\n.*?\.collect\(\);','let holders: Vec<PlayerId> = vec![owner];',chunk,flags=re.S)
  wrapped+='    if effect == crate::event::NativeDamageEffect::'+effect+' {\n'+chunk+'    }\n'
 s=s[:start]+s[start:body_start].replace('d: crate::event::DamageData)', 'd: crate::event::DamageData, effect: crate::event::NativeDamageEffect, owner: PlayerId)')+wrapped+s[end:];put(p,s)
p=base+'kernel.rs';s=read(p);start=s.index('        EventData::DamageWindow { root, window } => {');end=s.index('        EventData::LoseHp',start);s=s[:start]+'        EventData::DamageWindow { .. } => {}\n'+s[end:];put(p,s)
p=base+'kernel/damage_skill.rs';s=read(p).replace('s.behavior.native_damage_effect()', 's.behavior.native_damage_effect(self.window.event_kind())');s=s.replace('''            NativeDamageEffect::Zuijiu => {
                ctx.modify_damage_amount(1);
            }''','''            NativeDamageEffect::Zuijiu => { ctx.modify_damage_amount(1); }
            effect => {
                let root=ctx.current_event_frame_id();
                match self.window {
                    DamageWindowKind::End => post_damage(ctx,d,effect,self.owner),
                    DamageWindowKind::After => damage_after::post_damage_after(ctx,d,effect,self.owner),
                    _ => damage_windows::apply_native_damage(ctx,root,self.window,effect),
                }
            }''');put(p,s)
# One qualification consumer used by collector, queue start-stop and restore.
p=base+'resolution/trigger_order.rs';s=read(p).replace('    let Some(inst) = core.skills.get(&(q.owner, q.def)) else {\n        return false;\n    };','    let inst = core.skills.get(&(q.owner, q.def));');s=s.replace('''    inst.enabled
        && core.player(q.owner).alive
        && (!def.limits.once_per_turn || !inst.once_used)''','''    let helper=def.behavior.native_damage_effect(kind).is_some_and(|e|e.helper_name()==Some(def.name));
    (helper || inst.is_some_and(|i|i.enabled && (!def.limits.once_per_turn || !i.once_used)))
        && core.player(q.owner).alive''');s=s.replace('s.behavior.native_damage_effect()', 's.behavior.native_damage_effect(window.event_kind())');put(p,s)
p=base+'resolution.rs';s=read(p).replace('''    for ((owner, def_id), _) in core.skills.range(..) {''','''    let mut providers: Vec<_> = core.skills.keys().copied().collect();
    for &def in catalog.candidates(kind) {
        if catalog.skills[def as usize].behavior.native_damage_effect(kind).is_some_and(|e|e.helper_name()==Some(catalog.skills[def as usize].name)) {
            providers.extend((0..core.players.len() as PlayerId).map(|owner|(owner,def)));
        }
    }
    providers.sort_unstable(); providers.dedup();
    for (owner, def_id) in &providers {''');put(p,s)
# Extra source metadata can differ by grouped window (e.g. Lingqu -11 vs End 0).
# Replace per-frame meta lookups at central consumers with exact native window metadata.
p=base+'resolution.rs';s=read(p).replace('let Some(meta) = def.trigger.as_ref() else {','let native_meta=def.behavior.native_damage_effect(kind).map(|e|e.meta());\n        let Some(meta) = native_meta.as_ref().or(def.trigger.as_ref()) else {');s=s.replace('let meta = def.trigger.as_ref().expect("eligible meta");','let kind=trigger_order::event_kind(res.frame(frame_id).expect("frame"));\n    let native_meta=def.behavior.native_damage_effect(kind).map(|e|e.meta());\n    let meta = native_meta.as_ref().or(def.trigger.as_ref()).expect("eligible meta");');put(p,s)
# Keep grouped priority in queue restore aligned with actual window.
p=base+'resolution/trigger_order.rs';s=read(p);s=s.replace('''            let meta = catalog.skills[q.def as usize]
                .trigger
                .as_ref()
                .ok_or_else(bad)?;''','''            let skill=&catalog.skills[q.def as usize];
            let native_meta=skill.behavior.native_damage_effect(event_kind(frame)).map(|e|e.meta());
            let meta=native_meta.as_ref().or(skill.trigger.as_ref()).ok_or_else(bad)?;''');put(p,s)
(ev/'implementation-inventory-seed.json').write_text(json.dumps({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'groups':groups,'helpers':helpers,'change_hp':'explicit next bounded slice per supervisor; genuine ChangeHp consumer absent'},indent=2)+'\n')
print('Implemented finite native units:',len(variants))
