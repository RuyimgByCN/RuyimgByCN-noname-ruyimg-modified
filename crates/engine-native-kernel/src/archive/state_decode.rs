use super::*;

/// R05（存档 107，G 批接线）：detached 实体区（table 之后、rng 之前）。
pub(super) fn dec_detached(r: &mut R<'_>) -> Result<Vec<(CardId, DetachedKind)>, ArchiveError> {
    let len = r.len()? as usize;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        let card = r.u32()?;
        let kind = DetachedKind::from_tag(r.u8()?)?;
        out.push((card, kind));
    }
    Ok(out)
}

pub(super) fn dec_snapshot(r: &mut R<'_>) -> Result<StateSnapshot, ArchiveError> {
    let revision = r.u64()?;
    let ruleset_version = r.u64()?;
    let mode = match r.u8()? {
        0 => GameMode::Prototype,
        1 => {
            let roles = r.vec(dec_role)?;
            GameMode::Identity { roles }
        }
        2 => GameMode::Kingdoms,
        3 => {
            let teams = r.vec(|r| r.vec(dec_pid))?;
            let substitutes = r.vec(|r| r.vec(|r| r.u16()))?;
            GameMode::Versus { teams, substitutes }
        }
        4 => GameMode::Doudizhu { lord: dec_pid(r)? },
        5 => {
            let boss = dec_pid(r)?;
            let chongzheng = r.u8()?;
            let fellows = r.vec(dec_pid)?;
            let stage = dec_boss_stage(r)?;
            GameMode::Boss {
                boss,
                chongzheng,
                fellows,
                stage,
            }
        }
        6 => {
            let p0 = r.vec(|r| r.u16())?;
            let p1 = r.vec(|r| r.u16())?;
            let w0 = r.u32()?;
            let w1 = r.u32()?;
            let games = r.u32()?;
            GameMode::Kof {
                pools: [p0, p1],
                wins: [w0, w1],
                games,
            }
        }
        t => return Err(ArchiveError::BadTag("mode", t)),
    };
    let ring_len = r.len()?;
    let mut seat_ring = Vec::with_capacity(ring_len);
    for _ in 0..ring_len {
        seat_ring.push(dec_pid(r)?);
    }
    let players = r.vec(|r| {
        let alive = r.boolean()?;
        let hp = r.i32()?;
        let max_hp = r.u32()?;
        let hand = r.vec(|r| r.u32())?;
        let equips = r.vec(|r| r.u32())?;
        let judge = r.vec(|r| r.u32())?;
        let dying = r.boolean()?;
        let linked = r.boolean()?;
        // 双将槽与明置状态（国战 M1 数据模型，随存档往返）
        let mut general_slots = [None, None];
        for slot in 0..2 {
            general_slots[slot] = r.opt_u16()?;
        }
        let flags = r.u8()?;
        if flags > 3 {
            return Err(ArchiveError::BadTag("kingdom_identity_flags", flags));
        }
        let revealed_generals = [flags & 1 != 0, r.boolean()?];
        let (kingdom_identity, yexinjia_friends) = if flags & 2 != 0 {
            (
                match r.u8()? {
                    0 => crate::state::KingdomIdentity::Unknown,
                    1 => crate::state::KingdomIdentity::Ye,
                    2 => crate::state::KingdomIdentity::Declared(
                        String::from_utf8(r.bytes()?.to_vec())
                            .map_err(|_| ArchiveError::BadTag("kingdom_identity_utf8", 0))?,
                    ),
                    tag => return Err(ArchiveError::BadTag("kingdom_identity", tag)),
                },
                dec_pids(r)?,
            )
        } else {
            (crate::state::KingdomIdentity::Unknown, Vec::new())
        };
        let wunature = r.opt_u8()?;
        let ctianfu_source = r.opt_u8()?;
        let ctianfu_card = match r.u8()? {
            0 => None,
            1 => Some(r.u32()?),
            t => return Err(ArchiveError::BadTag("player_ctianfu_card", t)),
        };
        let gouhunluo_rounds = r.u32()?;
        let gouhunluo_source = r.opt_u8()?;
        let hujia = r.u8()?;
        let face_down = r.boolean()?;
        let boss_chongzheng = r.u8()?;
        let zc_count = r.len()?;
        let mut zhucheng = Vec::new();
        for _ in 0..zc_count {
            zhucheng.push(r.u32()? as crate::ids::CardId);
        }
        let jy_count = r.len()?;
        let mut jiguanyuan_cards = Vec::new();
        for _ in 0..jy_count {
            jiguanyuan_cards.push(r.u32()? as crate::ids::CardId);
        }
        let jileng_active = r.boolean()?;
        let chaofeng_active = r.boolean()?;
        let xingtigaizao_count = r.u8()?;
        let yinmi_active = r.boolean()?;
        let cm_count = r.len()?;
        let mut card_mods = std::collections::BTreeMap::new();
        for _ in 0..cm_count {
            let card = r.u32()?;
            let attached = r.u32()?;
            card_mods.insert(card, attached);
        }
        let paotai_lv = r.u8()?;
        let baolei_mode = r.u8()?;
        let baoxue_used = r.boolean()?;
        let guihan_used = r.boolean()?;
        let xuehuang_used = r.boolean()?;
        let xuejibingbao = r.u8()?;
        let store_len = r.len()?;
        let mut lianyaohu_store = Vec::with_capacity(store_len);
        for _ in 0..store_len {
            lianyaohu_store.push(r.u32()?);
        }
        let nigong_lv = r.u8()?;
        let jidong_active = r.boolean()?;
        let qianggu_active = r.boolean()?;
        let pingzhang_self_used = r.boolean()?;
        let pingzhang_other_used = r.boolean()?;
        let pingzhang_count = r.u8()?;
        let yinshen_active = r.boolean()?;
        let jijia_hp = r.u8()?;
        let feng_count = r.u8()?;
        let ya_count = r.u8()?;
        let song_count = r.u8()?;
        let anwugu_count = r.u8()?;
        let jiangcheng_longhuo = r.boolean()?;
        let yuexing_pair = r.opt_u8()?;
        let turn_damage = r.u8()?;
        let linghuo_ready_round = r.u32()?;
        let guijin_ready_round = r.u32()?;
        let skip_next_draw = r.boolean()?;
        let guanri_used = r.boolean()?;
        let off_court_len = r.len()?;
        let mut off_court = Vec::with_capacity(off_court_len);
        for _ in 0..off_court_len {
            off_court.push(r.u32()?);
        }
        let shuiyun_count = r.u8()?;
        let wangyou3_round = r.u32()?;
        let total_used = r.u32()?;
        let lingquan_awakened = r.boolean()?;
        let shenwu_awakened = r.boolean()?;
        let chengxin_active = r.boolean()?;
        let chengxin_ready_round = r.u32()?;
        let husha_mark = r.u8()?;
        let hyunshen_count = r.u8()?;
        let daixing_count = r.u8()?;
        let juxi_count = r.u8()?;
        let pozhou_count = r.u8()?;
        let sxianjing_n = r.len()?;
        let mut sxianjing_traps = Vec::with_capacity(sxianjing_n);
        for _ in 0..sxianjing_n {
            sxianjing_traps.push(r.u32()?);
        }
        let death_cancelled = r.boolean()?;
        let shiying_used = r.boolean()?;
        let huahu_used = r.boolean()?;
        let xuanning_mark = r.u32()?;
        let shedu = r.u8()?;
        let juyuan_storage = r.i32()?;
        let fengyin_active = r.boolean()?;
        let lingqu_count = r.u8()?;
        let xingqiu_toggled = r.boolean()?;
        let zlfanshi_terra = r.boolean()?;
        let wangshi_ban = r.boolean()?;
        let fuchou_len = r.len()?;
        let mut fuchou_marked = Vec::with_capacity(fuchou_len);
        for _ in 0..fuchou_len {
            fuchou_marked.push(r.u8()?);
        }
        let buchun_used_round = r.u32()?;
        let xingxia_used_round = r.u32()?;
        let nbianshen_toggle = r.boolean()?;
        let boss_mad = r.boolean()?;
        let jinlin_used = r.boolean()?;
        let hlss_stars = r.u8()?;
        let hlss_count = r.u8()?;
        let hlss_damage = r.u8()?;
        let fengqi_len = r.len()?;
        let mut hlss_fengqi_used = Vec::with_capacity(fengqi_len);
        for _ in 0..fengqi_len {
            hlss_fengqi_used.push(r.u32()? as u16);
        }
        let mbqianlong_marks = r.u8()?;
        let niepan_used = r.boolean()?;
        let shenyan_used = r.boolean()?;
        let shenyan_pending = r.vec(|r| r.u8())?;
        let skip_discard_phase = r.boolean()?;
        let yishan_lost = r.vec(|r| r.u32())?;
        let shouyin_used = r.boolean()?;
        let songci_targets = r.vec(|r| r.u8())?;
        let jilve_used = r.u8()?;
        let swdtianshu_from = match r.u8()? {
            0 => None,
            1 => Some(r.u8()?),
            t => return Err(ArchiveError::BadTag("swdtianshu_from", t)),
        };
        let swdtianshu_granted = match r.u8()? {
            0 => None,
            1 => Some(r.u16()?),
            t => return Err(ArchiveError::BadTag("swdtianshu_granted", t)),
        };
        let mailun_mode = r.u8()?;
        let mailun_out_used = r.boolean()?;
        let mailun_in_used = r.boolean()?;
        let hutian_deposit = match r.u8()? {
            0 => None,
            1 => Some(r.u8()?),
            t => return Err(ArchiveError::BadTag("hutian_deposit", t)),
        };
        let hutian_just_given = r.boolean()?;
        let hutian_hold = r.vec(|r| r.u32())?;
        let hutian_owner = match r.u8()? {
            0 => None,
            1 => Some(r.u8()?),
            t => return Err(ArchiveError::BadTag("hutian_owner", t)),
        };
        Ok(crate::state::PlayerState {
            alive,
            hp,
            max_hp,
            hand,
            equips,
            judge,
            dying,
            hujia,
            linked,
            general_slots,
            revealed_generals,
            kingdom_identity,
            yexinjia_friends,
            wunature,
            ctianfu_source,
            ctianfu_card,
            gouhunluo_rounds,
            gouhunluo_source,
            card_mods,
            jiguanyuan_cards,
            jileng_active,
            chaofeng_active,
            xingtigaizao_count,
            yinmi_active,
            face_down,
            boss_chongzheng,
            zhucheng,
            paotai_lv,
            baolei_mode,
            baoxue_used,
            guihan_used,
            xuehuang_used,
            xuejibingbao,
            lianyaohu_store,
            nigong_lv,
            jidong_active,
            qianggu_active,
            pingzhang_self_used,
            pingzhang_other_used,
            pingzhang_count,
            yinshen_active,
            jijia_hp,
            feng_count,
            ya_count,
            song_count,
            anwugu_count,
            jiangcheng_longhuo,
            yuexing_pair,
            turn_damage,
            linghuo_ready_round,
            guijin_ready_round,
            skip_next_draw,
            guanri_used,
            death_cancelled,
            shiying_used,
            huahu_used,
            xuanning_mark,
            shedu,
            juyuan_storage,
            fengyin_active,
            lingqu_count,
            xingqiu_toggled,
            zlfanshi_terra,
            wangshi_ban,
            fuchou_marked,
            buchun_used_round,
            xingxia_used_round,
            nbianshen_toggle,
            boss_mad,
            jinlin_used,
            hlss_stars,
            hlss_count,
            hlss_damage,
            hlss_fengqi_used,
            mbqianlong_marks,
            niepan_used,
            shenyan_used,
            shenyan_pending,
            skip_discard_phase,
            yishan_lost,
            shouyin_used,
            songci_targets,
            jilve_used,
            swdtianshu_from,
            swdtianshu_granted,
            mailun_mode,
            mailun_out_used,
            mailun_in_used,
            hutian_deposit,
            hutian_just_given,
            hutian_hold,
            hutian_owner,
            off_court,
            shuiyun_count,
            wangyou3_round,
            total_used,
            lingquan_awakened,
            shenwu_awakened,
            chengxin_active,
            chengxin_ready_round,
            husha_mark,
            hyunshen_count,
            daixing_count,
            juxi_count,
            pozhou_count,
            sxianjing_traps,
        })
    })?;
    let deck = r.vec(|r| r.u32())?;
    let discard = r.vec(|r| r.u32())?;
    let table = r.vec(|r| r.u32())?;
    // R05（存档 107，G 批）：table 之后、rng 之前——与编码序严格对应。
    let detached_cards = dec_detached(r)?;
    let rng_state = r.u64()?;
    let rng_calls = r.u64()?;
    let current = dec_pid(r)?;
    let phase = dec_phase(r)?;
    let round = r.u32()?;
    let sha_used_play_phase = r.boolean()?;
    let mufeng_used_basic = r.boolean()?;
    let wangchen_discard_basic = r.boolean()?;
    // v75（swd 卡牌面窗口机批）：神秘果/鼠槌三旗（尾部追加）
    let shenmiguo_used = r.boolean()?;
    let shuchui_armed = r.boolean()?;
    let shuchui_hit = r.boolean()?;
    let lianda_used = r.boolean()?;
    let land_used = r.boolean()?;
    let land_sha_used = r.boolean()?;
    let discard_phase_discarded = r.u32()?;
    let cq_count = r.len()?;
    let mut ciqiu_dying = Vec::new();
    for _ in 0..cq_count {
        ciqiu_dying.push(r.u8()? as PlayerId);
    }
    let kangyin2 = match r.u8()? {
        0 => None,
        1 => Some((r.u8()? as PlayerId, r.u32()?)),
        _ => return Err(ArchiveError::UnknownCardKind(255)),
    };
    let huanxia_len = r.len()?;
    let mut huanxia_track = Vec::with_capacity(huanxia_len);
    for _ in 0..huanxia_len {
        huanxia_track.push((r.u32()?, r.boolean()?));
    }
    let cards_used_play_phase = r.u32()?;
    let cards_used_turn = r.u32()?;
    let liaoyuan_count = r.u8()?;
    let xuanyuanjian_marked = r.boolean()?;
    let mut kunlunjing_snapshot = Vec::new();
    for _ in 0..r.len()? {
        let p = r.u8()?;
        let hand = r.vec(|r| r.u32())?;
        let equips = r.vec(|r| r.u32())?;
        let judge = r.vec(|r| r.u32())?;
        kunlunjing_snapshot.push((p, hand, equips, judge));
    }
    let kunlunjing_hp = r.u32()? as i32;
    let extra_phases = r.vec(|r| r.u8())?;
    let extra_phase_lingwu = r.boolean()?;
    let bte_used_play_phase = r.u32()?;
    let phase_number = r.u32()?;
    let rende_given = r.u8()?;
    let jiu_used = r.boolean()?;
    let jiu_active = r.boolean()?;
    let turn_opened = r.boolean()?;
    let skip_draw_phase = r.boolean()?;
    let skip_play_phase = r.boolean()?;
    let guxing_armed = r.boolean()?;
    let guxing_count = r.u8()?;
    let mengtai_draw_pending = r.boolean()?;
    let mengtai_use_pending = r.boolean()?;
    let zhufangshenshi_target = r.opt_u8()?;
    let yuedong_num = r.u8()?;
    let yuedong_eff = r.u8()?;
    let yuedong_recover = r.boolean()?;
    let liangou_hook = r.opt_u8()?;
    let liangou_bonus_used = r.boolean()?;
    let feitiao_target = r.opt_u8()?;
    let fanling_used = r.boolean()?;
    let xjyufeng_used = r.u8()?;
    let diewu_draw_used = r.boolean()?;
    let lueying_used = r.boolean()?;
    let txianqu_used = r.boolean()?;
    let jtjubao_used = r.boolean()?;
    let zhuyue_len = r.len()?;
    let mut zhuyue_targets = Vec::with_capacity(zhuyue_len);
    for _ in 0..zhuyue_len {
        zhuyue_targets.push(r.u8()?);
    }
    let sheling_used = r.u8()?;
    let sheying_used = r.boolean()?;
    let zhangmu_used = r.boolean()?;
    let yingfeng_chain = r.boolean()?;
    let mujiaren_used = r.boolean()?;
    let luanjian_card = match r.u8()? {
        0 => None,
        1 => Some(r.u32()?),
        t => return Err(ArchiveError::BadTag("turn_luanjian_card", t)),
    };
    let minwan_targets = r.vec(dec_pid)?;
    let yanyu_count = r.u8()?;
    let qizuo_copy = match r.u8()? {
        0 => None,
        1 => Some(r.u32()?),
        t => return Err(ArchiveError::BadTag("turn_qizuo_copy", t)),
    };
    let jidian_chain = r.boolean()?;
    let skill_count = r.len()?;
    let mut skills = std::collections::BTreeMap::new();
    for _ in 0..skill_count {
        let key_owner = dec_pid(r)?;
        let key_def = dec_skill(r)?;
        let owner = dec_pid(r)?;
        let enabled = r.boolean()?;
        let once_used = r.boolean()?;
        let expires_end_of_turn = r.boolean()?;
        let rounds = r.u8()?;
        skills.insert(
            (key_owner, key_def),
            SkillInstance {
                owner,
                enabled,
                once_used,
                expires_end_of_turn,
                rounds,
            },
        );
    }
    let mut declared_slots = std::collections::BTreeMap::new();
    for _ in 0..r.len()? {
        let card = r.u32()?;
        let slot = match r.u8()? {
            0 => crate::catalog::EquipSlot::Weapon,
            1 => crate::catalog::EquipSlot::Armor,
            2 => crate::catalog::EquipSlot::HorsePlus,
            3 => crate::catalog::EquipSlot::HorseMinus,
            4 => crate::catalog::EquipSlot::Treasure,
            t => return Err(ArchiveError::BadTag("equip_slot", t)),
        };
        declared_slots.insert(card, slot);
    }
    let dk_count = r.len()?;
    let mut declared_kinds = std::collections::BTreeMap::new();
    for _ in 0..dk_count {
        let card = r.u32()?;
        let kind = card_kind_from_num(r.u8()? as u32)?;
        declared_kinds.insert(card, kind);
    }
    let cn_count = r.len()?;
    let mut card_natures = std::collections::BTreeMap::new();
    for _ in 0..cn_count {
        let card = r.u32()?;
        let nature = r.u8()?;
        card_natures.insert(card, nature);
    }
    // 当前地（mtg land，规则 v100）
    let current_land = match r.u8()? {
        0 => None,
        1 => Some(card_kind_from_num(r.u8()? as u32)?),
        _ => return Err(ArchiveError::UnknownCardKind(255)),
    };
    let outcome = match r.u8()? {
        0 => None,
        1 => Some(crate::state::Outcome {
            winner: dec_winner(r)?,
        }),
        t => return Err(ArchiveError::BadTag("opt_outcome", t)),
    };
    let vm_aux = dec_vm_aux(r)?;
    let last_dedup = r.u64()?;
    let segment_open = r.boolean()?;
    let auth_generation = r.u64()?;
    let resolution = dec_resolution(r)?;
    let segment_checkpoint = match r.u8()? {
        0 => None,
        1 => Some(Box::new(dec_snapshot(r)?)),
        t => return Err(ArchiveError::BadTag("opt_snapshot", t)),
    };
    Ok(StateSnapshot {
        ruleset_version,
        core: crate::state::GameStateCore {
            seat_ring,
            revision,
            ruleset_version,
            mode,
            players,
            deck,
            discard,
            table,
            // R05（存档 107，G 批）：detached 实体区随批解码。
            detached_cards,
            rng: crate::rng::RuleRng::from_raw_parts(rng_state, rng_calls),
            turn: crate::state::TurnState {
                current,
                phase,
                round,
                sha_used_play_phase,
                mufeng_used_basic,
                wangchen_discard_basic,
                shenmiguo_used,
                shuchui_armed,
                shuchui_hit,
                lianda_used,
                land_used,
                land_sha_used,
                discard_phase_discarded,
                ciqiu_dying,
                kangyin2,
                huanxia_track,
                cards_used_play_phase,
                cards_used_turn,
                liaoyuan_count,
                xuanyuanjian_marked,
                kunlunjing_snapshot,
                kunlunjing_hp,
                extra_phases,
                extra_phase_lingwu,
                bte_used_play_phase,
                phase_number,
                rende_given,
                jiu_used,
                jiu_active,
                turn_opened,
                skip_draw_phase,
                skip_play_phase,
                guxing_armed,
                guxing_count,
                mengtai_draw_pending,
                mengtai_use_pending,
                zhufangshenshi_target: zhufangshenshi_target.map(|p| p as PlayerId),
                yuedong_num,
                yuedong_eff,
                yuedong_recover,
                liangou_hook: liangou_hook.map(|p| p as PlayerId),
                liangou_bonus_used,
                feitiao_target: feitiao_target.map(|p| p as PlayerId),
                fanling_used,
                xjyufeng_used,
                diewu_draw_used,
                lueying_used,
                txianqu_used,
                jtjubao_used,
                zhuyue_targets,
                sheling_used,
                sheying_used,
                zhangmu_used,
                yingfeng_chain,
                mujiaren_used,
                luanjian_card,
                minwan_targets,
                yanyu_count,
                qizuo_copy,
                jidian_chain,
            },
            skills,
            declared_kinds,
            declared_slots,
            card_natures,
            current_land,
            outcome,
            vm_aux,
        },
        last_dedup,
        auth_generation,
        segment_open,
        segment_checkpoint,
        resolution,
    })
}
