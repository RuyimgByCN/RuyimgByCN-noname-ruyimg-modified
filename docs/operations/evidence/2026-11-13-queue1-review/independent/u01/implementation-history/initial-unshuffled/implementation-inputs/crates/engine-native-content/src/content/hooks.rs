//! 内容层装入目录的回调（拆 crate 第 2 步）：内核经 `Catalog::hooks` 调用，
//! 不直接引用 `crate::content`。

use crate::catalog::{Catalog, ContentHooks, SkillMachine};
use crate::content::host::{HostBehavior, HostMachine, HostStage};
use crate::content::standard::{restore_content_machine, LeizhuMachine, RenaoMachine};
use crate::snap::{MachineSnapshot, SnapshotError};

/// 标准内容目录（含扩展合并目录）装入的回调。
pub(crate) const CONTENT_HOOKS: ContentHooks = ContentHooks {
    vm_active: super::vm_effects::active_eligible,
    vm_validate: super::vm_effects::validate,
    vm_commit: super::vm_effects::step,
    vm_ask: super::vm_effects::active_ask,
    card_intrinsic: crate::content::host::card_intrinsic,
    host_ask: crate::content::host::active_ask_host,
    restore_machine,
    borrow_options: crate::content::host::shengong_borrow_options,
    borrowed_owner: crate::content::host::shengong_borrowed_owner,
    r05_role_defs,
};

/// R05-F：目录名含 `#r05:{role}` 的全部定义号（装载器派生模板独占 `#`）。
fn r05_role_defs(catalog: &Catalog, role: &'static str) -> Vec<u16> {
    let suffix = format!("#r05:{role}");
    (0..catalog.skills.len() as u16)
        .filter(|id| catalog.skills[*id as usize].name.ends_with(&suffix))
        .collect()
}

/// 内容机器的快照重建（自 `MachineSnapshot::restore` 迁出的内容分支）：先走标准
/// 内容分派，再按 Boss/宿主机器标签重建；其余快照返回 `None` 交回内核分派。
fn restore_machine(
    s: &MachineSnapshot,
    catalog: &Catalog,
) -> Option<Result<Box<dyn SkillMachine>, SnapshotError>> {
    if let Some(m) = restore_content_machine(s, catalog) {
        return Some(Ok(m));
    }
    Some(match s {
        MachineSnapshot::BossLeizhu { owner, emitted } => {
            Ok(Box::new(LeizhuMachine::new(*owner, *emitted)))
        }
        MachineSnapshot::BossRenao { owner, emitted } => {
            Ok(Box::new(RenaoMachine::new(*owner, *emitted)))
        }
        MachineSnapshot::Host { .. } => restore_host(s),
        _ => return None,
    })
}

/// 宿主机器按行为/阶段标签重建；未知标签拒绝。
fn restore_host(s: &MachineSnapshot) -> Result<Box<dyn SkillMachine>, SnapshotError> {
    let MachineSnapshot::Host {
        def_key,
        behavior,
        stage,
        owner,
        target,
        card,
        cards,
        targets,
    } = s
    else {
        return Err(SnapshotError::UnknownMachine);
    };
    let behavior = HostBehavior::from_tag(*behavior).ok_or(SnapshotError::UnknownMachine)?;
    let stage = HostStage::from_tag(*stage).ok_or(SnapshotError::UnknownMachine)?;
    Ok(Box::new(HostMachine {
        def_key: def_key.clone(),
        behavior,
        stage,
        owner: *owner,
        target: *target,
        card: *card,
        cards: cards.clone(),
        targets: targets.clone(),
    }))
}
