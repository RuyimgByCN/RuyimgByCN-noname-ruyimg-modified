//! Attach `P3-E3` `L11` (`太虚幻境`) cross-package skills stripped from
//! `taixu-upstream-liuli/package.npack.json` natively via `HostBehavior::L11VmFixC`,
//! along with their non-colliding derived/temporary sub-skills (`gnjinfan_gain`,
//! `new_repaoxiao2`, `qiaoshui2`, `qiaoshui3`, `ranshang2`, `reshanxi2`,
//! `zhuikong_distance`).

use std::collections::BTreeSet;

use engine_native::content::host::{HostBehavior, HostSkillDecl};

/// The `42` `L11` cross-package skills stripped from `taixu-upstream-liuli`'s VM
/// bucket and implemented natively via `HostBehavior::L11VmFixC` in Track `P3-E3`:
/// `24` core placeholder/broken skills + `18` `AUDIT_36` skills with upstream
/// interactive/trigger/filter parity gaps.
pub const L11_TAIXU_VM_FIX_C_SKILLS: [&str; 42] = [
    // Core 24 FIX_27 stripped VM skills
    "anjian",
    "drlt_qianjie",
    "dujin",
    "fuli",
    "gnjinfan",
    "huoji",
    "juece",
    "juejing",
    "longhun",
    "mieji",
    "nsfeixiong",
    "nsyice",
    "nzry_jieying",
    "qiaoshui",
    "qice",
    "ranshang",
    "rekuangcai",
    "relianji",
    "reshanxi",
    "rezhenxing",
    "twjiange",
    "xinniluan",
    "xswuyan",
    "zhuikong",
    // 18 AUDIT_36 skills overridden cleanly for exact upstream noname-liuli parity
    "baonu",
    "chongzhen",
    "jintao",
    "new_repaoxiao",
    "nzry_cunmu",
    "redangxian",
    "rejizhi",
    "reqiangxi",
    "retieji",
    "rezhiheng",
    "shelie",
    "shibei",
    "twkujian",
    "xiandeng",
    "xinanjian",
    "xinkuanggu",
    "xiongyi",
    "yizhong",
];

/// Derived/temporary sub-skills owned by `L11VmFixC` (note `zishou2` stays on its existing
/// `taixu-upstream-liuli` VM definition shared with `txhj_liubiao`, and `g_nzry_jieying`
/// stays on `L25ZuanBatchA` while `L11VmFixC` evaluates `nzry_jieying` global hand limit
/// directly in `d34_l11_mods_vm_c`).
pub const L11_TAIXU_VM_FIX_C_SUBSKILLS: [&str; 7] = [
    "gnjinfan_gain",
    "new_repaoxiao2",
    "qiaoshui2",
    "qiaoshui3",
    "ranshang2",
    "reshanxi2",
    "zhuikong_distance",
];

/// Whether `name` must be stripped from `taixu-upstream-liuli`'s precompiled VM bucket.
pub fn is_stripped_vm_skill(name: &str) -> bool {
    L11_TAIXU_VM_FIX_C_SKILLS.contains(&name) || L11_TAIXU_VM_FIX_C_SUBSKILLS.contains(&name)
}

/// Register all `P3-E3` stripped skills and their sub-skills under `HostBehavior::L11VmFixC`.
pub fn attach_taixu_vm_fix_c(
    extra_host_skills: &mut Vec<HostSkillDecl>,
    seen_skills: &mut BTreeSet<String>,
) {
    for &skill in L11_TAIXU_VM_FIX_C_SKILLS
        .iter()
        .chain(L11_TAIXU_VM_FIX_C_SUBSKILLS.iter())
    {
        crate::push_host_skill_if_unseen(
            skill,
            HostBehavior::L11VmFixC,
            extra_host_skills,
            seen_skills,
        );
    }
}
