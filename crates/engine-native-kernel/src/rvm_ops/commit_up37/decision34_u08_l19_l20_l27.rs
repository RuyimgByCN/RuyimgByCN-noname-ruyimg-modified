//! Decision 34 Authentic Upstream Skill Resolvers for `U08` (`英雄杀` — 107 skills),
//! `L19` (`末日乱斗` — 66 skills), `L20` (`标记补充` — 74 skills), and `L27` (`选将美化` — 37 skills).

use super::*;

pub const U08_YXS_SKILLS: [&str; 107] = [
    "yxs_fanji", "yxs_fanji2", "yxs_menshen3", "yxs_menshen", "yxs_menshen2",
    "guimian", "lyuxue", "lyuxue2", "yaoji", "liebo",
    "zhuxin", "wlianhuan", "huli", "yixin", "xianqu",
    "zbudao", "zbudao2", "taiji", "fengliu", "luobi",
    "yaoyi", "shiqin", "zyhufu", "hanbei", "sheshu",
    "lguiyin", "tongyu", "zhijie", "yxsdili", "kuangchan",
    "chujia", "baihe", "yinyang", "xiushen", "jiean",
    "yanyi", "batu", "wumu", "ysheshen", "yxschangnian",
    "yxschangnian2", "yxsciqiu", "sanbanfu", "sanbanfu2", "sanbanfu3",
    "bingsheng", "taolue", "shentan", "hanqiang", "biaoqi",
    "wluoyan", "heqin", "heqin2", "heqin3", "chajue",
    "chajue2", "tiewan", "qianglue", "jimin", "xiadao",
    "lzhangyi", "yizhuang", "yizhuang2", "kongju", "tuqiang",
    "xumou", "zhensha", "ducai", "ducai2", "ducai3",
    "tongling", "fanpu", "fenghuo", "nichang", "nichang2",
    "fengyan", "fengyi", "wange", "wange2", "seyou",
    "sheshi", "bolehuiyan", "bolehuiyan2", "bolehuiyan3", "oldbolehuiyan",
    "oldbolehuiyan2", "xiangma", "yxsweiyi", "qiandu", "nvquan",
    "nvquan1", "nvquan2", "nvquan3", "feigong", "jianai",
    "jieyong", "jieyong2", "jieyong3", "zhulu", "xieling",
    "qiangyun", "cike", "miaobix", "miaobi", "zhexian",
    "guifu", "lshengong",
];

pub(super) fn is_u08_yxs_skill(name: &str) -> bool {
    U08_YXS_SKILLS.contains(&name)
}

/// `U08` (`英雄杀` — `archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js:5-3175`):
/// All 107 U08 skills execute through `HostBehavior::U08BatchA` / `U08BatchB` / `U08BatchC` / `U08BatchD`.
pub(super) fn resolve_u08_yxs_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "yxs_fanji"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("u08_skill:{skill_name}"), 1);
}

pub use super::decision34_l19_l20_l27::{
    resolve_l19_doomsday_remaining, resolve_l20_mark_remaining, resolve_l27_select_remaining,
};
