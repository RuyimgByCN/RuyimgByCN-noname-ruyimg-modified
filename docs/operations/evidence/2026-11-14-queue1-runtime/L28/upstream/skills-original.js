({
            th_chengye: {
              //audio: "ext:雷霆万钧/audio/skill:2",
              //马上就到你家门口
              audio: 'chengye',
              liujing_filter: [
                function (card) {
                  return get.type(card, false) == 'trick' && get.tag(card, 'damage', null, false) > 0;
                },
                (card) => get.type(card, false) == 'basic',
                (card) => get.name(card, false) == 'wuxie',
                (card) => get.name(card, false) == 'wuzhong',
                (card) => get.name(card, false) == 'lebu',
                (card) => get.type(card, false) == 'equip',
              ],
              getLiujing: function (player, index) {
                var filter = lib.skill.th_chengye.liujing_filter[index], expansion = player.getExpansions('th_chengye');
                for (var i of expansion) {
                  if (filter(i)) return i;
                }
                return false;
              },
              trigger: { global: ['useCardAfter', 'loseAfter', 'cardsDiscardAfter'] },
              forced: true,
              filter: function (event, player) {
                if (player == event.player) return false;
                if (event.name == 'useCard') {
                  if (!event.card.isCard) return false;
                  var cards = event.cards.filterInD();
                  if (!cards.length) return false;
                }
                else if (event.name == 'lose') {
                  if (event.position != ui.discardPile) return false;
                  var cards = event.cards2.filter(function (card) {
                    if (get.position(card, true) != 'd') return false;
                    var type = get.type(card, false);
                    return type == 'delay' || type == 'equip';
                  });
                  if (!cards.length) return false;
                }
                else {
                  var evtx = event.getParent();
                  if (evtx.name != 'orderingDiscard') return false;
                  var evt2 = (evtx.relatedEvent || evtx.getParent());
                  if (evt2.name != 'phaseJudge' || evt2.player == player) return;
                  var cards = event.cards.filter(function (card) {
                    if (get.position(card, true) != 'd') return false;
                    var type = get.type(card, false);
                    return type == 'delay';
                  });
                  if (!cards.length) return false;
                }
                for (var i = 0; i < 6; i++) {
                  if (lib.skill.th_chengye.getLiujing(player, i)) continue;
                  for (var j of cards) {
                    if (lib.skill.th_chengye.liujing_filter[i](j)) return true;
                  }
                }
                return false;
              },
              content: function () {
                var cards, cards2 = [];
                if (trigger.name == 'useCard') {
                  cards = trigger.cards.filterInD();
                }
                else if (trigger.name == 'lose') {
                  cards = trigger.cards2.filter(function (card) {
                    if (get.position(card, true) != 'd') return false;
                    var type = get.type(card, false);
                    return type == 'delay' || type == 'equip';
                  });
                }
                else {
                  cards = trigger.cards.filter(function (card) {
                    if (get.position(card, true) != 'd') return false;
                    var type = get.type(card, false);
                    return type == 'delay';
                  });
                }
                for (var i = 0; i < 6; i++) {
                  if (lib.skill.th_chengye.getLiujing(player, i)) continue;
                  for (var j of cards) {
                    if (lib.skill.th_chengye.liujing_filter[i](j)) {
                      cards.remove(j);
                      cards2.push(j);
                      break;
                    }
                  }
                  if (!cards.length) break;
                }
                player.addToExpansion(cards2, 'gain2').gaintag.add('th_chengye');
              },
              onremove: function (player, skill) {
                var cards = player.getExpansions(skill);
                if (cards.length) player.loseToDiscardpile(cards);
              },
              marktext: '典',
              intro: {
                name: '承业',
                markcount: 'expansion',
                content: 'expansion',
                mark: function (dialog, storage, player) {
                  var liujing = ['诗', '书', '礼', '易', '乐', '春秋'];
                  var liujingdian = ['伤害锦囊牌', '基本牌', '无懈可击', '无中生有', '乐不思蜀', '装备牌'];
                  var bg = ui.create.div('.th-dibeijing', document.body);
                  bg.style.cssText += 'font-family:"th-zhongli";font-size:15px';
                  bg.addEventListener(game.isMobile(lib.config.touchscreen) ? 'touchstart' : 'mousedown', function () {
                    this.remove();
                  });
                  var buxu = game.thunderDialog(player, '承业', bg, 300, 860);
                  var dianbg = ui.create.div('.th-mamididian', buxu.container);
                  for (var i = 0; i < 6; i++) {
                    var paizi = ui.create.div('.th-mamidipaizi', dianbg);
                    paizi.style.left = (4.5 + (i % 3) * 30.8) + '%';
                    paizi.style.top = (i > 2 ? 46 : 3) + '%';
                    paizi.innerHTML = liujing[i];
                    var card = lib.skill.th_chengye.getLiujing(player, i);
                    if (!card) {
                      var tips = ui.create.div('.th-mamiditips', dianbg);
                      tips.style.left = (12.5 + (i % 3) * 30.8) + '%';
                      tips.style.top = (i > 2 ? 61 : 16) + '%';
                      var str = "缺少<br>";
                      str += liujingdian[i];
                      tips.innerHTML = str;
                    } else {
                      var info = [card.suit || undefined, card.number || undefined, card.name || undefined, card.nature || undefined];
                      var smCard = ui.create.card(dianbg, 'noclick', true).init(info);
                      smCard.classList.add('th-zjcard');
                      if (game.thunderIsPhone()) smCard.classList.add('dui-mobile');
                      smCard.style.left = (11 + (i % 3) * 30.8) + '%';
                      smCard.style.top = (i > 2 ? 37 : -6.5) + '%';
                      smCard.style.transform = 'scale(0.6)';
                    }
                  }
                },
              },
              group: ['th_chengye_gain', 'th_chengye_look'],
              subSkill: {
                gain: {
                  trigger: { player: 'phaseUseBegin' },
                  forced: true,
                  filter: function (event, player) {
                    return player.getExpansions('th_chengye').length >= 6;
                  },
                  content: function () {
                    player.gain(player.getExpansions('th_chengye'), 'gain2');
                  },
                },
                look: {
                  enable: 'phaseUse',
                  direct: true,
                  delay: false,
                  content: function () {
                    var liujing = ['诗', '书', '礼', '易', '乐', '春秋'];
                    var liujingdian = ['伤害锦囊牌', '基本牌', '无懈可击', '无中生有', '乐不思蜀', '装备牌'];
                    event.bg = ui.create.div('.th-dibeijing', document.body);
                    event.bg.style.cssText += 'font-family:"th-zhongli";font-size:15px';
                    event.bg.style.zIndex = 8;
                    event.bg.addEventListener(game.isMobile(lib.config.touchscreen) ? 'touchstart' : 'mousedown', function () {
                      this.remove();
                      event.finish();
                    });
                    var buxu = game.thunderDialog(player, '承业', event.bg, 300, 860);
                    var dianbg = ui.create.div('.th-mamididian', buxu.container);
                    for (var i = 0; i < 6; i++) {
                      var paizi = ui.create.div('.th-mamidipaizi', dianbg);
                      paizi.style.left = (4.5 + (i % 3) * 30.8) + '%';
                      paizi.style.top = (i > 2 ? 46 : 3) + '%';
                      paizi.innerHTML = liujing[i];
                      var card = lib.skill.th_chengye.getLiujing(player, i);
                      if (!card) {
                        var tips = ui.create.div('.th-mamiditips', dianbg);
                        tips.style.left = (12.5 + (i % 3) * 30.8) + '%';
                        tips.style.top = (i > 2 ? 61 : 16) + '%';
                        var str = "缺少<br>";
                        str += liujingdian[i];
                        tips.innerHTML = str;
                      } else {
                        var info = [card.suit || undefined, card.number || undefined, card.name || undefined, card.nature || undefined];
                        var smCard = ui.create.card(dianbg, 'noclick', true).init(info);
                        smCard.classList.add('th-zjcard');
                        if (game.thunderIsPhone()) smCard.classList.add('dui-mobile');
                        smCard.style.left = (11 + (i % 3) * 30.8) + '%';
                        smCard.style.top = (i > 2 ? 37 : -6.5) + '%';
                        smCard.style.transform = 'scale(0.6)';
                      }
                    }
                  },
                }
              },
            },
            th_buxu: {
              //audio: "ext:雷霆万钧/audio/skill:2",
              //马上就到你家门口
              audio: 'buxu',
              enable: 'phaseUse',
              filter: function (event, player) {
                var num = (player.getStat('skill').th_buxu || 0) + 1;
                return player.countCards('he') >= num && player.getExpansions('th_chengye').length < 6;
              },
              selectCard: function () {
                var player = _status.event.player;
                return (player.getStat('skill').th_buxu || 0) + 1;
              },
              filterCard: true,
              position: 'he',
              prompt: function () {
                var player = _status.event.player;
                var str = '你可发动补续，弃置';
                str += (player.getStat('skill').th_buxu || 0) + 1;
                str += '张牌，将1张典置于武将牌上';
                //str = str.replace('补续', "<span style='color:#67ac9a'>补续</span>")
                return str;
              },
              check: function (card) {
                var player = _status.event.player;
                if (player.needsToDiscard() > ui.selected.cards.length) return 10 / Math.max(0.1, get.value(card));
                return 5 - (player.getStat('skill').th_buxu || 0) - get.value(card);
              },
              content: function () {
                'step 0'
                if (event.isMine()) {
                  game.pause(player);
                  var liujing = ['诗', '书', '礼', '易', '乐', '春秋'];
                  var liujingdian = ['伤害锦囊牌', '基本牌', '无懈可击', '无中生有', '乐不思蜀', '装备牌'];
                  event.bg = ui.create.div('.th-dibeijing', document.body);
                  event.bg.style.cssText += 'font-family:"th-zhongli";font-size:15px';
                  var buxu = game.thunderDialog(player, '补续', event.bg, 330, 860);
                  var dianbg = ui.create.div('.th-mamididian', buxu.container);
                  dianbg.style.height = '79%';
                  event.shoushaJDT = document.getElementById('jindutiao');
                  if (event.shoushaJDT) {
                    event.shoushaJDT.style.cssText += 'transition:none';
                    event.shoushaJDT.hide();
                  }
                  for (var i = 0; i < 6; i++) {
                    var paizi = ui.create.div('.th-mamidipaizi', dianbg);
                    paizi.style.left = (4.5 + (i % 3) * 30.8) + '%';
                    paizi.style.top = (i > 2 ? 46 : 3) + '%';
                    paizi.innerHTML = liujing[i];
                    var card = lib.skill.th_chengye.getLiujing(player, i);
                    if (!card) {
                      var tips = ui.create.div('.th-mamiditips', dianbg);
                      tips.style.left = (12.5 + (i % 3) * 30.8) + '%';
                      tips.style.top = (i > 2 ? 54 : 9) + '%';
                      var str = "选择<br>";
                      str += liujingdian[i];
                      tips.innerHTML = str;
                      var buBtn = ui.create.div('.th-mamidibu', dianbg);
                      buBtn.style.left = (18.3 + (i % 3) * 30.8) + '%';
                      buBtn.style.top = (i > 2 ? 68 : 24) + '%';
                      //buBtn.innerHTML = '补';
                      buBtn.id = i;
                      buBtn.listen(function () {
                        var filter = lib.skill.th_chengye.liujing_filter[this.id];
                        var card = get.cardPile(filter);
                        if (card) {
                          player.addToExpansion(card, 'gain2').gaintag.add('th_chengye');
                        } else {
                          game.log('剩余牌堆中没有你需要的牌');
                          player.getStat('skill').th_buxu--;
                        }
                        game.thunderClearInterval(time[1]);
                        game.resume(player);
                      });
                    } else {
                      var info = [card.suit || undefined, card.number || undefined, card.name || undefined, card.nature || undefined];
                      var smCard = ui.create.card(dianbg, 'noclick', true).init(info);
                      smCard.classList.add('th-zjcard');
                      if (game.thunderIsPhone()) smCard.classList.add('dui-mobile');
                      smCard.style.left = (11 + (i % 3) * 30.8) + '%';
                      smCard.style.top = (i > 2 ? 37 : -6.5) + '%';
                      smCard.style.transform = 'scale(0.6)';
                    }
                  }
                  var tishi = ui.create.div('.th-mamiditishi', buxu.container);
                  tishi.style.fontSize = Math.round(document.body.offsetWidth * 0.014) + 'px';
                  tishi.innerHTML = '你发动了<span style="color:#67ac9a">补续</span>，选择1种六经并将1张对应的典置于武将牌上';
                  var time = game.thunderCreateTimer(480, function () {
                    game.thunderClearInterval(time[1]);
                    time[0].remove();
                    var indexList = [], filter, index, card;
                    for (var i = 0; i < 6; i++) {
                      if (!lib.skill.th_chengye.getLiujing(player, i)) indexList.push(i);
                    }
                    if (indexList.length) {
                      index = indexList.randomGet();
                    }
                    filter = lib.skill.th_chengye.liujing_filter[index];
                    var card = get.cardPile(filter);
                    if (card) {
                      player.addToExpansion(card, 'gain2').gaintag.add('th_chengye');
                    } else {
                      game.log('剩余牌堆中没有你需要的牌');
                      player.getStat('skill').th_buxu--;
                    }
                    game.resume(player);
                  }, event.bg);
                  time[0].style.top = ((document.body.offsetHeight - 360) * 0.4 + document.body.offsetWidth * 0.052 + 265) + 'px';
                  time[0].style.left = 'calc(50% - var(--w) * 0.5 - 2px)';
                  buxu.onResize.push(function () {
                    if (buxu.min) time[0].style.top = (3 + document.body.offsetHeight * 0.6 + document.body.offsetWidth * 0.036) + 'px';
                    else time[0].style.top = ((document.body.offsetHeight - 360) * 0.4 + document.body.offsetWidth * 0.052 + 265) + 'px';
                  })
                } else {
                  var indexList = [], filter, index, card;
                  for (var i = 0; i < 6; i++) {
                    if (!lib.skill.th_chengye.getLiujing(player, i)) indexList.push(i);
                  }
                  if (indexList.length) {
                    index = indexList.randomGet();
                  }
                  filter = lib.skill.th_chengye.liujing_filter[index];
                  var card = get.cardPile(filter);
                  if (card) {
                    player.addToExpansion(card, 'gain2').gaintag.add('th_chengye');
                  } else {
                    game.log('剩余牌堆中没有你需要的牌');
                    player.getStat('skill').th_buxu--;
                  }
                  event.finish();
                }
                'step 1'
                if (event.shoushaJDT) event.shoushaJDT.show();
                event.bg.remove();
              },
              ai: {
                combo: 'th_chengye',
                order: 0.2,
                result: { player: 1 },
              },
            },

            //南华老仙
            th_zhengjing: {
              //audio: "ext:雷霆万钧/audio/skill:2",
              //马上就到你家门口
              audio: 'zhengjing',
              enable: 'phaseUse',
              usable: 1,
              direct: true,
              // init: function () {
              //     game.thunderLoadFont({ url: lib.assetURL + 'extension/雷霆万钧/assets/th-poem.woff2', cssValue: 'th-poem' });
              //     game.thunderLoadFont({ url: lib.assetURL + 'extension/雷霆万钧/assets/th-zhongli.woff2', cssValue: 'th-zhongli' });
              // },
              filter: function (event, player) {
                if (!window.zhengjingxiao) {
                  window.zhengjingxiao = true;
                  dcdAnim.loadSpine(window._Thunder.assets.zj_jihuo.name, "skel");
                  dcdAnim.loadSpine(window._Thunder.assets.zj_mianban.name, "skel");
                }
                return true;
              },
              content: function () {
                'step 0'
                game.thunderForbidTouch();
                game.pause(player);
                _status.startCut = false;
                _status.fruitCutting = false;
                _status.thzjFinished = false;
                _status.thzhengjingfirstload = false;
                _status.threpeatAudio = lib.config.repeat_audio;
                lib.config.repeat_audio = false;
                var cards = [];
                var names = [];
                while (true) {
                  var card = get.cardPile(function (carde) {
                    return carde.name != 'du' && !names.contains(carde.name);
                  });
                  if (card) {
                    cards.push(card);
                    names.push(card.name);
                    if (_status.qhly_skillTest) {
                      if (cards.length >= 5) break;
                    }
                    else if (get.mode() == 'doudizhu') {
                      if (cards.length == 1 && !get.isLuckyStar(player) && Math.random() < 0.33) break;
                      if (cards.length == 2 && !get.isLuckyStar(player) && Math.random() < 0.5) break;
                      if (cards.length >= 3) break;
                    }
                    else {
                      if (cards.length == 3 && !get.isLuckyStar(player) && Math.random() < 0.33) break;
                      if (cards.length == 4 && !get.isLuckyStar(player) && Math.random() < 0.5) break;
                      if (cards.length >= 5) break;
                    }
                  }
                  else break;
                };
                if (!cards.length) { event.finish(); return; };
                event.blackbg = ui.create.div('.th-dibeijing', document.body);
                event.blackbg.style.background = 'rgba(0,0,0,0.7)';
                event.shoushaJDT = document.getElementById('jindutiao');
                if (event.shoushaJDT) {
                  event.shoushaJDT.style.cssText += 'transition:none';
                  event.shoushaJDT.hide();
                }
                var zjBg = dcdAnim.playSpine(window._Thunder.assets.zj_mianban, { y: document.body.offsetHeight * 0.5 + 40, scale: 0.7 });
                _status.thzhengjingDaojishi = _status.thzhengjingDaojishiTotal = 520;
                setTimeout(function () {
                  zjBg.clip = {
                    x: [0, -0.01],
                    y: 0,
                    width: [0, 0.25],
                    height: [0, 1],
                    clipParent: true
                  }
                }, 2200);
                setTimeout(function () {
                  player.thunderFruit(cards);
                }, 2100);
                'step 1'
                event.blackbg.remove();
                game.thunderAllowTouch();
                if (_status.threpeatAudio) lib.config.repeat_audio = _status.threpeatAudio;
                if (_status.qhly_skillTest) {
                  _status.qhly_skillTest = false;
                  event.finish();
                  return;
                } else {
                  _status.firstZhengjing = true;
                  if (result.bool) {
                    event.cards = result.cards;
                    game.cardsGotoOrdering(event.cards);
                  } else event.finish();
                }
                'step 2'
                game.updateRoundNumber();
                if (event.shoushaJDT) event.shoushaJDT.show();
                player.chooseTarget('将整理出的经典置于一名角色的武将牌上', true).set('ai', function (target) {
                  if (target.hasSkill('xinfu_pdgyingshi')) return 0;
                  var player = _status.event.player;
                  var att = get.attitude(player, target);
                  return -att;
                });
                'step 3'
                if (result.bool) {
                  player.logSkill('th_zhengjing', target);
                  var target = result.targets[0];
                  event.target = target;
                  player.line(target, 'thunder');
                }
                'step 4'
                if (event.cards.length == 1) {
                  event._result = { bool: true, moved: [cards, []] };
                  return;
                }
                var next = player.chooseToMove('整经：请分配整理出的经典', true);
                next.set('list', [
                  ['置于' + get.translation(target) + '的武将牌上', event.cards],
                  ['自己获得'],
                ]);
                next.set('filterMove', function (from, to, moved) {
                  if (moved[0].length == 1 && to == 1 && from.link == moved[0][0]) return false;
                  return true;
                });
                next.set('filterOk', function (moved) {
                  return moved[0].length > 0;
                });
                next.set('processAI', function (list) {
                  var cards = list[0][1].slice(0).sort(function (a, b) {
                    return get.value(a) - get.value(b);
                  });
                  return [cards.splice(0, 1), cards];
                })
                'step 5'
                if (result.bool) {
                  var cards = result.moved[0], gains = result.moved[1];
                  target.addSkill('th_zhengjing2');
                  target.addToExpansion(cards, 'gain2').gaintag.add('th_zhengjing2');
                  if (gains.length) player.gain(gains, 'gain2');
                }
              },
              ai: {
                order: 10,
                result: { player: 1 },
                threaten: 3.2,
              }
            },
            th_zhengjing2: {
              trigger: { player: 'phaseZhunbeiBegin' },
              forced: true,
              charlotte: true,
              intro: { content: 'expansion', markcount: 'expansion' },
              onremove: function (player, skill) {
                var cards = player.getExpansions(skill);
                if (cards.length) player.loseToDiscardpile(cards);
              },
              content: function () {
                'step 0'
                player.gain(player.getExpansions('th_zhengjing2'), 'gain2');
                player.skip('phaseJudge');
                player.skip('phaseDraw');
                'step 1'
                player.removeSkill('th_zhengjing2');
              },
            },

            //马日磾
            th_yeyan: {
              forceDie: true,
              enable: 'phaseUse',
              //audio: "ext:雷霆万钧/audio/skill:3",
              //马上就到你家门口
              audio: 'yeyan',
              limited: true,
              skillAnimation: true,
              animationColor: 'fire',
              direct: true,
              choose4Cards: function (event, player) {
                let cards = player.getCards('h');
                let selectCards = [];
                let suits = [];
                let hasSelected = [];
                for (let i of cards) {
                  let suit = get.suit(i, player);
                  if (suit && !suits.contains(suit)) {
                    selectCards.push(i);
                    suits.push(suit)
                  }
                  if (i.classList.contains('selected')) hasSelected.push(i);
                }

                if (hasSelected.length == 4) return hasSelected;
                if (suits.length == 4) return selectCards;
                return false;
              },
              content: function () {
                'step 0'
                if (event.isMine()) {
                  window.qhly_forbidPlayerWindow = true;
                  event.targets = [];
                  game.thunderForbidTouch();
                  function knock(node1, node2) {
                    var obj1 = node1.getBoundingClientRect(), obj2 = node2.getBoundingClientRect();
                    if (obj2.left > obj1.right || obj2.right < obj1.left || obj2.top > obj1.bottom || obj2.bottom < obj1.top + 60) return false;
                    return true;
                  }
                  var yeyanDialogHeight = 320 * (game.thunderIsPhone() ? 0.66 : 0.88) + 40;
                  var yeyanpartWidth = game.thunderIsPhone() ? 450 : 540;
                  event.dibeijing = ui.create.div('.th-dibeijing', document.body);
                  event.dibeijing.style.pointerEvents = 'none';
                  var yeyan = game.thunderDialog(player, '业炎', event.dibeijing, yeyanDialogHeight, yeyanpartWidth, undefined, 1, true);
                  yeyan.container.style.justifyContent = 'space-around';
                  var fires = [], flames = [];
                  for (var i = 0; i < 3; i++) {                                       //绘制对话框
                    fires[i] = ui.create.div('.th-yeyanfire', yeyan.container);
                    fires[i].id = 'th-yeyanfires' + i;
                    fires[i].style.setProperty('--w', document.body.offsetWidth * 0.08 + 'px');
                    flames[i] = ui.create.div('.th-yeyanflame', fires[i]);
                    flames[i].id = 'th-yeyanflames' + i;
                    flames[i].style.setProperty('--w', document.body.offsetWidth * 0.08 + 'px');
                    flames[i].restore = function () {
                      let num = +this.id.substr(-1);
                      let fire = document.querySelector(`#th-yeyanfires${num}`)
                      this.style.setProperty('--str', "'×1 '");
                      this.style.setProperty('--c', 'red');
                      this.style.setProperty('--ani', 'th-yeyanflame 0.2s infinite');
                      this.style.transform = 'none';
                      this.classList.remove('extinguish');
                      fire.style.setProperty('--str', "'拖动分配'");
                      fire.style.setProperty('--c', "#9c603e");
                    }
                    flames[i].addEventListener(game.isMobile(lib.config.touchscreen) ? 'touchstart' : 'mousedown', function (e) {
                      if (this.classList.contains('extinguish')) return;
                      let num = +this.id.substr(-1);
                      event._yeyanBeginMove = num;
                      document.querySelector('#th-yeyanfires' + num).style.setProperty('--str', ' ');
                      event._yeyanX = game.isMobile(lib.config.touchscreen) ? (e.touches[0].clientX / game.documentZoom) : (e.clientX / game.documentZoom);
                      event._yeyanY = game.isMobile(lib.config.touchscreen) ? (e.touches[0].clientY / game.documentZoom) : (e.clientY / game.documentZoom);
                    });
                  }
                  event.yeyanmove = function (e) {
                    if (typeof event._yeyanBeginMove != 'number') return;
                    let flame = document.querySelector(`#th-yeyanflames${event._yeyanBeginMove}`);
                    let x = game.isMobile(lib.config.touchscreen) ? (e.touches[0].clientX / game.documentZoom) : (e.clientX / game.documentZoom);
                    let y = game.isMobile(lib.config.touchscreen) ? (e.touches[0].clientY / game.documentZoom) : (e.clientY / game.documentZoom);
                    flame.style.transform = `translate(${x - event._yeyanX}px,${y - event._yeyanY}px)`;
                  }
                  event.yeyanend = function (e) {
                    delete event._yeyanTarget;
                    if (typeof event._yeyanBeginMove != 'number') return;
                    let flame = document.querySelector(`#th-yeyanflames${event._yeyanBeginMove}`);
                    let fire = document.querySelector('#th-yeyanfires' + event._yeyanBeginMove);
                    if(game.initJinDuTiao) game.initJinDuTiao(_status.event.player);
                    for (let k of game.players) {
                      //业炎可以选中自己的
                      //if (k == player) continue;
                      if (knock(flame, k)) {
                        if (k._thyeyanfires && k._thyeyanfires.length) {
                          if (lib.skill.th_yeyan.choose4Cards(event, player).length) event._yeyanTarget = k;
                        }
                        else event._yeyanTarget = k;
                      }
                    }
                    if (event._yeyanTarget) {
                      event.targets.add(event._yeyanTarget);
                      event.recheckCards = true;
                      if (!event._yeyanTarget._thyeyanfires || !event._yeyanTarget._thyeyanfires.length) {
                        event._yeyanTarget._thyeyanfires = [event._yeyanBeginMove];
                        ui.create.div('.th-yeyanflame.wanjia', event._yeyanTarget);
                        game.check();
                      } else {
                        event._yeyanTarget._thyeyanfires.add(event._yeyanBeginMove);
                        let fire = event._yeyanTarget.querySelector('.th-yeyanflame');
                        if (fire) fire.style.setProperty('--str', `"${event._yeyanTarget._thyeyanfires.length}"`);
                        event.needCards = true;
                        game.check();
                      }
                      fire.style.setProperty('--str', `"${get.translation(event._yeyanTarget)}×1"`);
                      fire.style.setProperty('--c', 'gray');
                      flame.style.setProperty('--str', ' ');
                      flame.style.setProperty('--ani', 'th-yeyanflame2 0s forwards');
                      flame.classList.add('extinguish');
                    } else {
                      fire.style.setProperty('--str', '"拖动分配"');
                    }
                    flame.style.transform = `none`;
                    delete event._yeyanBeginMove;
                    delete event._yeyanTarget;
                  }
                  event.playerListen = function (e) {
                    if (!this.player._thyeyanfires || !this.player._thyeyanfires.length) return;
                    if(game.initJinDuTiao) game.initJinDuTiao(_status.event.player);
                    let num = this.player._thyeyanfires.pop();
                    let flame = document.querySelector('#th-yeyanflames' + num);
                    if (flame) flame.restore();
                    let fire = this.player.querySelector('.th-yeyanflame');
                    event.recheckCards = true;
                    if (fire) {
                      if (!this.player._thyeyanfires.length) {
                        fire.remove();
                        event.targets.remove(this.player);
                      }
                      else {
                        if (this.player._thyeyanfires.length < 2) {
                          event.needCards = false;
                        }
                        fire.style.setProperty('--str', `"${this.player._thyeyanfires.length}"`)
                      }
                    }
                    if (!event.needCards) game.uncheck();
                    game.check();
                  }
                  document.addEventListener(game.isMobile(lib.config.touchscreen) ? 'touchmove' : 'mousemove', event.yeyanmove);
                  document.addEventListener(game.isMobile(lib.config.touchscreen) ? 'touchend' : 'mouseup', event.yeyanend);
                  for (let j of game.players) {
                    j.node.avatar.player = j;
                    j._thyeyanfires = [];
                    j.node.avatar.addEventListener(game.isMobile(lib.config.touchscreen) ? 'touchstart' : 'mousedown', event.playerListen);
                  }
                  player.chooseCard('h', function (card) {
                    if (!event.targets.length || !event.needCards) return false;
                    if (event.recheckCards) {
                      var cards = lib.skill.th_yeyan.choose4Cards(event, player);
                      if (cards.length) {
                        ui.selected.cards = cards;
                        for (var m of cards) {
                          m.classList.add('selected');
                          m.updateTransform(true);
                        }
                      }
                      event.recheckCards = false;
                    }
                    var suit = get.suit(card);
                    for (var i = 0; i < ui.selected.cards.length; i++) {
                      if (get.suit(ui.selected.cards[i]) == suit) return false;
                    }
                    return true;
                  }).set('prompt', '').set('complexCard', true).set('selectCard', function () {
                    if (event.targets.length) {
                      for (var i of event.targets) {
                        if (i._thyeyanfires && i._thyeyanfires.length > 1) return [4, 4];
                      }
                      return [0, Infinity];
                    }
                    return [Infinity, Infinity];
                  }).ai = (card) => 1;
                } else {
                  var players = game.filterPlayer(function (current) {
                    return get.damageEffect(current,player,player,'fire')>0;
                  });
                  if (players.length) players.sort(function (a, b) {
                    return get.damageEffect(a,player,player,'fire') - get.damageEffect(b,player,player,'fire');
                    //return get.attitude(player, a) - get.attitude(player, b)
                  });
                  //alert_old(get.translation(players));
                  var cards = lib.skill.th_yeyan.choose4Cards(event, player);
                  if (cards.length) {
                    if (players.length) {
                      /*if (game.me.identity == 'fan' && players.contains(game.zhu)) {
                        event.targets = [players.shift()];
                        event.targets[0]._thyeyanfires = [0, 1, 2];
                      } else if (players.length > 1) {
                        event.targets = [players.shift(), players.shift()];
                        event.targets[0]._thyeyanfires = [0, 1];
                        event.targets[1]._thyeyanfires = [2];
                      }*/
                      if(game.me.identity == 'fan' && players.contains(game.zhu)) {
                          event.targets = [game.zhu];
                          event.targets[0]._thyeyanfires = [0, 1, 2];
                      }else if(players[0].isLinked()||players.length == 1) {
                          event.targets = [players.shift()];
                          event.targets[0]._thyeyanfires = [0, 1, 2];
                      }else {
                          event.targets = [players.shift(), players.shift()];
                          event.targets[0]._thyeyanfires = [0, 1];
                          event.targets[1]._thyeyanfires = [2];
                      }
                      event.needCards = true;
                      event._result = {
                        bool: true,
                        cards: cards,
                      }
                    }
                  } else {
                    /*if (players.length > 1) {
                      event.targets = [players.shift(), players.shift()];
                      event.targets[0]._thyeyanfires = [0];
                      event.targets[1]._thyeyanfires = [1];
                      if (players.length) {
                        event.targets.push(players.shift());
                        event.targets[2]._thyeyanfires = [2];
                      }
                    }*/
                    //修复ai逻辑
                    if (players.length > 0) {
                        event.targets = [];
                        for(var i=0;i<players.length;i++) {
                            if(get.attitude(player,players[i])>0) continue;
                            event.targets.push(players[i]);
                            event.targets[i]._thyeyanfires = [i];
                        }
                    }
                    event._result = {
                      bool: true,
                    }
                  }
                }
                'step 1'
                game.thunderAllowTouch();
                delete window.qhly_forbidPlayerWindow;
                if (event.dibeijing) event.dibeijing.remove();
                document.removeEventListener(game.isMobile(lib.config.touchscreen) ? 'touchmove' : 'mousemove', event.yeyanmove);
                document.removeEventListener(game.isMobile(lib.config.touchscreen) ? 'touchend' : 'mouseup', event.yeyanend);
                for (var i of game.players) {
                  i.node.avatar.removeEventListener(game.isMobile(lib.config.touchscreen) ? 'touchstart' : 'mousedown', event.playerListen);
                }
                var flames = document.querySelectorAll('.th-yeyanflame');
                for (var i of flames) {
                  i.remove();
                }
                if (event.targets && event.targets.length && result.bool) {
                  event.targets.sortBySeat();
                  if (event.needCards) {
                    player.discard(result.cards);
                    if(window.getStrength(true,false)) player.loseHp(3);
                  }
                  player.awakenSkill('th_yeyan');
                  player.logSkill('th_yeyan', event.targets)
                } else event.finish();
                'step 2'
                for (var i = 0; i < event.targets.length; i++) {
                  event.targets[i].damage(event.targets[i]._thyeyanfires.length, 'fire');
                }
              },
              ai: {
                order: function (item, player) {
                  var cards = lib.skill.th_yeyan.choose4Cards(event, player);
                  if (cards) return 10;
                  return 1
                },
                fireAttack: true,
                result: {
                  player: function (player, target) {
                    var cards = lib.skill.th_yeyan.choose4Cards(_status.event, player);
                    //防止一直在放技能，检测最多放两遍
                    if(player.getStat('skill').th_yeyan>=2) return -1;
                    if (cards.length) {
                      if (game.hasPlayer(function (current) {
                        return get.damageEffect(current, player)
                      })) return 1;
                    } else if (game.filterPlayer(function (current) {
                      return get.damageEffect(current, player)
                    }).length > 1) return 1;
                    return 0;
                  }
                }
              }
            },















})
