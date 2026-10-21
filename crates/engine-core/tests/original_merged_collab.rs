//! O2 全量合并批次 1（2026-10-14）：家族 collab 的 20 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_collab/original_collab_dcfanzhuan.rs"]
mod original_collab_dcfanzhuan;

#[path = "original_merged_collab/original_collab_dcjianxiong.rs"]
mod original_collab_dcjianxiong;

#[path = "original_merged_collab/original_collab_dclonggong.rs"]
mod original_collab_dclonggong;

#[path = "original_merged_collab/original_collab_dczhiheng.rs"]
mod original_collab_dczhiheng;

#[path = "original_merged_collab/original_collab_fuzhao.rs"]
mod original_collab_fuzhao;

#[path = "original_merged_collab/original_collab_haoxue.rs"]
mod original_collab_haoxue;

#[path = "original_merged_collab/original_collab_mount_skills.rs"]
mod original_collab_mount_skills;

#[path = "original_merged_collab/original_collab_natuxing_gate.rs"]
mod original_collab_natuxing_gate;

#[path = "original_merged_collab/original_collab_olchenlong.rs"]
mod original_collab_olchenlong;

#[path = "original_merged_collab/original_collab_olchouniu.rs"]
mod original_collab_olchouniu;

#[path = "original_merged_collab/original_collab_olhaizhu.rs"]
mod original_collab_olhaizhu;

#[path = "original_merged_collab/original_collab_olmaotu.rs"]
mod original_collab_olmaotu;

#[path = "original_merged_collab/original_collab_olshenhou.rs"]
mod original_collab_olshenhou;

#[path = "original_merged_collab/original_collab_olsishe.rs"]
mod original_collab_olsishe;

#[path = "original_merged_collab/original_collab_olweiyang.rs"]
mod original_collab_olweiyang;

#[path = "original_merged_collab/original_collab_olwuma.rs"]
mod original_collab_olwuma;

#[path = "original_merged_collab/original_collab_olxugou.rs"]
mod original_collab_olxugou;

#[path = "original_merged_collab/original_collab_olyinhu.rs"]
mod original_collab_olyinhu;

#[path = "original_merged_collab/original_collab_olzishu.rs"]
mod original_collab_olzishu;

#[path = "original_merged_collab/original_collab_spfengyin.rs"]
mod original_collab_spfengyin;
