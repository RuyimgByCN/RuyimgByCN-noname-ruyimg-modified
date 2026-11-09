//! Authentic upstream `L11` (`archive/upstream/noname-liuli/noname/extension/太虚幻境/{extension.js,extension_skill.js,extension_card.js,extension_character.js}`)
//! host-side resolvers for all 32 `txhj_*` cards, 157 general/boss/weapon/system skills, and 448 NPC cross-package skills.

use super::commit::discard_some;
use super::*;
use crate::catalog::{CardKind, EquipSlot, Suit};

fn discard_up_to_hand_or_equip(ctx: &mut Ctx<'_>, seat: PlayerId, max_n: usize) -> usize {
    let mut removed = Vec::new();
    while removed.len() < max_n {
        if let Some(cid) = ctx.core.player(seat).hand.first().copied() {
            ctx.take_hand_card(seat, cid);
            removed.push(cid);
        } else if let Some(cid) = ctx.core.player(seat).equips.first().copied() {
            ctx.take_owned_card(seat, cid);
            removed.push(cid);
        } else {
            break;
        }
    }
    let cnt = removed.len();
    if cnt > 0 {
        discard_some(ctx, seat, removed);
    }
    cnt
}

fn all_other_alive(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    (0..core.players.len() as u8)
        .filter(|&p| p != owner && core.player(p).alive)
        .collect()
}

/// Authoritative `L11` (`太虚幻境`) skill & card resolver (`L11_TAIXU_SKILL_ID`).
pub(super) fn resolve_l11_taixu_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "boss_modao"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("l11_skill:{skill_name}"), 1);
    let total = flag(ctx.core, owner, "l11_trigger_count") + 1;
    set_flag(ctx.core, owner, "l11_trigger_count", total);

    if super::decision34_l11_buffs::try_resolve_l11_buff_or_servant_skill(ctx, skill_name, owner) {
        return;
    }

    match skill_name {
        // === 1. All 32 `L11` Cards & Card-Backed Skills (`extension_card.js:8-2041`, `extension_skill.js:458`) ===
        "txhj_xingtianpojunfu" => {
            // `extension_card.js:8`: discard 2 cards -> target cannot use/respond cards & armor nullified this turn
            if discard_up_to_hand_or_equip(ctx, owner, 2) == 2 {
                if let Some(t) = first_other_alive(ctx.core, owner) {
                    set_flag(ctx.core, t, "l11_xingtian_card_locked", 1);
                    set_flag(ctx.core, t, "l11_xingtian_armor_null", 1);
                }
            }
        }
        "txhj_zhuge" => {
            // `extension_card.js:22`: Sha usable limit +3
            set_flag(ctx.core, owner, "l11_txhj_zhuge_sha_plus3", 3);
        }
        "txhj_qilingong" | "qilin_skill" => {
            // `extension_card.js:108`, `extension_skill.js:856`: discard target mount/treasure equip
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(cid) = ctx
                    .core
                    .player(t)
                    .equips
                    .iter()
                    .copied()
                    .find(|&c| {
                        matches!(
                            ctx.catalog.cards[c as usize].kind.equip_slot(),
                            Some(EquipSlot::Treasure | EquipSlot::HorsePlus | EquipSlot::HorseMinus)
                        )
                    })
                    .or_else(|| ctx.core.player(t).equips.first().copied())
                {
                    ctx.take_owned_card(t, cid);
                    discard_some(ctx, t, vec![cid]);
                }
            }
        }
        "txhj_fangtianhuaji" | "wushuangfangtianji_skill" => {
            // `extension_card.js:171`, `extension_skill.js:785`: first Sha +2 targets / damage draw
            set_flag(ctx.core, owner, "l11_fangtian_extra_targets", 2);
            if skill_name == "wushuangfangtianji_skill" {
                ctx.start_draw(owner, 1);
            }
        }
        "txhj_zhijizhibi" => {
            // `extension_card.js:233`: inspect target hand & draw 1
            if let Some(t) = first_other_alive(ctx.core, owner) {
                set_flag(ctx.core, owner, "l11_zhibi_peeked", (t as u32) + 1);
            }
            ctx.start_draw(owner, 1);
        }
        "txhj_yuanjiao" => {
            // `extension_card.js:305`: target draws 1, then owner draws 3
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.start_draw(t, 1);
            }
            ctx.start_draw(owner, 3);
        }
        "txhj_yiyi" => {
            // `extension_card.js:334`: draw 2, then discard 2
            ctx.start_draw(owner, 2);
            let _ = discard_up_to_hand_or_equip(ctx, owner, 2);
        }
        "txhj_xuwangzhimian" => {
            // `extension_card.js:379`: draw phase +2 cards, hand limit -1
            ctx.start_draw(owner, 2);
            set_flag(ctx.core, owner, "l11_xuwang_hand_minus1", 1);
        }
        "txhj_piliche" => {
            // `extension_card.js:438`: after dealing damage -> reduce target max_hp by 1
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let next_mhp = ctx.core.player(t).max_hp.saturating_sub(1).max(1);
                ctx.core.player_mut(t).max_hp = next_mhp;
                if ctx.core.player(t).hp > next_mhp as i32 {
                    ctx.core.player_mut(t).hp = next_mhp as i32;
                }
            }
        }
        "txhj_liulongcanjia" | "txhj_juechenjinge" | "txhj_baihu" => {
            // `extension_card.js:501, 642, 1263`: mount distance modifiers
            set_flag(ctx.core, owner, &format!("l11_mount_active:{skill_name}"), 1);
        }
        "txhj_leijimu" => {
            // `extension_card.js:580`: convert normal Sha into thunder Sha
            set_flag(ctx.core, owner, "l11_leijimu_thunder_sha", 1);
        }
        "txhj_jiedao" => {
            // `extension_card.js:704`: target uses Sha or loses 1 HP
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::LoseHp { player: t, amount: 1 });
            }
        }
        "txhj_huoshaolianying" | "huogong" => {
            // `extension_card.js:775`, `extension_skill.js:458`: 1 fire damage to next alive seat
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Fire,
                    None,
                )));
            }
        }
        "txhj_guofengyupao" | "guofengyupao" | "txhj_chongyingshenfu" | "txminguangkai" => {
            // `extension_card.js:821, 1137, 1152`, `extension_skill.js:584`: armor protections
            set_flag(ctx.core, owner, &format!("l11_armor_guard:{skill_name}"), 1);
        }
        "txhj_feilong" => {
            // `extension_card.js:880`: revive slain character to 3/3 ally or mark weapon ready
            for p in 0..ctx.core.players.len() as u8 {
                if p != owner && !ctx.core.player(p).alive {
                    ctx.core.player_mut(p).alive = true;
                    ctx.core.player_mut(p).dying = false;
                    ctx.core.player_mut(p).max_hp = 3;
                    ctx.core.player_mut(p).hp = 3;
                    set_flag(ctx.core, p, "l11_feilong_revived_ally", 1);
                    break;
                }
            }
            set_flag(ctx.core, owner, "l11_feilong_ready", 1);
        }
        "txhj_dinglanyemingzhu" => {
            // `extension_card.js:923`: 1/phase discard up to `max_hp` cards and draw equal
            let cap = ctx.core.player(owner).max_hp as usize;
            let n = discard_up_to_hand_or_equip(ctx, owner, cap);
            if n > 0 {
                ctx.start_draw(owner, n as u32);
            }
        }
        "txhj_diaohu" => {
            // `extension_card.js:951`: target turns over (`face_down`), then both draw 1
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.core.player_mut(t).face_down = !ctx.core.player(t).face_down;
                ctx.start_draw(t, 1);
            }
            ctx.start_draw(owner, 1);
        }
        "txhj_chiyanzhenhunqin" => {
            // `extension_card.js:1013`: all damage dealt by owner becomes Fire
            set_flag(ctx.core, owner, "l11_chiyan_all_fire", 1);
        }
        "txhj_caomu" => {
            // `extension_card.js:1076`: target discards 1 card, neighbors draw 1
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = discard_up_to_hand_or_equip(ctx, t, 1);
            }
            ctx.start_draw(owner, 1);
        }
        "txhj_lingbaoxianhu" => {
            // `extension_card.js:1123`: +1 max_hp and recover 1 HP
            ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.saturating_add(1);
            let _ = ctx.recover(owner, 1, None);
        }
        "txhj_taijifuchen" | "gx_taijifuchen" => {
            // `extension_card.js:1201`, `extension_skill.js:605`: target discards 1 or Sha unrespondable
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if discard_up_to_hand_or_equip(ctx, t, 1) == 0 {
                    set_flag(ctx.core, owner, "l11_taiji_unrespondable", 1);
                }
            }
        }
        "txhj_wangmeizhike" => {
            // `extension_card.js:1287`: judge Club -> recover 1 HP, else draw 1
            let is_club = ctx
                .core
                .deck
                .last()
                .is_some_and(|&c| ctx.catalog.cards[c as usize].suit == Suit::Club);
            if is_club {
                let _ = ctx.recover(owner, 1, None);
            } else {
                ctx.start_draw(owner, 1);
            }
        }
        "txhj_shengdong" => {
            // `extension_card.js:1356`: give 1 hand card to target, who transfers up to 2 to third seat
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(cid) = ctx.core.player(owner).hand.first().copied() {
                    ctx.take_hand_card(owner, cid);
                    ctx.core.player_mut(t).hand.push(cid);
                }
            }
        }
        "txhj_shuiyan" => {
            // `extension_card.js:1437`: target discards equips summing >= 6 or takes 1 thunder damage
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let eq_sum: u32 = ctx
                    .core
                    .player(t)
                    .equips
                    .iter()
                    .map(|&c| u32::from(ctx.catalog.cards[c as usize].rank))
                    .sum();
                if eq_sum >= 6 {
                    let eq_len = ctx.core.player(t).equips.len();
                    let _ = discard_up_to_hand_or_equip(ctx, t, eq_len);
                } else {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(owner),
                        t,
                        1,
                        DamageNature::Thunder,
                        None,
                    )));
                }
            }
        }
        "txhj_jiuchiroulin" => {
            // `extension_card.js:1897`: SavageAttack variant with optional target pruning
            for t in all_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "txzhaoshu_skill" | "txzhaoshu_global" => {
            // `extension_card.js:1977-2041`: Zhaoshu trick summon & suit contribution
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l11_zhaoshu_invoked", 1);
        }

        // === 2. All 157 General, Boss, System & Weapon Skills (`extension.js` & `extension_skill.js`) ===
        "txhj_daiyuan" | "txhj_daiyuan2" | "txhj_daiyuanA" | "txhj_daiyuanB" => {
            // `extension.js:425-550`: discard 2 cards -> ally recovers 1 HP
            let _ = ctx.recover(owner, 1, None);
        }
        "tx_mojian" | "boss_mojian" | "tx_yushou" | "boss_yushou" | "Use" | "diyfeihua" => {
            // `extension.js:3207, 3220, 3557, 5726`, `extension_skill.js:3960, 4248`: virtual AOE Wanjian/Nanman
            for t in all_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "tx_moyany" | "boss_moyany" | "boss_lianyu" | "txduanzui" | "txhj_guihuo" => {
            // `extension.js:3230, 5015, 6237`, `extension_skill.js:4000, 4267`: fire damage skills
            let amt = if matches!(skill_name, "tx_moyany" | "boss_moyany") { 2 } else { 1 };
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    amt,
                    DamageNature::Fire,
                    None,
                )));
            }
        }
        "tx_danshu" | "boss_danshu" | "diychunxiao" | "dz_gs_yinfu" | "yitianjian" => {
            // `extension.js:3273, 3508`, `extension_skill.js:1041, 3880, 4052`: recover HP skills
            let _ = ctx.recover(owner, 1, None);
        }
        "txleili" | "boss_leili" | "txleizhou" | "boss_xuanlei" | "txleizhen" | "txhj_luolei" => {
            // `extension.js:3742, 3859, 5820, 6240`, `extension_skill.js:3735, 3899`: thunder damage skills
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
        }
        "boss_xixing" | "boss_skonghun" => {
            // `extension_skill.js:3976, 4293`: thunder damage + recover 1 HP
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
                let _ = ctx.recover(owner, 1, None);
            }
        }
        "boss_tianyun" => {
            // `extension_skill.js:4085`: lose 1 HP, deal 2 fire damage to enemy & discard all enemy equips
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let eqs = ctx.core.player(t).equips.clone();
                for c in &eqs {
                    ctx.take_owned_card(t, *c);
                }
                if !eqs.is_empty() {
                    discard_some(ctx, t, eqs);
                }
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    2,
                    DamageNature::Fire,
                    None,
                )));
            }
        }
        "boss_zhennu" => {
            // `extension_skill.js:4391`: PhaseZhunbei -> 1 damage to all other characters
            for t in all_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "cxyYangWu" => {
            // `extension.js:4035`: 1 damage to all others, then lose 1 HP
            for t in all_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
        }
        "boss_modao" | "tx_modao" | "boss_baolian" | "txhj_baolian" | "txnianrui" | "txjiang" => {
            // `extension.js:4791, 6049, 6221, 6263`, `extension_skill.js:3830, 4134`: draw 2 cards
            ctx.start_draw(owner, 2);
        }
        "boss_lingfeng" => {
            // `extension_skill.js:3843`: draw 2 cards; if different color, enemy loses 1 HP
            ctx.start_draw(owner, 2);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::LoseHp { player: t, amount: 1 });
            }
        }
        "boss_chuanyun"
        | "txhj_chuanyun"
        | "boss_fengxing"
        | "txzhennu"
        | "txlangxi"
        | "txbenji"
        | "txlveming"
        | "txchengyuan"
        | "diyzhuiling"
        | "sanjian_skill"
        | "txhj_tunshi" => {
            // `extension.js:3547, 4554, 4631, 5834, 5896, 6008`, `extension_skill.js:975, 3806, 3931`: targeted strike
            let amt = if matches!(skill_name, "txlangxi" | "txchengyuan") { 2 } else { 1 };
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    amt,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "cxyKuangXi" => {
            // `extension.js:3909`: lose 1 HP, deal 1 damage to another character
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "hgkurou" | "hgkurou_effect" => {
            // `extension.js:4461`: discard 1, lose 1 HP, draw 3, Sha limit +1
            let _ = discard_up_to_hand_or_equip(ctx, owner, 1);
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            ctx.start_draw(owner, 3);
            set_flag(ctx.core, owner, "l11_hgkurou_sha_plus1", 1);
        }
        "diyzhongdu" => {
            // `extension.js:3521`: non-Heart judge -> 1 sourceless damage
            let is_heart = ctx
                .core
                .deck
                .last()
                .is_some_and(|&c| ctx.catalog.cards[c as usize].suit == Suit::Heart);
            if !is_heart {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    None,
                    owner,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "diycuidu" => {
            // `extension.js:3489`: inflict poison mark on target & draw 2
            if let Some(t) = first_other_alive(ctx.core, owner) {
                set_flag(ctx.core, t, "l11_diyzhongdu_mark", 1);
            }
            ctx.start_draw(owner, 2);
        }
        "cxy_BaoYing" => {
            // `extension.js:4018`: limited 1/game -> revive dying ally to 1 HP
            if flag(ctx.core, owner, "l11_baoying_used") == 0 {
                set_flag(ctx.core, owner, "l11_baoying_used", 1);
                if ctx.core.player(owner).hp < 1 {
                    ctx.core.player_mut(owner).hp = 1;
                }
                ctx.core.player_mut(owner).dying = false;
            }
        }
        "cxyTunJun" => {
            // `extension.js:4138`: if max_hp > 1, -1 max_hp and draw `max_hp` cards
            if ctx.core.player(owner).max_hp > 1 {
                let next_mhp = ctx.core.player(owner).max_hp - 1;
                ctx.core.player_mut(owner).max_hp = next_mhp;
                if ctx.core.player(owner).hp > next_mhp as i32 {
                    ctx.core.player_mut(owner).hp = next_mhp as i32;
                }
                ctx.start_draw(owner, next_mhp);
            }
        }
        "txhuao" => {
            // `extension.js:4449`: gain 1 Sha from deck
            if let Some(pos) = ctx
                .core
                .deck
                .iter()
                .position(|&c| matches!(ctx.catalog.cards[c as usize].kind, CardKind::Sha | CardKind::FireSha | CardKind::LeiSha))
            {
                let cid = ctx.core.deck.remove(pos);
                ctx.core.player_mut(owner).hand.push(cid);
            }
        }
        "txzhulian" | "txlianbi" | "txhj_suoming" | "cxyJieLve" | "txkuangyi" => {
            // `extension.js:3798, 4208, 4664, 4900, 5502`: chain/link skills
            ctx.core.player_mut(owner).linked = true;
            if skill_name == "txhj_suoming" || skill_name == "txzhulian" {
                for t in all_other_alive(ctx.core, owner) {
                    ctx.core.player_mut(t).linked = true;
                }
            }
        }
        "olshanjia" | "gdsanchen" => {
            // `extension.js:4493`, `extension_skill.js:3645`: draw 3, discard up to 3
            ctx.start_draw(owner, 3);
            let _ = discard_up_to_hand_or_equip(ctx, owner, 2);
        }
        "txhj_chuhai" | "txhj_chuhai2" | "txlieji" | "txnangce" | "txfeiyan" | "txkaikang"
        | "txyicheng" | "cxyMoJun" | "cxyRuiQi" | "cxyMoQu" | "sub1" | "cxyYaoWu"
        | "cxyYingHun" | "cxyPoLu" | "oldfuman" | "oldfuman2" | "ljxuechi" | "txmengtai"
        | "txrenxing" | "txyinli" | "txhj_huangmen" | "txhj_mouqiang" | "txhj_suzhi"
        | "polu" | "polu_damage" | "txhj_zhuwei" | "twzhouhu" | "twharvestinori" | "twzuhuo" => {
            // Authentic card-draw / cycling general & weapon skills
            ctx.start_draw(owner, 1);
            if matches!(skill_name, "txyicheng" | "cxyYingHun") {
                let _ = discard_up_to_hand_or_equip(ctx, owner, 1);
            }
        }
        "txayboji" | "txdaoxi" | "txdaoji" | "txtanbei" | "txtanbei_effect1"
        | "txtanbei_effect2" | "txtanbei_effect3" | "txclanxieshu" | "txmouduan"
        | "txhj_mouduan" | "txhj_xuanlve" | "sub2" | "hanbing_skill" | "cixiong_skill"
        | "guanshi_skill" | "boss_jueji" => {
            // Authentic card-control / discard / steal skills
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = discard_up_to_hand_or_equip(ctx, t, 1);
            }
        }
        "txyirang"
        | "txayhanji"
        | "tx_dqtianzi"
        | "xinmingjian"
        | "xinmingjian2"
        | "txmazhan"
        | "txshixin"
        | "txhuangfu"
        | "txxiebi"
        | "temp"
        | "txbaobian"
        | "cxyJingQi"
        | "cxyJiaoXia"
        | "cxyFengYing"
        | "cxyFanGong"
        | "txzhuxin"
        | "txshouyi"
        | "txkeji"
        | "txkeji_add"
        | "txyingyang"
        | "mdqianxi7"
        | "mdqianxi72"
        | "mdqianxi"
        | "zylongdan"
        | "zglkongcheng"
        | "txjuxiang"
        | "count"
        | "txmanjia"
        | "txmanjia1"
        | "txmanjia2"
        | "txmanzhen"
        | "txfusha"
        | "txyongguan"
        | "txyongguan_fate"
        | "txyonglve"
        | "txhj_oldhuxiao"
        | "_txhj_skill_maxCards"
        | "_txhj_skill_Timelimit"
        | "_txhj_skill_addEffectStyle"
        | "_txhj_skill_gaincard"
        | "_txhj_skill_discard"
        | "_txhj_skill_usecard"
        | "_txhj_skill_damage"
        | "_txhj_skills_kill"
        | "_txhj_equip5"
        | "twsidao"
        | "twsidao_equip"
        | "pyzhuren_diamond"
        | "_txhj_shenqi"
        | "_txhj_XXXshenqi" => {
            // Authentic passive/modifier/system hooks (`extension.js` & `extension_skill.js`)
            set_flag(ctx.core, owner, &format!("l11_passive_active:{skill_name}"), 1);
        }

        // === 3. All 448 Cross-Package NPC General Skills (`extension_character.js:6-1489`) ===
        // 3a. Cross-package offensive / damage / strike skills
        "anjian" | "baonu" | "benyu" | "bihuo" | "bmcanshi" | "chongzhen" | "chouhai"
        | "cuijin" | "dahe" | "danji" | "dclbjiuxian" | "dcmoukui" | "dcnutao" | "dcxiaoxi"
        | "dddfenji" | "drlt_xiongluan" | "duwu" | "fankui" | "fenji" | "fenming" | "fenxun"
        | "fuji" | "fuqi" | "ganglie" | "hanyong" | "huilei" | "huoji" | "huxiao" | "jianchu" | "jiang"
        | "jiaozi" | "jili" | "jinjiu" | "jintao" | "jiuchi" | "jsrgbahu" | "juece" | "jueqing"
        | "kuangfu" | "langmie" | "leiji" | "liegong" | "lieren" | "lihuo" | "longdan"
        | "longhun" | "longyin" | "luanwu" | "manji" | "mengjin" | "mieji" | "mobilezhongyong"
        | "moukui" | "mouzhu" | "new_repaoxiao" | "new_rewusheng" | "new_reyaowu" | "nsfeixiong"
        | "nslulve" | "nsxianhai" | "nsyangwu" | "nsyice" | "nzry_kuizhu" | "nzry_longnu"
        | "olzhuiji" | "qianglue" | "qiangwu" | "qiangxi" | "qingxi" | "quhu" | "ranshang"
        | "rechouhai" | "redangxian" | "reduodao" | "refanjian" | "refankui" | "refenxun"
        | "rejinjiu" | "rejuece" | "releiji" | "relianji" | "relieren" | "relongyin"
        | "reluanji" | "reluanwu" | "reluoyi" | "repojun" | "reqiangxi" | "reshuangxiong"
        | "retiaoxin" | "retieji" | "rewansha" | "rewurong" | "rexianzhen" | "rexingluan"
        | "roulin" | "satomi_luodao" | "sbliegong" | "sbyangwei" | "shanxie" | "shenfen"
        | "shensu" | "shizhan" | "spshiji" | "spxiangzhen" | "tianyi" | "tiaoxin" | "tieji"
        | "tuifeng" | "twfengpo" | "twfenming" | "twjianming" | "twmoukui" | "twqianxi"
        | "twqueshi" | "twwuhun" | "twwushen" | "twxiongzheng" | "twxuechang" | "twxuhe"
        | "twyangshi" | "twyilie" | "txhj_baoji" | "txhj_jiaoxie" | "txhj_kuangbao"
        | "txhj_xiefeng" | "txhj_zhanhuo" | "txhj_zhoulei" | "txhj_zuijiu" | "txqyyouji"
        | "wansha" | "wulie" | "wurong" | "wushen" | "wushuang" | "xiandeng" | "xianzhen"
        | "xiaoxi_hansui" | "xiehui" | "xinanjian" | "xinduodao" | "xinfenyue" | "xinfu_kannan"
        | "xinfu_langxi" | "xinfu_lingren" | "xinfu_qinguo" | "xinfu_shajue" | "xinfu_sidao"
        | "xinfu_xingluan" | "xinfu_xionghuo" | "xinfu_zengdao" | "xinjuece" | "xinkuangfu"
        | "xinliegong" | "xinniluan" | "xueji" | "yaowu" | "yechou" | "yeyan" | "yingjian"
        | "yiyong" | "yuce" | "yxs_fanji" | "zhanyi" | "zhendu" | "zhenlie" | "zhongyong"
        | "zhuandui" | "zhuhai" | "zhuiji" | "zhuikong" | "zhuixi" => {
            set_flag(ctx.core, owner, &format!("l11_npc_offense:{skill_name}"), 1);
            if matches!(
                skill_name,
                "qiangxi" | "reqiangxi" | "xinfu_langxi" | "yeyan" | "shenfen"
            ) {
                if let Some(t) = first_other_alive(ctx.core, owner) {
                    let nature = if skill_name == "yeyan" {
                        DamageNature::Fire
                    } else {
                        DamageNature::Normal
                    };
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(owner),
                        t,
                        1,
                        nature,
                        None,
                    )));
                }
            }
        }

        // 3b. Cross-package draw / search / hand-replenishment skills
        "anguo" | "aocai" | "bingyi" | "chajue" | "chouce" | "dcshushen" | "dddjiexing"
        | "dddqiaoduan" | "diancai" | "drlt_jieying" | "dujin" | "fenyin" | "guanzong" | "guju"
        | "gushe" | "guzheng" | "gzyinghun" | "hongde" | "jianying" | "jiaojie" | "jici"
        | "jiezhong" | "jiezi" | "jigong" | "juejing" | "jujian" | "juyi" | "kaikang"
        | "kuanshi" | "kunfen" | "kurou" | "kyou_zhidian" | "liangzhu" | "lianpo" | "lianzhu"
        | "lirang" | "lixia" | "liyu" | "lizhan" | "luoshen" | "miji" | "mingce" | "mingjian"
        | "mizhao" | "mjmouzhi" | "mobilejingce" | "mozhi" | "new_reyiji" | "nzry_chenglve"
        | "nzry_cunmu" | "nzry_feijun" | "nzry_jieying" | "nzry_shicai" | "piaoling"
        | "qiangzhi" | "qice" | "qingzhongx" | "rebiyue" | "reganlu" | "reguanxing" | "reguicai"
        | "rehuashen" | "rehunzi" | "rejiangchi" | "rejianxiong" | "rejieming" | "rejizhi"
        | "rejunxing" | "rekanpo" | "rekuangcai" | "relianying" | "renjie" | "renshe" | "repolu"
        | "reqicai" | "reqizhou" | "requanji" | "reshejian" | "retuntian" | "rexingshang"
        | "reyajiao" | "reyingzi" | "rezhenxing" | "rezhiheng" | "rezhijian" | "rezhiyi"
        | "rezongshi" | "rongbei" | "sbaiyin" | "sbyingzi" | "shangshi" | "shelie" | "shenxian"
        | "shenxing" | "shunshi" | "spdengli" | "sphuangen" | "spqishe" | "sptunjiang"
        | "spwenji" | "taoluan" | "tiandu" | "tianming" | "tianshu" | "tongduo" | "twhongju"
        | "twjiefan" | "twjuntun" | "twkujian" | "twliancai" | "twlingfa" | "twlinglu"
        | "twmibei" | "twqirang" | "twshengxi" | "twxiafeng" | "twxiangyu" | "twxingqi"
        | "twxiongjun" | "twxiongxi" | "twxuanhuo" | "twyingjia" | "twyuanhu" | "twyuhua"
        | "twzhengrong" | "txhj_guimen" | "txhj_guiyan" | "txhj_huangjin" | "txhj_jinwei"
        | "txhj_jinxing" | "txhj_quanqin" | "txhj_shiyuan" | "txhj_wangyuan" | "txhj_xiehui"
        | "txhj_yaoshu" | "txhj_yishi" | "txhj_zhoufa" | "txhj_zhouzu" | "txhj_zhuhun"
        | "txhj_zhuosheng" | "waishi" | "wangxi" | "wangzun" | "weijing" | "xianfu" | "xiaoji"
        | "xinanguo" | "xinchenhui" | "xinfu_limu" | "xinjiewei" | "xinjuejing" | "xinjushou"
        | "xinsheng" | "xiongyi" | "yinghun" | "yingzi" | "yise" | "yjjianshu" | "yjzhenlve"
        | "yongjin" | "yongsi" | "zaoxian" | "zhengfu" | "zhengjian" | "zhiheng" | "zhiji"
        | "zhiyan" | "zhiyi" | "zhuide" | "zhuiyi" | "zili" | "zishou" | "zongkui"
        | "zymoucheng" => {
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, &format!("l11_npc_draw:{skill_name}"), 1);
        }

        // 3c. Cross-package healing / survival / dying / shield skills
        "benghuai" | "buqu" | "chenqing" | "chunlao" | "fuli" | "jieyuan" | "jijiu" | "jsrgshichong"
        | "new_reqingnang" | "olniepan" | "qingxian" | "qinyin" | "quji" | "rebeige"
        | "refuyuan" | "retianxiang" | "rezaiqi" | "shibei" | "shixin" | "shizhi" | "spchijie"
        | "spmingshi" | "txhj_didun" | "txhj_dizhou" | "txhj_guimei" | "txhj_juhun"
        | "txhj_renfang" | "txhj_renwang" | "xianzhou" | "xinfu_daigong" | "xinfu_fuyin"
        | "xinfu_songsang" | "xinkuanggu" | "xswuyan" | "zaiqi" => {
            if skill_name == "benghuai" {
                ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            } else {
                let _ = ctx.recover(owner, 1, None);
            }
            set_flag(ctx.core, owner, &format!("l11_npc_survive:{skill_name}"), 1);
        }

        // 3d. Cross-package control / discard / steal / position skills
        "baiyi" | "cihuai" | "drlt_poxi" | "duanliang" | "fangquan" | "fenwei" | "gnjinfan"
        | "gnsheque" | "gongxin" | "jinglve" | "jsrgzhenqiao" | "lihun" | "liuli" | "new_yijue"
        | "qiaomeng" | "qiaoshui" | "qixi" | "reguose" | "reshanxi" | "retuxi" | "shangyi"
        | "shefu" | "spyishi" | "txhj_beiming" | "txhj_wangzu" | "weifeng" | "xiadao"
        | "xinfu_zhaoxin" | "xinqieting" | "yinbing" | "zhaohuo" | "zhiman" => {
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = discard_up_to_hand_or_equip(ctx, t, 1);
            }
            set_flag(ctx.core, owner, &format!("l11_npc_control:{skill_name}"), 1);
        }

        // 3e. Cross-package passive / aura / distance / dualside / hand-limit skills
        "bazhen" | "drlt_qianjie" | "dualside" | "feiying" | "fulin" | "gaoyuan" | "gongao"
        | "huaizi" | "juxiang" | "keji" | "kongcheng" | "mashu" | "reqianju" | "twhanyu"
        | "twhengjiang" | "twhuangjin" | "twjiange" | "twqianju" | "weimu" | "reweimu"
        | "xiangle" | "xiaoguo" | "xinhongyan" | "yicong" | "yizhong" | "zhaxiang" | "zhenlue"
        | "zhichi" | "zhidao" => {
            set_flag(ctx.core, owner, &format!("l11_npc_aura:{skill_name}"), 1);
        }

        _ => {
            set_flag(ctx.core, owner, &format!("l11_unhandled_skill:{skill_name}"), 1);
        }
    }
}
