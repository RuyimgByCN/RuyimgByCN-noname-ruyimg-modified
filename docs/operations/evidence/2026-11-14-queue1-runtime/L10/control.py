import json
import os
import pathlib
import urllib.request

BASE = pathlib.Path(__file__).resolve().parent


def evaluate(expression):
    meta = json.loads((BASE / ('active-session-' + os.environ.get('L10_SESSION', 'l10') + '.json')).read_text())
    request = urllib.request.Request(
        f"http://127.0.0.1:{meta['controlPort']}", data=expression.encode(), method='POST'
    )
    response = json.loads(urllib.request.urlopen(request, timeout=55).read())
    result = response.get('result', {})
    if result.get('exceptionDetails'):
        raise RuntimeError(json.dumps(response, ensure_ascii=False))
    value = result.get('result', {}).get('value')
    if isinstance(value, str):
        try:
            return json.loads(value)
        except json.JSONDecodeError:
            return value
    return value


def wait_for(names, allow_done=False):
    expression = """new Promise((resolve,reject)=>{
      let ticks=0;
      const test=()=>{
        if (window.l10Done && ALLOW_DONE) return resolve(JSON.stringify(window.l10Snapshot()));
        if (_status.paused && NAMES.includes(_status.event.name))
          return resolve(JSON.stringify(window.l10Snapshot()));
        if(++ticks>700) return reject(Error('choice timeout at '+_status.event.name+' step '+_status.event.step));
        setTimeout(test,20);
      }; test();
    })""".replace('ALLOW_DONE', json.dumps(allow_done)).replace('NAMES', json.dumps(names))
    return evaluate(expression)


def start(name, fixture):
    evaluate('window.l10Case=' + json.dumps(name) + ';window.l10Fixture=' + json.dumps(fixture) + ';true')
    return evaluate((BASE / 'upstream-setup.js').read_text())


def snapshot():
    return evaluate('JSON.stringify(window.l10Snapshot())')


def cancel():
    return evaluate('_status.clicked=false;ui.click.cancel();true')


def ok():
    return evaluate('_status.clicked=false;ui.click.ok();true')


if __name__ == '__main__':
    import sys
    print(json.dumps(evaluate(sys.stdin.read()), ensure_ascii=False, indent=2))
