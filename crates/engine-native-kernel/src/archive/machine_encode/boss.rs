//! Boss 机族的快照编码臂（自 machine_encode.rs 按机族拆出；字节序与
//! tag 零变化，仅代码组织调整）。
use super::*;

pub(super) fn enc(w: &mut W, m: &MachineSnapshot) {
    match m {
        MachineSnapshot::BossXiangde { owner } => {
            w.u8(61);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossFutai { owner } => {
            w.u8(62);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossLuanchang { owner } => {
            w.u8(63);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossZhaohuo { owner } => {
            w.u8(65);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossBuchun {
            owner,
            stage,
            targets,
        } => {
            w.u8(66);
            enc_pid(w, owner);
            w.u8(*stage);
            w.u32(targets.len() as u32);
            for t in targets {
                w.u32(*t);
            }
        }
        MachineSnapshot::BossXingxia {
            owner,
            stage,
            actor,
        } => {
            w.u8(67);
            enc_pid(w, owner);
            w.u8(*stage);
            w.u32(*actor);
        }
        MachineSnapshot::BossQizuo { owner } => {
            w.u8(68);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossXiaoxiong { owner } => {
            w.u8(69);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossLingbo { owner } => {
            w.u8(70);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossTashui { owner } => {
            w.u8(71);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossZjLeiji { owner } => {
            w.u8(72);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossHeisheng { owner } => {
            w.u8(73);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossLeizhu { owner, emitted } => {
            w.u8(74);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossRenao { owner, emitted } => {
            w.u8(75);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossWanghun { owner } => {
            w.u8(76);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossZhuxin { owner, emitted } => {
            w.u8(77);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossPingdeng { owner, emitted } => {
            w.u8(78);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossHuoxing { owner, emitted } => {
            w.u8(79);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossBingfeng { owner } => {
            w.u8(80);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossWangxiang { owner } => {
            w.u8(81);
            enc_pid(w, owner);
        }
        MachineSnapshot::Shedu { owner } => {
            w.u8(64);
            enc_pid(w, owner);
        }
        _ => unreachable!("boss family variant routed by machine_encode dispatcher"),
    }
}
