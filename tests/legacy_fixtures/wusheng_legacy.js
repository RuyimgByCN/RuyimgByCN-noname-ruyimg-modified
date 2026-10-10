// Legacy noname skill.js fixture — 武圣 (wusheng)
var skills = {
    wusheng: {
        viewAs: {
            name: 'sha',
        },
        enable: "phaseUse",
        filterCard: function (card) {
            return get.color(card) == 'red';
        },
        check: function (card) {
            return true;
        },
        content: function () {
            'step 0'
            player.useCard({ name: 'sha' }, event.targets);
        },
        ai: {
            order: 5,
        },
    },
};
