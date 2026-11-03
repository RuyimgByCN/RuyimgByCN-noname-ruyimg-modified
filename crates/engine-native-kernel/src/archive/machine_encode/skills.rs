//! 经典武将技能机的快照编码臂（自 machine_encode.rs 按机族拆出；字节
//! 序与 tag 零变化，仅代码组织调整）。
use super::*;

pub(super) fn enc(w: &mut W, m: &MachineSnapshot) {
    match m {
        MachineSnapshot::SanCai {
            owner,
            stage,
            target,
        } => {
            w.u8(13);
            enc_pid(w, owner);
            w.u8(match stage {
                SanCaiStageSnap::Draw => 0,
                SanCaiStageSnap::Target => 1,
                SanCaiStageSnap::GiveCard => 2,
                SanCaiStageSnap::Finish => 3,
            });
            enc_opt_pid(w, target);
        }
        MachineSnapshot::YiShang { owner, stage } => {
            w.u8(14);
            enc_pid(w, owner);
            w.u8(match stage {
                YiShangStageSnap::Target => 0,
                YiShangStageSnap::Apply => 1,
            });
        }
        MachineSnapshot::SuiDun => w.u8(15),
        MachineSnapshot::SuQing { target } => {
            w.u8(16);
            enc_pid(w, target);
        }
        MachineSnapshot::ShouYi { owner } => {
            w.u8(17);
            enc_pid(w, owner);
        }
        MachineSnapshot::FanKui {
            owner,
            source,
            asked,
        } => {
            w.u8(18);
            enc_pid(w, owner);
            enc_opt_pid(w, source);
            w.boolean(*asked);
        }
        MachineSnapshot::JianXiong { owner, card, asked } => {
            w.u8(19);
            enc_pid(w, owner);
            w.u32(*card);
            w.boolean(*asked);
        }
        MachineSnapshot::BiYue { owner } => {
            w.u8(21);
            enc_pid(w, owner);
        }
        MachineSnapshot::JiZhi { owner } => {
            w.u8(25);
            enc_pid(w, owner);
        }
        MachineSnapshot::ZhiHeng { owner, stage } => {
            w.u8(27);
            enc_pid(w, owner);
            w.u8(match stage {
                ZhiHengStageSnap::Select => 0,
                ZhiHengStageSnap::Draw => 1,
            });
        }
        MachineSnapshot::KuRou { owner, stage } => {
            w.u8(26);
            enc_pid(w, owner);
            w.u8(match stage {
                KuRouStageSnap::Lose => 0,
                KuRouStageSnap::Draw => 1,
            });
        }
        MachineSnapshot::LuoShen { owner } => {
            w.u8(22);
            enc_pid(w, owner);
        }
        MachineSnapshot::YingZi { owner } => {
            w.u8(28);
            enc_pid(w, owner);
        }
        MachineSnapshot::Dismantle {
            card,
            source,
            target,
            asked,
        } => {
            w.u8(29);
            w.u32(*card);
            enc_pid(w, source);
            enc_pid(w, target);
            w.boolean(*asked);
        }
        MachineSnapshot::MonkeySteal {
            card,
            user,
            holders,
            asked,
        } => {
            // E03 猴子偷桃：使用前取消窗口（tag 44，表尾追加）
            w.u8(44);
            w.u32(*card);
            enc_pid(w, user);
            enc_pids(w, holders);
            w.boolean(*asked);
        }
        MachineSnapshot::XiaoJi { owner } => {
            // tag 43（表尾追加）：tag 29 曾与 Dismantle 冲突（解码分支
            // 不可达，枭姬挂起无法恢复），2026-09-24 经 A0 唯一性检查
            // 定位后修正；旧字节中 XiaoJi 挂起本就无法正确解码。
            w.u8(43);
            enc_pid(w, owner);
        }
        MachineSnapshot::QingNang {
            owner,
            stage,
            healed,
        } => {
            w.u8(30);
            enc_pid(w, owner);
            w.u8(match stage {
                QingNangStageSnap::TargetAsk => 0,
                QingNangStageSnap::TargetRead => 1,
                QingNangStageSnap::CardRead => 2,
            });
            enc_opt_pid(w, healed);
        }
        MachineSnapshot::JiJi {
            owner,
            stage,
            target,
            given,
        } => {
            w.u8(20);
            enc_pid(w, owner);
            w.u8(match stage {
                JiJiStageSnap::Draw => 0,
                JiJiStageSnap::Target => 1,
                JiJiStageSnap::Retarget => 2,
                JiJiStageSnap::GiveCard => 3,
                JiJiStageSnap::GiveChooseCard => 4,
            });
            enc_opt_pid(w, target);
            w.u8(*given);
        }
        MachineSnapshot::GangLie {
            owner,
            source,
            asked,
        } => {
            w.u8(31);
            enc_pid(w, owner);
            enc_pid(w, source);
            w.u8(*asked as u8);
        }
        MachineSnapshot::XiaoGuo {
            owner,
            target,
            stage,
        } => {
            w.u8(45);
            enc_pid(w, owner);
            enc_pid(w, target);
            w.u8(match stage {
                crate::snap::XiaoGuoStageSnap::Ask => 0,
                crate::snap::XiaoGuoStageSnap::Read => 1,
            });
        }
        MachineSnapshot::KuangFu { owner, target } => {
            w.u8(46);
            enc_pid(w, owner);
            enc_pid(w, target);
        }
        MachineSnapshot::YaoWu {
            owner,
            source,
            stage,
        } => {
            w.u8(47);
            enc_pid(w, owner);
            enc_pid(w, source);
            w.u8(match stage {
                crate::snap::YaoWuStageSnap::Ask => 0,
                crate::snap::YaoWuStageSnap::Read => 1,
            });
        }
        MachineSnapshot::ShuShen {
            owner,
            target,
            stage,
        } => {
            w.u8(48);
            enc_pid(w, owner);
            enc_opt_pid(w, target);
            w.u8(match stage {
                ShuShenStageSnap::Ask => 0,
                ShuShenStageSnap::Read => 1,
            });
        }
        MachineSnapshot::ShenZhi { owner } => {
            w.u8(49);
            enc_pid(w, owner);
        }
        MachineSnapshot::XunXun { owner, stage, top } => {
            w.u8(50);
            enc_pid(w, owner);
            w.u8(match stage {
                XunXunStageSnap::Undo => 0,
                XunXunStageSnap::Arrange => 1,
            });
            w.len(top.len());
            for c in top {
                w.u32(*c);
            }
        }
        MachineSnapshot::WangXi { owner, other } => {
            w.u8(51);
            enc_pid(w, owner);
            enc_pid(w, other);
        }
        MachineSnapshot::JiJie { owner, card, stage } => {
            w.u8(52);
            enc_pid(w, owner);
            match card {
                None => w.u8(0),
                Some(c) => {
                    w.u8(1);
                    w.u32(*c);
                }
            }
            w.u8(match stage {
                JiJieStageSnap::Ask => 0,
                JiJieStageSnap::Read => 1,
            });
        }
        MachineSnapshot::JiYuan { owner } => {
            w.u8(53);
            enc_pid(w, owner);
        }
        MachineSnapshot::WangZun { owner, target } => {
            w.u8(54);
            enc_pid(w, owner);
            enc_pid(w, target);
        }
        MachineSnapshot::TongJi { owner, stage } => {
            w.u8(55);
            enc_pid(w, owner);
            w.u8(match stage {
                TongJiStageSnap::Ask => 0,
                TongJiStageSnap::Read => 1,
            });
        }
        MachineSnapshot::Wuzhong { owner } => {
            w.u8(56);
            enc_pid(w, owner);
        }
        MachineSnapshot::Taoyuan { owner } => {
            w.u8(57);
            enc_pid(w, owner);
        }
        MachineSnapshot::Wugu {
            owner,
            cards,
            pickers,
        } => {
            w.u8(58);
            enc_pid(w, owner);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
            w.len(pickers.len());
            for p in pickers {
                enc_pid(w, p);
            }
        }
        MachineSnapshot::Jiedao {
            owner,
            holder,
            victim,
            stage,
        } => {
            w.u8(59);
            enc_pid(w, owner);
            enc_pid(w, holder);
            enc_pid(w, victim);
            w.u8(match stage {
                JiedaoStageSnap::Ask => 0,
                JiedaoStageSnap::Read => 1,
            });
        }
        MachineSnapshot::TuXi { owner, stage } => {
            w.u8(32);
            enc_pid(w, owner);
            w.u8(match stage {
                TuXiStageSnap::Ask => 0,
                TuXiStageSnap::Read => 1,
            });
        }
        MachineSnapshot::LuoYi { owner } => {
            w.u8(33);
            enc_pid(w, owner);
        }
        MachineSnapshot::GuanXing { owner, stage, top } => {
            w.u8(34);
            enc_pid(w, owner);
            w.u8(match stage {
                GuanXingStageSnap::Arrange => 0,
                GuanXingStageSnap::Place => 1,
            });
            w.len(top.len());
            for c in top {
                w.u32(*c);
            }
        }
        MachineSnapshot::LianYing { owner } => {
            w.u8(35);
            enc_pid(w, owner);
        }
        MachineSnapshot::Rende {
            owner,
            stage,
            cards,
        } => {
            w.u8(36);
            enc_pid(w, owner);
            w.u8(match stage {
                RendeStageSnap::SelectCardsAsk => 0,
                RendeStageSnap::TargetAsk => 1,
                RendeStageSnap::TargetRead => 2,
            });
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
        }
        MachineSnapshot::Jijiang {
            owner,
            stage,
            queue,
            helper,
            sha,
            inner,
        } => {
            w.u8(37);
            enc_pid(w, owner);
            w.u8(match stage {
                JijiangStageSnap::HelperAsk => 0,
                JijiangStageSnap::TargetAsk => 1,
            });
            // 逐人询问队列：None=0，Some=1+len+pid 序列
            match queue {
                None => w.u8(0),
                Some(q) => {
                    w.u8(1);
                    w.len(q.len());
                    for p in q {
                        enc_pid(w, p);
                    }
                }
            }
            enc_opt_pid(w, helper);
            enc_opt_u32(w, *sha);
            match inner {
                Some(m) => {
                    w.u8(1);
                    super::enc_machine(w, m);
                }
                None => w.u8(0),
            }
        }
        MachineSnapshot::Snatch {
            card,
            source,
            target,
            asked,
        } => {
            w.u8(38);
            w.u32(*card);
            enc_pid(w, source);
            enc_pid(w, target);
            w.boolean(*asked);
        }
        MachineSnapshot::JieYin {
            owner,
            stage,
            cards,
        } => {
            w.u8(39);
            enc_pid(w, owner);
            w.u8(match stage {
                JieYinStageSnap::CardsAsk => 0,
                JieYinStageSnap::TargetAsk => 1,
                JieYinStageSnap::TargetRead => 2,
            });
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
        }
        MachineSnapshot::FanJian {
            owner,
            stage,
            target,
            chosen_suit,
        } => {
            w.u8(40);
            enc_pid(w, owner);
            w.u8(match stage {
                FanJianStageSnap::TargetAsk => 0,
                FanJianStageSnap::SuitAsk => 1,
                FanJianStageSnap::PickAsk => 2,
                FanJianStageSnap::Consume => 3,
            });
            match *chosen_suit {
                None => w.u8(0),
                Some(t) => {
                    w.u8(1);
                    w.u8(t);
                }
            }
            enc_opt_pid(w, target);
        }
        MachineSnapshot::LiJian {
            owner,
            stage,
            card,
            inner,
        } => {
            w.u8(41);
            enc_pid(w, owner);
            w.u8(match stage {
                LiJianStageSnap::DiscardAsk => 0,
                LiJianStageSnap::TargetsAsk => 1,
                LiJianStageSnap::Launch => 2,
            });
            enc_opt_u32(w, *card);
            match inner {
                Some(m) => {
                    w.u8(1);
                    super::enc_machine(w, m);
                }
                None => w.u8(0),
            }
        }
        _ => unreachable!("classic skill family routed by machine_encode dispatcher"),
    }
}
