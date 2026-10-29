use super::*;

mod bi_yue;
mod bingfeng;
mod buchun;
mod fan_jian;
mod fan_kui;
mod futai;
mod gang_lie;
mod guan_xing;
mod heisheng;
mod huoxing;
mod ji_ji;
mod ji_jie;
mod ji_yuan;
mod ji_zhi;
mod jian_xiong;
mod jie_yin;
mod jijiang;
mod jushou;
mod ku_rou;
mod kuang_fu;
mod leizhu;
mod li_jian;
mod lian_ying;
mod lingbo;
mod luan_chang;
mod luo_shen;
mod luo_yi;
mod noop;
mod pingdeng;
mod qing_nang;
mod qizuo;
mod renao;
mod rende;
mod san_cai;
mod shed_u;
mod shen_zhi;
mod shou_yi;
mod shu_shen;
mod su_qing;
mod sui_dun;
mod tashui;
mod tong_ji;
mod tu_xi;
mod wang_xi;
mod wang_zun;
mod wanghun;
mod wangxiang;
mod xiang_de;
mod xiao_guo;
mod xiao_ji;
mod xiaoxiong;
mod xingxia;
mod xun_xun;
mod yao_wu;
mod yi_shang;
mod ying_zi;
mod zhangba;
mod zhao_huo;
mod zhi_heng;
mod zhuxin;
mod zj_leiji;

pub(super) use bi_yue::*;
pub(super) use bingfeng::*;
pub(super) use buchun::*;
pub(super) use fan_jian::*;
pub(super) use fan_kui::*;
pub(super) use futai::*;
pub(super) use gang_lie::*;
pub(super) use guan_xing::*;
pub(super) use heisheng::*;
pub(super) use huoxing::*;
pub(super) use ji_ji::*;
pub(super) use ji_jie::*;
pub(super) use ji_yuan::*;
pub(super) use ji_zhi::*;
pub(super) use jian_xiong::*;
pub(super) use jie_yin::*;
pub(super) use jijiang::*;
pub(super) use jushou::*;
pub(super) use ku_rou::*;
pub(super) use kuang_fu::*;
pub(super) use leizhu::LeizhuBehavior;
pub(crate) use leizhu::LeizhuMachine;
pub(super) use li_jian::*;
pub(super) use lian_ying::*;
pub(super) use lingbo::*;
pub(super) use luan_chang::*;
pub(super) use luo_shen::*;
pub(super) use luo_yi::*;
pub(super) use noop::*;
pub(super) use pingdeng::*;
pub(super) use qing_nang::*;
pub(super) use qizuo::*;
pub(super) use renao::RenaoBehavior;
pub(crate) use renao::RenaoMachine;
pub(super) use rende::*;
pub(super) use san_cai::*;
pub(super) use shed_u::*;
pub(super) use shen_zhi::*;
pub(super) use shou_yi::*;
pub(super) use shu_shen::*;
pub(super) use su_qing::*;
pub(super) use sui_dun::*;
pub(super) use tashui::*;
pub(super) use tong_ji::*;
pub(super) use tu_xi::*;
pub(super) use wang_xi::*;
pub(super) use wang_zun::*;
pub(super) use wanghun::*;
pub(super) use wangxiang::*;
pub(super) use xiang_de::*;
pub(super) use xiao_guo::*;
pub(super) use xiao_ji::*;
pub(super) use xiaoxiong::*;
pub(super) use xingxia::*;
pub(super) use xun_xun::*;
pub(super) use yao_wu::*;
pub(super) use yi_shang::*;
pub(super) use ying_zi::*;
pub(super) use zhangba::*;
pub(super) use zhao_huo::*;
pub(super) use zhi_heng::*;
pub(super) use zhuxin::*;
pub(super) use zj_leiji::*;

/// 性别门控助手：指定座位的双将槽中任一武将为男性。
fn seat_is_male(ctx: &Ctx<'_>, p: PlayerId) -> bool {
    slots_is_male(&ctx.core.player(p).general_slots)
}

/// 将槽性别判定（非国战局无将槽 → 门控不适用，默认 true）。
fn slots_is_male(slots: &[Option<u16>; 2]) -> bool {
    if slots.iter().all(|s| s.is_none()) {
        return true;
    }
    slots.iter().flatten().any(|g| {
        crate::content::standard::general_gender(*g) == Some(crate::content::standard::Gender::Male)
    })
}
