//! Phase 3 P3-C1a `L11` 跨包 NPC Part A（80 + 3 技，统一归口 `HostBehavior::L11CrossNpcA`）。

pub(super) mod activate;
mod active_a;
mod active_b;
pub(super) mod asks;
pub(super) mod asks_b;
mod card_use_b;
mod damage_a;
mod dying_lose_resp;
pub(super) mod filter;
pub(super) mod helpers;
pub(super) mod lifecycle;
pub(super) mod meta;
mod phase_turn;

pub(in crate::content::host) use activate::activate_l11_npc_a;
pub(in crate::content::host) use asks::ask_l11_npc_a;
pub(in crate::content::host) use dying_lose_resp::{
    aocai_respond_skill_options, aocai_try_activate_respond_skill,
};
pub(in crate::content::host) use filter::{filter_l11_npc_a_with_frames, machine_l11_npc_a};
pub(in crate::content::host) use helpers::on_death_before_l11_npc_a;
pub(in crate::content::host) use lifecycle::{
    on_card_use_after_l11_npc_a, on_cards_to_discard_l11_npc_a, on_damage_end_l11_npc_a,
    on_equip_changed_l11_npc_a, on_game_start_l11_npc_a, on_hand_card_removed_l11_npc_a,
    on_phase_end_l11_npc_a, on_phase_use_end_l11_npc_a, on_sha_end_step_l11_npc_a,
    on_turn_begin_l11_npc_a, on_turn_phase_after_l11_npc_a,
};
pub(in crate::content::host) use meta::{
    has_phase_use_l11_npc_a, limits_l11_npc_a, trigger_meta_l11_npc_a,
    window_trigger_meta_l11_npc_a,
};

use super::*;
use crate::machine::{Ctx, Flow};

pub(in crate::content::host) fn handle_l11_npc_a(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    m.card = None;
    ctx.core
        .vm_aux
        .flags
        .remove(&(m.owner, "l11_npc_a_ask".to_string()));
    match m.def_key.as_str() {
        "baiyi" | "duwu" | "fenxun" | "guanzong" | "gushe" | "jinglve" | "jujian"
        | "lianzhu" | "lihun" | "mingjian" | "mizhao" | "mouzhu" => {
            active_a::handle_active_a(m, ctx)
        }
        "new_reqingnang" | "new_yijue" | "nzry_chenglve" | "nzry_feijun" | "quji"
        | "refenxun" | "reganlu" | "reguose" | "rejunxing" | "reluanji" | "rewurong"
        | "rezhijian" | "rongbei" | "kyou_zhidian" | "dclbjiuxian" | "olhuoji"
        | "reshuangxiong" => active_b::handle_active_b(m, ctx),
        "benyu" | "chouce" | "dcmoukui" | "dcnutao" | "guju" | "jieyuan" | "leiji"
        | "liyu" | "mjmouzhi" | "nsxianhai" | "qingxian" | "rejinjiu" | "repojun"
        | "retianxiang" => damage_a::handle_damage_a(m, ctx),
        "bmcanshi" | "dcmoukui_dodged" | "gaoyuan" | "jili" | "jsrgshichong"
        | "jsrgzhenqiao" | "nzry_shicai" | "olzhuiji" | "reduodao" | "rexingluan"
        | "reyajiao" => card_use_b::handle_card_use_b(m, ctx),
        "diancai" | "gnsheque" | "jigong" | "lixia" | "mozhi" | "nzry_kuizhu"
        | "piaoling" | "qingzhongx" | "reguanxing" | "rejuece" | "reluoyi" | "retuxi"
        | "rezaiqi" => phase_turn::handle_phase_turn(m, ctx),
        "aocai" | "buqu" | "chenqing" | "dualside" | "fenji" | "liangzhu" | "lirang"
        | "olniepan" | "relianying" | "renshe" | "requanji" | "retuntian" | "rexingshang"
        | "rexuanfeng" | "reqizhou" => dying_lose_resp::handle_dying_lose_resp(m, ctx),
        _ => helpers::clear_and_done(m),
    }
}
