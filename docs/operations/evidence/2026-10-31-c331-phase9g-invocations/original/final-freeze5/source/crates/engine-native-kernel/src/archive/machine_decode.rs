use super::*;

pub(super) fn dec_machine(r: &mut R<'_>) -> Result<MachineSnapshot, ArchiveError> {
    Ok(match r.u8()? {
        87 => MachineSnapshot::JudgeContinuation {
            owner: r.u8()?,
            queue: r.vec(|r| r.u32())?,
            ordinary: dec_opt_u32(r)?,
            started: r.boolean()?,
            current: if r.boolean()? {
                Some(Box::new(dec_judgement_progress(r)?))
            } else {
                None
            },
            payment: if r.boolean()? {
                Some(crate::snap::JudgeReplacementSnap {
                    player: r.u8()?,
                    old: r.u32()?,
                    discard_old: r.boolean()?,
                })
            } else {
                None
            },
        },
        86 => MachineSnapshot::NativeInvocation {
            owner: r.u8()?,
            def: r.u16()?,
            window: crate::event::DamageWindowKind::from_tag(r.u8()?)
                .ok_or(ArchiveError::BadTag("invocation_window", 0))?,
            progress: super::invocation::dec_progress(r)?,
        },
        85 => MachineSnapshot::NativeDamageSkill {
            owner: r.u8()?,
            def: r.u16()?,
            window: crate::event::DamageWindowKind::from_tag(r.u8()?)
                .ok_or(ArchiveError::BadTag("native_damage_window", 0))?,
        },
        84 => MachineSnapshot::DamageFlow { root: r.u32()? },
        83 => MachineSnapshot::TriggerOrder {
            frame: r.u32()?,
            owner: dec_pid(r)?,
            candidates: r.vec(|r| r.u16())?,
            asked: r.boolean()?,
            cursor: r.u16()?,
            done: r.vec(|r| Ok((dec_pid(r)?, r.u16()?)))?,
            owners: r.vec(dec_pid)?,
            normal_owners: r.u16()?,
        },
        0 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let target = dec_pid(r)?;
            let nature = dec_damage_nature(r)?;
            let asked = r.boolean()?;
            let armor_judged = r.boolean()?;
            let dodges_used = r.u8()?;
            let tieji_red = match r.u8()? {
                0 => None,
                1 => Some(false),
                2 => Some(true),
                t => return Err(ArchiveError::BadTag("tieji_red", t)),
            };
            let liuli_done = r.boolean()?;
            let liuli_stage = r.u8()?;
            let liuli_card = match r.u8()? {
                0 => None,
                1 => Some(r.u32()?),
                t => return Err(ArchiveError::BadTag("sha_liuli_card", t)),
            };
            let hujia_queue = match r.u8()? {
                0 => None,
                1 => Some(r.vec(dec_pid)?),
                t => return Err(ArchiveError::BadTag("hujia_queue", t)),
            };
            let hujia_holder = dec_opt_pid(r)?;
            let jiu_bonus = r.boolean()?;
            let asking_qilin = r.boolean()?;
            let qilin_horse = match r.u8()? {
                0 => None,
                1 => Some(r.u32()?),
                t => return Err(ArchiveError::BadTag("sha_qilin_horse", t)),
            };
            let asking_hanbing = r.boolean()?;
            let asking_fengxueren = r.boolean()?;
            let asking_guanshi = r.boolean()?;
            let guanshi_cards = r.vec(|r| r.u32())?;
            let asking_qinglong = r.boolean()?;
            let asking_cixiong = r.boolean()?;
            let cixiong_done = r.boolean()?;
            let extra_targets = r.vec(|r| r.u32())?;
            let hanbing_pending = r.u8()?;
            let asking_xiandan = r.boolean()?;
            let xiandan_done = r.boolean()?;
            let xiandan_bonus = r.boolean()?;
            let asking_lueying = r.boolean()?;
            let lueying_victim = match r.u8()? {
                0 => None,
                1 => Some(r.u32()?),
                _ => return Err(ArchiveError::UnknownCardKind(255)),
            };
            let lueying_declined = r.boolean()?;
            let asking_fuchou = r.boolean()?;
            let fuchou_card = dec_opt_u32(r)?;
            // v75（swd 卡牌面窗口机批）：萨登荆环两询问挂起旗（尾部追加）
            let asking_sadeng = r.boolean()?;
            let asking_sadeng_shan = r.boolean()?;
            // v78（swd deferred 消解批 95）：狩猎不可闪避旗（尾部追加）
            let shoulie_direct = r.boolean()?;
            MachineSnapshot::Sha {
                card,
                source,
                target,
                nature,
                asked,
                armor_judged,
                dodges_used,
                tieji_red,
                liuli_done,
                liuli_stage,
                liuli_card,
                hujia_queue,
                hujia_holder,
                jiu_bonus,
                asking_qilin,
                qilin_horse,
                asking_hanbing,
                asking_fengxueren,
                asking_guanshi,
                guanshi_cards,
                asking_qinglong,
                asking_cixiong,
                cixiong_done,
                extra_targets,
                hanbing_pending,
                asking_xiandan,
                xiandan_done,
                xiandan_bonus,
                asking_lueying,
                lueying_victim,
                lueying_declined,
                asking_fuchou,
                fuchou_card,
                asking_sadeng,
                asking_sadeng_shan,
                shoulie_direct,
            }
        }
        1 => {
            let dying = dec_pid(r)?;
            let source = dec_opt_pid(r)?;
            let responders = dec_pids(r)?;
            let asked = r.boolean()?;
            let round_used = r.boolean()?;
            let guihan_asked = r.boolean()?;
            MachineSnapshot::DyingSave {
                dying,
                source,
                responders,
                asked,
                round_used,
                guihan_asked,
            }
        }
        2 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let target = dec_pid(r)?;
            let on_target = r.boolean()?;
            let asked = r.boolean()?;
            let second_pending = r.boolean()?;
            MachineSnapshot::Duel {
                card,
                source,
                target,
                on_target,
                asked,
                second_pending,
            }
        }
        3 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let nullified = r.boolean()?;
            let queue = dec_pids(r)?;
            let asked = r.boolean()?;
            let window_open = r.boolean()?;
            let owns_disposal = r.boolean()?;
            let effect = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_machine(r)?)),
                t => return Err(ArchiveError::BadTag("opt_machine", t)),
            };
            MachineSnapshot::NullifyWindow {
                card,
                source,
                nullified,
                queue,
                asked,
                window_open,
                owns_disposal,
                effect,
            }
        }
        4 => {
            let owner = dec_pid(r)?;
            let queue = r.vec(|r| r.u32())?;
            let current = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_judgement_progress(r)?)),
                t => return Err(ArchiveError::BadTag("opt_judge", t)),
            };
            MachineSnapshot::JudgePhase {
                owner,
                queue,
                current,
            }
        }
        5 => MachineSnapshot::Noop,
        6 => {
            let card = r.u32()?;
            let target = dec_pid(r)?;
            let placed = r.boolean()?;
            MachineSnapshot::PlaceDelayed {
                card,
                target,
                placed,
            }
        }
        7 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let need = card_kind_from_num(r.u32()?)?;
            let targets = r.vec(|r| r.u8())?;
            let idx = r.u64()? as usize;
            let inner = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_machine(r)?)),
                t => return Err(ArchiveError::BadTag("opt_machine", t)),
            };
            let nature = nature_from_num(r.u8()?)?;
            MachineSnapshot::Aoe {
                card,
                source,
                need,
                targets,
                idx,
                inner,
                nature,
            }
        }
        8 => {
            let source = dec_pid(r)?;
            let target = dec_pid(r)?;
            let need = card_kind_from_num(r.u32()?)?;
            let asked = r.boolean()?;
            let card = r.u32()?;
            let nature = nature_from_num(r.u8()?)?;
            MachineSnapshot::AoeTarget {
                source,
                target,
                need,
                asked,
                card,
                nature,
            }
        }
        9 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let targets = r.vec(|r| r.u8())?;
            let idx = r.u64()? as usize;
            let inner = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_machine(r)?)),
                t => return Err(ArchiveError::BadTag("opt_machine", t)),
            };
            let dispatcher = r.boolean()?;
            MachineSnapshot::Chain {
                card,
                source,
                targets,
                idx,
                inner,
                dispatcher,
            }
        }
        10 => {
            let player = dec_pid(r)?;
            let need = r.u64()? as usize;
            let asked = r.boolean()?;
            MachineSnapshot::DiscardToLimit {
                player,
                need,
                asked,
            }
        }
        11 => {
            let player = dec_pid(r)?;
            let killer = dec_opt_pid(r)?;
            MachineSnapshot::Death { player, killer }
        }
        12 => {
            let owner = dec_pid(r)?;
            let def = dec_skill(r)?;
            let event = dec_event_data(r)?;
            let asked = r.boolean()?;
            let inner = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_machine(r)?)),
                t => return Err(ArchiveError::BadTag("opt_machine", t)),
            };
            MachineSnapshot::OptionalGate {
                owner,
                def,
                event,
                asked,
                inner,
            }
        }
        13 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => SanCaiStageSnap::Draw,
                1 => SanCaiStageSnap::Target,
                2 => SanCaiStageSnap::GiveCard,
                3 => SanCaiStageSnap::Finish,
                t => return Err(ArchiveError::BadTag("sancai_stage", t)),
            };
            let target = dec_opt_pid(r)?;
            MachineSnapshot::SanCai {
                owner,
                stage,
                target,
            }
        }
        14 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => YiShangStageSnap::Target,
                1 => YiShangStageSnap::Apply,
                t => return Err(ArchiveError::BadTag("yishang_stage", t)),
            };
            MachineSnapshot::YiShang { owner, stage }
        }
        15 => MachineSnapshot::SuiDun,
        16 => {
            let target = dec_pid(r)?;
            MachineSnapshot::SuQing { target }
        }
        17 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::ShouYi { owner }
        }
        18 => {
            let owner = dec_pid(r)?;
            let source = dec_opt_pid(r)?;
            let asked = r.boolean()?;
            MachineSnapshot::FanKui {
                owner,
                source,
                asked,
            }
        }
        19 => {
            let owner = dec_pid(r)?;
            let card = r.u32()?;
            let asked = r.boolean()?;
            MachineSnapshot::JianXiong { owner, card, asked }
        }
        20 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => JiJiStageSnap::Draw,
                1 => JiJiStageSnap::Target,
                2 => JiJiStageSnap::Retarget,
                3 => JiJiStageSnap::GiveCard,
                4 => JiJiStageSnap::GiveChooseCard,
                t => return Err(ArchiveError::BadTag("jiji_stage", t)),
            };
            let target = dec_opt_pid(r)?;
            let given = r.u8()?;
            MachineSnapshot::JiJi {
                owner,
                stage,
                target,
                given,
            }
        }
        21 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BiYue { owner }
        }
        25 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::JiZhi { owner }
        }
        27 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => ZhiHengStageSnap::Select,
                1 => ZhiHengStageSnap::Draw,
                t => return Err(ArchiveError::BadTag("zhiheng_stage", t)),
            };
            MachineSnapshot::ZhiHeng { owner, stage }
        }
        26 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => KuRouStageSnap::Lose,
                1 => KuRouStageSnap::Draw,
                t => return Err(ArchiveError::BadTag("kurou_stage", t)),
            };
            MachineSnapshot::KuRou { owner, stage }
        }
        22 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::LuoShen { owner }
        }
        28 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::YingZi { owner }
        }
        29 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let target = dec_pid(r)?;
            let asked = r.boolean()?;
            MachineSnapshot::Dismantle {
                card,
                source,
                target,
                asked,
            }
        }
        43 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::XiaoJi { owner }
        }
        44 => {
            let card = r.u32()?;
            let user = dec_pid(r)?;
            let holders = r.vec(|r| dec_pid(r))?;
            let asked = r.boolean()?;
            MachineSnapshot::MonkeySteal {
                card,
                user,
                holders,
                asked,
            }
        }
        30 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => QingNangStageSnap::TargetAsk,
                1 => QingNangStageSnap::TargetRead,
                2 => QingNangStageSnap::CardRead,
                t => return Err(ArchiveError::BadTag("qingnang_stage", t)),
            };
            let healed = dec_opt_pid(r)?;
            MachineSnapshot::QingNang {
                owner,
                stage,
                healed,
            }
        }
        31 => {
            let owner = dec_pid(r)?;
            let source = dec_pid(r)?;
            let asked = r.u8()? != 0;
            MachineSnapshot::GangLie {
                owner,
                source,
                asked,
            }
        }
        45 => {
            let owner = dec_pid(r)?;
            let target = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => XiaoGuoStageSnap::Ask,
                1 => XiaoGuoStageSnap::Read,
                other => return Err(ArchiveError::BadTag("xiao_guo_stage", other)),
            };
            MachineSnapshot::XiaoGuo {
                owner,
                target,
                stage,
            }
        }
        46 => {
            let owner = dec_pid(r)?;
            let target = dec_pid(r)?;
            MachineSnapshot::KuangFu { owner, target }
        }
        47 => {
            let owner = dec_pid(r)?;
            let source = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => YaoWuStageSnap::Ask,
                1 => YaoWuStageSnap::Read,
                other => return Err(ArchiveError::BadTag("yao_wu_stage", other)),
            };
            MachineSnapshot::YaoWu {
                owner,
                source,
                stage,
            }
        }
        48 => {
            let owner = dec_pid(r)?;
            let target = dec_opt_pid(r)?;
            let stage = match r.u8()? {
                0 => ShuShenStageSnap::Ask,
                1 => ShuShenStageSnap::Read,
                other => return Err(ArchiveError::BadTag("shu_shen_stage", other)),
            };
            MachineSnapshot::ShuShen {
                owner,
                target,
                stage,
            }
        }
        49 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::ShenZhi { owner }
        }
        50 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => XunXunStageSnap::Undo,
                1 => XunXunStageSnap::Arrange,
                other => return Err(ArchiveError::BadTag("xun_xun_stage", other)),
            };
            let top = r.vec(|r| r.u32())?;
            MachineSnapshot::XunXun { owner, stage, top }
        }
        51 => {
            let owner = dec_pid(r)?;
            let other = dec_pid(r)?;
            MachineSnapshot::WangXi { owner, other }
        }
        52 => {
            let owner = dec_pid(r)?;
            let card = match r.u8()? {
                0 => None,
                1 => Some(r.u32()?),
                other => return Err(ArchiveError::BadTag("ji_jie_card", other)),
            };
            let stage = match r.u8()? {
                0 => JiJieStageSnap::Ask,
                1 => JiJieStageSnap::Read,
                other => return Err(ArchiveError::BadTag("ji_jie_stage", other)),
            };
            MachineSnapshot::JiJie { owner, card, stage }
        }
        53 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::JiYuan { owner }
        }
        54 => {
            let owner = dec_pid(r)?;
            let target = dec_pid(r)?;
            MachineSnapshot::WangZun { owner, target }
        }
        55 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => TongJiStageSnap::Ask,
                1 => TongJiStageSnap::Read,
                other => return Err(ArchiveError::BadTag("tong_ji_stage", other)),
            };
            MachineSnapshot::TongJi { owner, stage }
        }
        56 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::Wuzhong { owner }
        }
        57 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::Taoyuan { owner }
        }
        58 => {
            let owner = dec_pid(r)?;
            let cards = r.vec(|r| r.u32())?;
            let pickers = r.vec(dec_pid)?;
            MachineSnapshot::Wugu {
                owner,
                cards,
                pickers,
            }
        }
        60 => {
            let owner = dec_pid(r)?;
            let cards = r.vec(|r| r.u32())?;
            let stage = match r.u8()? {
                0 => ZhangbaStageSnap::Cards,
                1 => ZhangbaStageSnap::Target,
                2 => ZhangbaStageSnap::TargetRead,
                t => return Err(ArchiveError::BadTag("zhangba_stage", t)),
            };
            MachineSnapshot::Zhangba {
                owner,
                cards,
                stage,
            }
        }
        61 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossXiangde { owner }
        }
        62 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossFutai { owner }
        }
        63 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossLuanchang { owner }
        }
        65 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossZhaohuo { owner }
        }
        66 => {
            let owner = dec_pid(r)?;
            let stage = r.u8()?;
            let n = r.u32()?;
            let mut targets = Vec::with_capacity(n as usize);
            for _ in 0..n {
                targets.push(r.u32()?);
            }
            MachineSnapshot::BossBuchun {
                owner,
                stage,
                targets,
            }
        }
        67 => {
            let owner = dec_pid(r)?;
            let stage = r.u8()?;
            let actor = r.u32()?;
            MachineSnapshot::BossXingxia {
                owner,
                stage,
                actor,
            }
        }
        68 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossQizuo { owner }
        }
        69 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossXiaoxiong { owner }
        }
        70 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossLingbo { owner }
        }
        71 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossTashui { owner }
        }
        72 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossZjLeiji { owner }
        }
        73 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossHeisheng { owner }
        }
        74 => {
            let owner = dec_pid(r)?;
            let emitted = r.u8()?;
            MachineSnapshot::BossLeizhu { owner, emitted }
        }
        75 => {
            let owner = dec_pid(r)?;
            let emitted = r.u8()?;
            MachineSnapshot::BossRenao { owner, emitted }
        }
        76 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossWanghun { owner }
        }
        77 => {
            let owner = dec_pid(r)?;
            let emitted = r.u8()?;
            MachineSnapshot::BossZhuxin { owner, emitted }
        }
        78 => {
            let owner = dec_pid(r)?;
            let emitted = r.u8()?;
            MachineSnapshot::BossPingdeng { owner, emitted }
        }
        79 => {
            let owner = dec_pid(r)?;
            let emitted = r.u8()?;
            MachineSnapshot::BossHuoxing { owner, emitted }
        }
        80 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossBingfeng { owner }
        }
        81 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossWangxiang { owner }
        }
        82 => {
            let owner = dec_pid(r)?;
            let module = r.bytes()?.to_vec();
            let paused = match r.u8()? {
                0 => None,
                1 => {
                    let frames = r.bytes()?.to_vec();
                    let pc = r.u32()?;
                    let fuel_remaining = r.u32()?;
                    let id = r.u64()?;
                    let raw = r.bytes()?;
                    if raw.len() != 32 {
                        return Err(ArchiveError::UnexpectedEof);
                    }
                    let mut module_hash = [0u8; 32];
                    module_hash.copy_from_slice(raw);
                    let n = r.u32()? as usize;
                    let mut options = Vec::with_capacity(n.min(1024));
                    for _ in 0..n {
                        options.push(r.u32()?);
                    }
                    let pending = r.bytes()?.to_vec();
                    Some(crate::snap::RvmPaused {
                        frames,
                        pc,
                        fuel_remaining,
                        id,
                        module_hash,
                        options,
                        pending,
                    })
                }
                _ => return Err(ArchiveError::BadTag("Rvm.paused", 255)),
            };
            // B7（规则 v324）：技能身份/触发语境/限次/命中旗标（尾部追加；
            // v324 前的旧档在此读完即止 → UnexpectedEof，不能可靠恢复）。
            let skill_name_bytes = r.bytes()?;
            let skill_name = std::str::from_utf8(skill_name_bytes)
                .map_err(|_| ArchiveError::BadTag("Rvm.skill_name", 0))?
                .to_string();
            let event = dec_event_data(r)?;
            let n_consume = r.u32()? as usize;
            let mut consume = Vec::with_capacity(n_consume.min(1024));
            for _ in 0..n_consume {
                let key_bytes = r.bytes()?;
                let key = std::str::from_utf8(key_bytes)
                    .map_err(|_| ArchiveError::BadTag("Rvm.consume_key", 0))?
                    .to_string();
                let scope = r.u8()?;
                consume.push((key, scope));
            }
            let count_hits = r.boolean()?;
            // J1（规则 v328）：提交期挂起（尾部追加——旧格式档在此读完即止，
            // 读到 EOF 拒绝；v327 及更早档已被规则版本门禁先行拒绝）。
            let commit_pending = r.bytes()?.to_vec();
            let commit_pick = match r.u8()? {
                0 => None,
                1 => {
                    let player = r.u8()?;
                    let count = r.u32()?;
                    Some((player, count))
                }
                _ => return Err(ArchiveError::BadTag("Rvm.commit_pick", 255)),
            };
            let n_following = r.u32()? as usize;
            if n_following > 256 {
                return Err(ArchiveError::BadTag("Rvm.following", 255));
            }
            let mut following = Vec::new();
            for _ in 0..n_following {
                following.push(dec_machine(r)?);
            }
            let n_state = r.u32()? as usize;
            if n_state > 128 {
                return Err(ArchiveError::BadTag("Rvm.commit_state", 255));
            }
            let mut commit_state = Vec::new();
            for _ in 0..n_state {
                commit_state.push(r.u32()?);
            }
            let committing = r.boolean()?;
            MachineSnapshot::Rvm {
                owner,
                module,
                paused,
                skill_name,
                event,
                consume,
                count_hits,
                commit_pending,
                commit_pick,
                following,
                commit_state,
                committing,
            }
        }
        64 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::Shedu { owner }
        }
        59 => {
            let owner = dec_pid(r)?;
            let holder = dec_pid(r)?;
            let victim = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => JiedaoStageSnap::Ask,
                1 => JiedaoStageSnap::Read,
                other => return Err(ArchiveError::BadTag("jie_dao_stage", other)),
            };
            MachineSnapshot::Jiedao {
                owner,
                holder,
                victim,
                stage,
            }
        }
        32 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => TuXiStageSnap::Ask,
                1 => TuXiStageSnap::Read,
                t => return Err(ArchiveError::BadTag("tuxi_stage", t)),
            };
            MachineSnapshot::TuXi { owner, stage }
        }
        33 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::LuoYi { owner }
        }
        34 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => GuanXingStageSnap::Arrange,
                1 => GuanXingStageSnap::Place,
                t => return Err(ArchiveError::BadTag("guanxing_stage", t)),
            };
            let n = r.len()?;
            let mut top = Vec::with_capacity(n);
            for _ in 0..n {
                top.push(r.u32()?);
            }
            MachineSnapshot::GuanXing { owner, stage, top }
        }
        35 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::LianYing { owner }
        }
        36 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => RendeStageSnap::SelectCardsAsk,
                1 => RendeStageSnap::TargetAsk,
                2 => RendeStageSnap::TargetRead,
                t => return Err(ArchiveError::BadTag("rende_stage", t)),
            };
            let n = r.len()?;
            let mut cards = Vec::with_capacity(n);
            for _ in 0..n {
                cards.push(r.u32()?);
            }
            MachineSnapshot::Rende {
                owner,
                stage,
                cards,
            }
        }
        37 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => JijiangStageSnap::HelperAsk,
                1 => JijiangStageSnap::TargetAsk,
                t => return Err(ArchiveError::BadTag("jijiang_stage", t)),
            };
            let queue = match r.u8()? {
                0 => None,
                1 => Some(r.vec(dec_pid)?),
                t => return Err(ArchiveError::BadTag("jijiang_queue", t)),
            };
            let helper = dec_opt_pid(r)?;
            let sha = dec_opt_u32(r)?;
            let inner = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_machine(r)?)),
                t => return Err(ArchiveError::BadTag("jijiang_inner", t)),
            };
            MachineSnapshot::Jijiang {
                owner,
                stage,
                queue,
                helper,
                sha,
                inner,
            }
        }
        38 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let target = dec_pid(r)?;
            let asked = r.boolean()?;
            MachineSnapshot::Snatch {
                card,
                source,
                target,
                asked,
            }
        }
        39 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => JieYinStageSnap::CardsAsk,
                1 => JieYinStageSnap::TargetAsk,
                2 => JieYinStageSnap::TargetRead,
                t => return Err(ArchiveError::BadTag("jieyin_stage", t)),
            };
            let n = r.len()?;
            let mut cards = Vec::with_capacity(n);
            for _ in 0..n {
                cards.push(r.u32()?);
            }
            MachineSnapshot::JieYin {
                owner,
                stage,
                cards,
            }
        }
        40 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => FanJianStageSnap::TargetAsk,
                1 => FanJianStageSnap::SuitAsk,
                2 => FanJianStageSnap::PickAsk,
                3 => FanJianStageSnap::Consume,
                t => return Err(ArchiveError::BadTag("fanjian_stage", t)),
            };
            let chosen_suit = match r.u8()? {
                0 => None,
                1 => Some(r.u8()?),
                t => return Err(ArchiveError::BadTag("fanjian_chosen_suit", t)),
            };
            let target = dec_opt_pid(r)?;
            MachineSnapshot::FanJian {
                owner,
                stage,
                target,
                chosen_suit,
            }
        }
        41 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => LiJianStageSnap::DiscardAsk,
                1 => LiJianStageSnap::TargetsAsk,
                2 => LiJianStageSnap::Launch,
                t => return Err(ArchiveError::BadTag("lijian_stage", t)),
            };
            let card = dec_opt_u32(r)?;
            let inner = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_machine(r)?)),
                t => return Err(ArchiveError::BadTag("lijian_inner", t)),
            };
            MachineSnapshot::LiJian {
                owner,
                stage,
                card,
                inner,
            }
        }
        42 => {
            let key_bytes = r.bytes()?;
            if key_bytes.len() > 64 {
                return Err(ArchiveError::BadTag("host_def_key_len", 0));
            }
            let def_key = std::str::from_utf8(key_bytes)
                .map_err(|_| ArchiveError::BadTag("host_def_key_utf8", 0))?
                .to_string();
            let behavior = r.u16()?;
            let stage = r.u8()?;
            let owner = dec_pid(r)?;
            let target = dec_opt_pid(r)?;
            let card = dec_opt_u32(r)?;
            let cards = r.vec(|r| r.u32())?;
            let targets = r.vec(dec_pid)?;
            MachineSnapshot::Host {
                def_key,
                behavior,
                stage,
                owner,
                target,
                card,
                cards,
                targets,
            }
        }
        t => return Err(ArchiveError::BadTag("machine", t)),
    })
}
