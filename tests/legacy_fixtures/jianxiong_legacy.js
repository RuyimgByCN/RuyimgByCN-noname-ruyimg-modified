// Legacy noname skill.js fixture — 奸雄 (jianxiong)
// Format: skill table with `name: { ... }` pattern that the analyzer extracts.
var skills = {
    jianxiong: {
        trigger: { player: "damageEnd" },
        forced: true,
        filter: function (event, player) {
            return event.card && get.tag(event.card, 'damage');
        },
        content: function () {
            player.draw(1);
        },
        ai: {
            maixie_defend: true,
        },
    },
};
