new Promise((resolve, reject) => {
  let ticks = 0;
  const responses = [];
  const tick = () => {
    const { game, ui, _status } = q1;
    if (runtimeError) return reject(Error(runtimeError));
    const event = _status.event;
    if (_status.paused && event?.name === runtimeExpected) {
      return resolve(JSON.stringify({ event: event.name, prompt: event.prompt,
        player: game.players.indexOf(event.player), forced: event.forced,
        confirm: ui.confirm?.str, controls: event.controls, responses,
        selectableTargets: (ui.selectableTargets || []).map(p => game.players.indexOf(p)),
        hand: game.me.getCards('h').map(c => ({ name: c.name, suit: c.suit, rank: c.number })),
        buttons: ui.dialog?.buttons?.map(b => ({ text: b.innerText, link: typeof b.link === 'string' ? b.link : b.link?.name })),
      }));
    }
    if (_status.paused && ['chooseToRespond', 'chooseToUse'].includes(event?.name)) {
      responses.push({ name: event.name, player: game.players.indexOf(event.player), decision: 'cancel' });
      _status.clicked = false;
      ui.click.cancel();
    }
    if (runtimeDone) return reject(Error('completed before expected ' + runtimeExpected));
    if (++ticks > 600) return reject(Error('choice timeout at ' + event?.name + ', expected ' + runtimeExpected));
    setTimeout(tick, 20);
  };
  tick();
});
