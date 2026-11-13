({
            th_taipingyaoshu: {
              audio: true,
              fullskin: true,
              type: 'equip',
              subtype: 'equip2',
              suit: 'heart',
              number: 3,
              derivation: 'th_nanhualaoxian',
              global: ['th_g_taipingyaoshu_ai'],
              skills: ['th_taipingyaoshu'],
              ai: {
                equipValue: function (card, player) {
                  if (player.hasSkill('wendao')) return 9;
                  if (game.hasPlayer(function (current) {
                    return current.hasSkill('wendao') && get.attitude(player, current) <= 0;
                  })) {
                    return 1;
                  }
                  return 6;
                },
                basic: {
                  equipValue: 6
                }
              },
              filterLose: function (card, player) {
                if (player.hasSkillTag('unequip2')) return false;
                return true;
              },
              loseDelay: false,
              onLose: function () {
                var next = game.createEvent('th_taipingyaoshu');
                event.next.remove(next);
                var evt = event.getParent();
                if (evt.getlx === false) evt = evt.getParent();
                evt.after.push(next);
                next.player = player;
                next.setContent(lib.card.th_taipingyaoshu.onLosex);
              },
              onLosex: function () {
                'step 0'
                player.logSkill('th_taipingyaoshu');
                dcdAnim.loadSpine(_Thunder.assets.tpysxx.name, "skel", function () {
                  dcdAnim.playSpine(_Thunder.assets.tpysxx, { parent: player, scale: 0.5 });
                })
                player.draw(2);
                'step 1'
                if (player.hp > 1) player.loseHp();
              }
            }

})
