import json, csv
def pct(vals,p):
    vals=sorted(v for v in vals if v is not None)
    if not vals: return None
    return round(vals[min(len(vals)-1,int(round(p/100*(len(vals)-1))))],3)
def analyze(path, ic, label):
    evs=json.load(open(path))["events"]
    cnt={}
    for e in evs:
        if e.get("name")=="EventDispatch": k=(e["pid"],e["tid"]); cnt[k]=cnt.get(k,0)+1
    pid,tid=max(cnt,key=cnt.get)
    def main(e): return (e.get("pid"),e.get("tid"))==(pid,tid)
    pres=sorted(e["ts"] for e in evs if e.get("name")=="AnimationFrame::Presentation")
    bmfs=sorted(e["ts"] for e in evs if e.get("name")=="BeginMainThreadFrame" and main(e))
    rafs=sorted(e["ts"] for e in evs if e.get("name")=="FireAnimationFrame" and main(e))
    styles=sorted(e["ts"] for e in evs if e.get("name")=="AnimationFrame::StyleAndLayout" and main(e))
    import bisect
    def nxt(arr,t): i=bisect.bisect_left(arr,t); return arr[i] if i<len(arr) else None
    def prv(arr,t): i=bisect.bisect_right(arr,t); return arr[i-1] if i else None
    downs=sorted(e["ts"] for e in evs if e.get("name")=="EventDispatch" and main(e)
                 and (e.get("args",{}).get("data",{}) or {}).get("type")=="mousedown")
    page=[float(r["input_to_paint_ms"]) for r in csv.DictReader(open(ic)) if r.get("input_to_paint_ms")]
    recs=[]
    for t0 in downs:
        prev_bmf=prv(bmfs,t0); next_bmf=nxt(bmfs,t0)
        raf=nxt(rafs,t0); style=nxt(styles,raf) if raf else None
        present=nxt(pres,style) if style else None   # 该帧的呈现
        prev_present=prv(pres,t0)                    # 输入前最近一次呈现
        recs.append(dict(t0=t0,prev_bmf=prev_bmf,next_bmf=next_bmf,raf=raf,style=style,present=present,prev_present=prev_present))
    ms=lambda a,b: None if a is None or b is None else round((a-b)/1000,3)
    out={label:{
        "n_clicks":len(downs),
        "输入相位(距上一BeginMainFrame,ms)":pct([ms(r["t0"],r["prev_bmf"]) for r in recs],50),
        "input→nextBeginMainFrame":pct([ms(r["next_bmf"],r["t0"]) for r in recs],50),
        "input→firstRAF":pct([ms(r["raf"],r["t0"]) for r in recs],50),
        "RAF→该帧StyleAndLayout":pct([ms(r["style"],r["raf"]) for r in recs],50),
        "input→该帧Presentation(真i2p)":pct([ms(r["present"],r["t0"]) for r in recs],50),
        "真i2p_p95":pct([ms(r["present"],r["t0"]) for r in recs],95),
        "该帧Presentation距上一次Presentation(帧数间隔*8.33)":pct([ (r["present"]-r["prev_present"])/1000 if r["present"] is not None and r["prev_present"] is not None else None for r in recs],50),
        "页面口径i2p":pct(page,50),
        "页面口径p95":pct(page,95),
        "真i2p−页面i2p":pct([(ms(r["present"],r["t0"])-p) if ms(r["present"],r["t0"]) is not None else None for r,p in zip(recs,page)],50),
    }}
    return out,recs
sab,rs=analyze("/tmp/pa07/trace-sab.json","/tmp/pa07/out-sab/interactions.csv","sab")
base,rb=analyze("/tmp/pa07/trace-base.json","/tmp/pa07/out-base/interactions.csv","base")
print(json.dumps({"sab":sab["sab"],"base":base["base"]},indent=1,ensure_ascii=False))
