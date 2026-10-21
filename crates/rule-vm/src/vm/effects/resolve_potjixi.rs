use super::*;

pub(in super::super) fn decode_request(
    host: &mut dyn VmHost,
    frame: &mut Frame,
    tag: u16,
) -> Result<EffectRequest, VmError> {
    Ok(match tag {
        effect_tag::RESOLVE_POTJIXI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotjixi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                action,
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTCEDUAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotceduan {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTTAISHICI => {
            let zhanlie_def = pop(frame)?.0 as u32;
            let hanzhan_def = pop(frame)?.0 as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePottaishici {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                action,
                cards: host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                hanzhan_def: engine_api::SkillDefId::new(hanzhan_def),
                zhanlie_def: engine_api::SkillDefId::new(zhanlie_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_RELEIJI => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveReleiji {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_HUANGTIAN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveHuangtian {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBZHONGHUI => {
            let mbkechang_def = pop(frame)?.0 as u32;
            let mbsizi_def = pop(frame)?.0 as u32;
            let amount = pop(frame)?.0.max(0) as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbzhonghui {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                action,
                amount,
                mbsizi_def: engine_api::SkillDefId::new(mbsizi_def),
                mbkechang_def: engine_api::SkillDefId::new(mbkechang_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTJIEJIE => {
            let qingshi_def = pop(frame)?.0 as u32;
            let diamond_def = pop(frame)?.0 as u32;
            let club_def = pop(frame)?.0 as u32;
            let heart_def = pop(frame)?.0 as u32;
            let spade_def = pop(frame)?.0 as u32;
            let used_def = pop(frame)?.0 as u32;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotjiejie {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                used_def: engine_api::SkillDefId::new(used_def),
                suit_defs: [spade_def, heart_def, club_def, diamond_def]
                    .map(engine_api::SkillDefId::new),
                qingshi_def: engine_api::SkillDefId::new(qingshi_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTQINGSHI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotqingshi {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTWEIYAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotweiyan {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTZHOUYU => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotzhouyu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                card_flags: 0,
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTLVYI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotlvyi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                event_cards: host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                targets: host
                    .event_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTCAOSHUANG => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotcaoshuang {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTYUJI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotyuji {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                cards: host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                targets: host
                    .event_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_REHENGJIANG => {
            let draw_def = pop(frame)?.0 as u32;
            let hand_limit_def = pop(frame)?.0 as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveRehengjiang {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                action,
                hand_limit_def: engine_api::SkillDefId::new(hand_limit_def),
                draw_def: engine_api::SkillDefId::new(draw_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_HAOSHI => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveHaoshi {
                player: PlayerId::new(player),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DIMENG => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDimeng {
                player: PlayerId::new(player),
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_QIAOBIAN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let phase = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveQiaobian {
                player: PlayerId::new(player),
                phase,
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::EXCLUDE_SAVED_SHA_TARGET => {
            let target = pop(frame)?.0 as u32;
            let card = pop(frame)?.0 as u32;
            EffectRequest::ExcludeSavedShaTarget {
                card: engine_api::CardId::new(card),
                target: PlayerId::new(target),
            }
        }
        effect_tag::RESOLVE_XIANGLE => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0 as u32;
            let source = pop(frame)?.0 as u32;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveXiangle {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_FANGQUAN_SKIP => {
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveFangquanSkip {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_FANGQUAN_GRANT => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveFangquanGrant {
                player: PlayerId::new(player),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_RUOYU => {
            let jijiang_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveRuoyu {
                player: PlayerId::new(player),
                jijiang_def: engine_api::SkillDefId::new(jijiang_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZUILUN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveZuilun {
                player: PlayerId::new(player),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_FUYIN => {
            let card = pop(frame)?.0 as u32;
            let source = pop(frame)?.0 as u32;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveFuyin {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::BAN_USE_AGAINST => {
            let target = pop(frame)?.0 as u32;
            let user = pop(frame)?.0 as u32;
            EffectRequest::BanUseAgainst {
                user: PlayerId::new(user),
                target: PlayerId::new(target),
            }
        }
        effect_tag::RESOLVE_JUZHAN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0 as u32;
            let source = pop(frame)?.0 as u32;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveJuzhan {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                targets: host
                    .event_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_QIZHI => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveQizhi {
                player: PlayerId::new(player),
                targets: host
                    .event_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_JINQU => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let qizhi_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveJinqu {
                player: PlayerId::new(player),
                qizhi_def: engine_api::SkillDefId::new(qizhi_def),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_JIANXIANG => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveJianxiang {
                player: PlayerId::new(player),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SHENSHI => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let source = pop(frame)?.0 as u32;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveShenshi {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MINGREN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveMingren {
                player: PlayerId::new(player),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHENLIANG => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let mingren_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0 as u32;
            let source = pop(frame)?.0 as u32;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveZhenliang {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                mingren_def: engine_api::SkillDefId::new(mingren_def),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_KUIZHU => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveKuizhu {
                player: PlayerId::new(player),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHIZHENG => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveZhizheng {
                player: PlayerId::new(player),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_LIJUN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0 as u32;
            let source = pop(frame)?.0 as u32;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveLijun {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_LIANGYIN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let kongsheng_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveLiangyin {
                player: PlayerId::new(player),
                kongsheng_def: engine_api::SkillDefId::new(kongsheng_def),
                stage,
                target: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_KONGSHENG => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0 as u32;
            EffectRequest::ResolveKongsheng {
                player: PlayerId::new(player),
                stage,
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHENGRONG => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhengrong {
                player: PlayerId::new(player),
                candidates: host
                    .event_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                target: PlayerId::new(target),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_HONGJU => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let rong_def = pop(frame)?.0 as u32;
            let qingjian_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveHongju {
                player: PlayerId::new(player),
                qingjian_def: engine_api::SkillDefId::new(qingjian_def),
                rong_def: engine_api::SkillDefId::new(rong_def),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_QINGJIAN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let rong_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveQingjian {
                player: PlayerId::new(player),
                rong_def: engine_api::SkillDefId::new(rong_def),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TUNTIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTuntian {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZAOXIAN => {
            let tuntian_def = pop(frame)?.0 as u32;
            let jixi_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZaoxian {
                player: PlayerId::new(player),
                jixi_def: engine_api::SkillDefId::new(jixi_def),
                tuntian_def: engine_api::SkillDefId::new(tuntian_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SHUANGXIONG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveShuangxiong {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHIJIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            let card = host.selected_cards().first().copied().unwrap_or(0);
            EffectRequest::ResolveZhijian {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                stage: 0,
                target: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_GUZHENG => {
            let event_player = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveGuzheng {
                player: PlayerId::new(player),
                event_player: PlayerId::new(event_player),
                stage: 0,
                cards: Vec::new(),
                return_card: engine_api::CardId::NONE,
                take_rest: false,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHIBA => {
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhiba {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHIBA_SETTLE => {
            let target_card = pop(frame)?.0.max(0) as u32;
            let player_card = pop(frame)?.0.max(0) as u32;
            let winner = pop(frame)?.0.max(0) as u32;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhibaSettle {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                winner: PlayerId::new(winner),
                player_card: engine_api::CardId::new(player_card),
                target_card: engine_api::CardId::new(target_card),
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DRLT_CONGJIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDrltCongjian {
                player: PlayerId::new(player),
                targets: Vec::new(),
                card: engine_api::CardId::NONE,
                target: PlayerId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DRLT_YONGSI => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDrltYongsi {
                player: PlayerId::new(player),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DRLT_WEIDI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDrltWeidi {
                player: PlayerId::new(player),
                cards: Vec::new(),
                targets: Vec::new(),
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_LIEREN => {
            let lose_skill_def = pop(frame)?.0 as u32;
            let win_skill_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveLieren {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                win_skill_def: engine_api::SkillDefId::new(win_skill_def),
                lose_skill_def: engine_api::SkillDefId::new(lose_skill_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_NZRY_FEIJUN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveNzryFeijun {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                branch: 0,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::SET_SKILL_CONTENT_FLAG => {
            let value = pop(frame)?.0 != 0;
            let bit = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::SetSkillContentFlag {
                player: PlayerId::new(player),
                bit,
                value,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWMOUKUI => {
            let card = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwmoukui {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWFUMAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            let context = host.context();
            let cards = match action {
                0 => host.selected_cards().to_vec(),
                1 => (context.event_card != 0)
                    .then_some(context.event_card)
                    .into_iter()
                    .collect(),
                _ => host.event_cards().to_vec(),
            };
            EffectRequest::ResolveTwfuman {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                cards: cards.iter().copied().map(engine_api::CardId::new).collect(),
                action,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_CUIJIN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0 as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveCuijin {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWQINGKOU => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0 as u32;
            let winner = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwqingkou {
                player: PlayerId::new(player),
                winner: PlayerId::new(winner),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWCHENXUN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwchenxun {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWFENXIAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwfenxian {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TIANXIANG => {
            let draw_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTianxiang {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                discard_card: engine_api::CardId::NONE,
                source: PlayerId::NONE,
                amount: 0,
                nature: 0,
                card: engine_api::CardId::NONE,
                card_kind: 0,
                card_flags: 0,
                draw_skill_def: engine_api::SkillDefId::new(draw_skill_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::SUPPRESS_NON_LOCKED_SKILLS_UNTIL_TURN_END => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::SuppressNonLockedSkillsUntilTurnEnd {
                player: PlayerId::new(player),
            }
        }
        effect_tag::CHOOSE_RECOVER_TARGET => {
            let amount = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let chooser = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseRecoverTarget {
                chooser: PlayerId::new(chooser),
                target: PlayerId::new(target),
                amount,
            }
        }
        effect_tag::GIVE_ALL_HAND_THEN_CHOOSE_USE_ONE_OR_DAMAGE => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                cards: Vec::new(),
                targets: Vec::new(),
                stage: 0,
                card: rule_api::CardId::NONE,
                skill: rule_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_CATEGORY_THEN_TARGET_GIVE_MATCHING_OR_SHA_UNRESPONDABLE => {
            let target = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
                player: PlayerId::new(player),
                card: rule_api::CardId::new(card),
                target: PlayerId::new(target),
                skill: rule_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_LOSE_HP_USE_VIRTUAL_SHA_BY_BASIC_VARIANTS => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
                player: PlayerId::new(player),
                card: rule_api::CardId::NONE,
                targets: Vec::new(),
                stage: 0,
                damage_bonus: 0,
                skill: rule_api::SkillId::NONE,
            }
        }
        effect_tag::ADD_BOUND_SKILL => {
            let target = pop(frame)?.0.max(0) as u32;
            let skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::AddBoundSkill {
                player: PlayerId::new(player),
                skill_def: rule_api::SkillDefId::new(skill_def),
                bound_target: PlayerId::new(target),
            }
        }
        effect_tag::REPLACE_SKILL_WITH_BOUND_AFTER_RESOLUTION_IF_TARGET_ALIVE => {
            let target = pop(frame)?.0.max(0) as u32;
            let skill_def = pop(frame)?.0 as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ReplaceSkillWithBoundAfterResolutionIfTargetAlive {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
                skill_def: rule_api::SkillDefId::new(skill_def),
                bound_target: PlayerId::new(target),
                ready_to_replace: false,
            }
        }
        effect_tag::CHOOSE_DISCARD_ALL_HAND_OR_EQUIP_DRAW_LOST_HP_THEN_LOSE_HP => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
                zone: 0,
                discarded_count: 0,
                ready_to_draw: false,
            }
        }
        effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_RECOVER => {
            let amount = pop(frame)?.0.max(0) as u32;
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardHandEquipThenRecover {
                player: PlayerId::new(player),
                count,
                amount,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_OLDANXU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOldAnxu {
                player: PlayerId::new(player),
                stage: 0,
                giver: PlayerId::NONE,
                receiver: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::STORE_SELECTED_IN_SKILL_PILE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::StoreSelectedInSkillPile {
                player: PlayerId::new(player),
                cards: host
                    .selected_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::END_MATCH_FOR_CAMP_IF_SKILL_PILE_AT_LEAST => {
            let count = pop(frame)?.0.max(0) as u32;
            let pile_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::EndMatchForCampIfSkillPileAtLeast {
                player: PlayerId::new(player),
                pile_skill_def: engine_api::SkillDefId::new(pile_skill_def),
                count,
            }
        }
        effect_tag::RESOLVE_OLDQINQING => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOldQinqing {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                lord: PlayerId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_OLDMINGJIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOldMingjian {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_OLDXUANFENG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOldXuanfeng {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                stage: 0,
                branch: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_JIANROU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveJianrou {
                player: PlayerId::new(player),
                damage_id: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_OLDHUISHENG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOldHuisheng {
                player: PlayerId::new(player),
                source: PlayerId::NONE,
                damage_id: 0,
                stage: 0,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_OLDJIEFAN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOldJiefan {
                player: PlayerId::new(player),
                dying: PlayerId::NONE,
                current: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                damage_id: 0,
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_CHEGU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveChegu {
                player: PlayerId::new(player),
                card: engine_api::CardId::NONE,
                targets: Vec::new(),
                cards: Vec::new(),
                branch: 0,
                index: 0,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_JUNKGUIXIN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveJunkguixin {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                stage: 0,
                group: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_YAO_CHENWEI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveYaoChenwei {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                mode: 0,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::REFRESH_SKILL => {
            let skill = pop(frame)?.0.max(0) as u32;
            EffectRequest::RefreshSkill {
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::TAG_CARD => {
            let skill = pop(frame)?.0.max(0) as u32;
            let cards: Vec<engine_api::CardId> = host
                .event_cards()
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect();
            EffectRequest::TagCard { cards, tag: skill }
        }
        effect_tag::UNTAG_CARD => {
            let skill = pop(frame)?.0.max(0) as u32;
            let cards: Vec<engine_api::CardId> = host
                .event_cards()
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect();
            EffectRequest::UntagCard { cards, tag: skill }
        }
        effect_tag::CLANSHENJUN_REVEAL => {
            let skill = pop(frame)?.0.max(0) as u32;
            let event_card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ClanshenjunReveal {
                player: engine_api::PlayerId::new(player),
                event_card: engine_api::CardId::new(event_card),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::CLANSHENJUN_USE => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ClanshenjunUse {
                player: engine_api::PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::CLANYUZHI_REVEAL => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ClanyuzhiReveal {
                player: engine_api::PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::CLANYUZHI_ROUND_END => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ClanyuzhiRoundEnd {
                player: engine_api::PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::QIEYI_PHASE_START => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::QieyiPhaseStart {
                player: engine_api::PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::QIEYI_USE_AFTER => {
            let skill = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::QieyiUseAfter {
                player: engine_api::PlayerId::new(player),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::JIANZHI_PHASE_END => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::JianzhiPhaseEnd {
                player: engine_api::PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::STORAGE_SET => {
            let value = pop(frame)?.0;
            let key = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            EffectRequest::StorageSet {
                skill: engine_api::SkillId::new(skill),
                key,
                value: value.max(0) as u32,
            }
        }
        effect_tag::DRAW_ALL_EVENT_TARGETS => {
            let count = pop(frame)?.0.max(0) as u32;
            // Event player + declared event targets, de-duplicated, in declaration order.
            let mut players = vec![PlayerId::new(host.context().event_player)];
            for target in host.event_targets() {
                let player = PlayerId::new(*target);
                if player != PlayerId::NONE && !players.contains(&player) {
                    players.push(player);
                }
            }
            EffectRequest::DrawAllEventTargets { players, count }
        }
        _ => {
            return Err(VmError::Host(format!("unknown effect tag {tag}")));
        }
    })
}
