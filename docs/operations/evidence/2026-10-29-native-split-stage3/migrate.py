from pathlib import Path
import re,shutil
src=Path('crates/engine-native/src');v=src/'vocabulary';v.mkdir(exist_ok=True)
def change(p,old,new):
 t=p.read_text();assert old in t,(p,old);p.write_text(t.replace(old,new))
def move(a,b): b.parent.mkdir(parents=True,exist_ok=True);shutil.move(a,b)
for name in ['behavior','behavior_list','keys','tags','tag_decode','metadata']:
 move(src/f'content/host/{name}.rs',v/f'host/{name}.rs')
move(src/'content/host/metadata',v/'host/metadata')
# Public immutable vocabulary stays shared; content-only tuning constants remain local.
p=src/'content/host/constants.rs';blocks=p.read_text().split('\n\n');private=[b for b in blocks if 'pub(super)' in b];public=[b for b in blocks if 'pub(super)' not in b];p.write_text('\n\n'.join(private)+'\n');(v/'host/constants.rs').write_text('\n\n'.join(public)+'\n')
move(src/'content/standard/names.rs',v/'names.rs');move(src/'content/standard/generals.rs',v/'generals.rs')
change(v/'generals.rs','use super::*;','use crate::catalog::Faction;\nuse super::names::*;')
p=src/'content/extension.rs';t=p.read_text();a=t.index('/// 扩展提供的额外武将条目。');b=t.index('/// 开局初始化钩子',a);decl=t[a:b];p.write_text(t[:a]+'pub use crate::vocabulary::generals::ExtensionGeneral;\n\n'+t[b:]);(v/'generals.rs').write_text((v/'generals.rs').read_text()+'\n'+decl)
p=src/'content/host.rs';t=p.read_text();a=t.index('#[derive(Debug, Clone, Copy)]\npub struct HostSkillDecl');b=t.index('impl SkillBehavior',a);decl=t[a:b];t=t[:a]+t[b:];a=t.index('/// 牌种固有的装备技能');tables=t[a:];t=t[:a]+'pub use crate::vocabulary::card_skills::{card_kind_host_skills, card_kind_equip_skills};\n';
for n in ['behavior','behavior_list','keys','tags','tag_decode','metadata']:
 t=t.replace(f'mod {n};\n','')
t=t.replace('pub use behavior::HostBehavior;\n','pub use crate::vocabulary::host::*;\n').replace('pub use constants::*;','use constants::*;');p.write_text(t)
(v/'host.rs').write_text('//! 宿主行为的不可变描述、键、标签与触发元数据；执行由内容层 wrapper 承担。\n\nuse crate::catalog::TriggerMeta;\nuse crate::event::{EventKind, Timing};\n\nmod behavior;\npub use behavior::HostBehavior;\nmod behavior_list;\nmod keys;\nmod tags;\nmod tag_decode;\nmod metadata;\nmod constants;\npub use constants::*;\n\n'+decl)
# Card category is a pure descriptor, shared by content and kernel queries.
p=src/'content/host/support.rs';t=p.read_text();a=t.index('/// 羊皮卷发现类别池');b=t.index('\n}',a)+2;classification=t[a:b];p.write_text(t[:a]+'pub use crate::vocabulary::card_skills::yangpijuan_class;'+t[b:]);(v/'card_skills.rs').write_text('//! 牌种伴随技能和发现类别的不可变描述。\n\nuse crate::catalog::CardKind;\nuse super::host::*;\n\n'+tables+'\n'+classification+'\n')
(src/'vocabulary.rs').write_text('//! 内核与内容共享的不可变规则词汇；不含技能机器或结算执行。\n\npub mod names;\npub mod host;\npub mod generals;\npub mod card_skills;\n')
change(src/'lib.rs','pub mod state;','pub mod state;\npub mod vocabulary;')
change(src/'content/standard.rs','mod names;\npub use names::*;\nmod generals;\npub use generals::*;','pub use crate::vocabulary::names::*;\npub use crate::vocabulary::generals::*;')
# One shared assembly implementation, with engine's internal name preserved.
p=src/'engine.rs';t=p.read_text();a=t.index('pub(crate) fn apply_general_assembly(');b=t.index('\n#[cfg(test)]',a);assembly=t[a:b];p.write_text(t[:a]+'pub(crate) use crate::state::apply_general_assembly;\n'+t[b:]);(src/'state/general_assembly.rs').write_text('//! 武将装配：明置、调试装配和内容转将共用的状态更新。\n\nuse crate::ids::PlayerId;\n\n'+assembly)
with (src/'state.rs').open('a') as f:f.write('\nmod general_assembly;\npub(crate) use general_assembly::apply_general_assembly;\n')
# Rewrite only the kernel layer and the newly shared assembly path.
import importlib.util
s=importlib.util.spec_from_file_location('l','scripts/check-native-layering.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
for p in m.kernel_files(Path('.')):
 t=p.read_text().replace('crate::content::host::shengong_borrow_options(', '(ctx.catalog.hooks.borrow_options)(').replace('crate::content::host::shengong_borrowed_owner(', '(ctx.catalog.hooks.borrowed_owner)(')
 t=t.replace('crate::content::host::yangpijuan_class','crate::vocabulary::card_skills::yangpijuan_class').replace('crate::content::host::{card_kind_equip_skills, card_kind_host_skills}','crate::vocabulary::card_skills::{card_kind_equip_skills, card_kind_host_skills}')
 t=t.replace('crate::engine::apply_general_assembly','crate::state::apply_general_assembly')
 t=t.replace('crate::content::host::','crate::vocabulary::host::').replace('crate::content::extension::ExtensionGeneral','crate::vocabulary::generals::ExtensionGeneral')
 for n in ['Gender','general_gender','STANDARD_GENERALS','BOSS_GENERALS']:
  t=t.replace('crate::content::standard::'+n,'crate::vocabulary::generals::'+n)
 t=t.replace('crate::content::standard::{general_gender, Gender, GUI_CAI, JI_JIU, LONG_DAN, QING_GUO};','crate::vocabulary::generals::{general_gender, Gender};\nuse crate::vocabulary::names::{GUI_CAI, JI_JIU, LONG_DAN, QING_GUO};')
 t=t.replace('crate::content::standard::','crate::vocabulary::names::')
 p.write_text(t)
# Typed content callbacks for skill-specific borrowed equipment responses.
p=src/'catalog.rs';t=p.read_text();t=t.replace('    pub restore_machine: RestoreMachineHook,','    pub restore_machine: RestoreMachineHook,\n    pub borrow_options: fn(&crate::machine::Ctx<\'_>, crate::ids::PlayerId, bool) -> Vec<crate::choice::ChoiceOption>,\n    pub borrowed_owner: fn(&crate::machine::Ctx<\'_>, crate::ids::PlayerId, crate::ids::CardId) -> Option<crate::ids::PlayerId>,');t=t.replace('        restore_machine: |_, _| None,','        restore_machine: |_, _| None,\n        borrow_options: |_, _, _| Vec::new(),\n        borrowed_owner: |_, _, _| None,');p.write_text(t)
p=src/'content/hooks.rs';change(p,'    restore_machine,','    restore_machine,\n    borrow_options: crate::content::host::shengong_borrow_options,\n    borrowed_owner: crate::content::host::shengong_borrowed_owner,')
# Keep the ratchet current in the same mutation.
p=Path('scripts/check-native-layering.py');t=p.read_text().replace('BASELINE = 240','BASELINE = 0').replace('    "kernel", "snap",','    "vocabulary", "kernel", "snap",');p.write_text(t)
