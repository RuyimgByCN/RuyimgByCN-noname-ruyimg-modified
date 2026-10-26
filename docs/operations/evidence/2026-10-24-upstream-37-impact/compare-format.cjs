const ts=require('/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/node_modules/typescript/lib/typescript.js');
const {execFileSync}=require('node:child_process');const fs=require('node:fs');
const cwd='/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname';const out='/tmp/noname-next-upstream-37-impact-20261024';
const printer=ts.createPrinter({removeComments:true,newLine:ts.NewLineKind.LineFeed});
const rows=[];for(const sha of ['ad1065b0','a15a1e4f']){
 const paths=execFileSync('git',['diff-tree','--no-commit-id','--name-only','-r',sha],{cwd,encoding:'utf8'}).trim().split('\n');
 for(const p of paths){if(!/\.(js|ts)$/.test(p))continue;
 const render=rev=>{const raw=execFileSync('git',['show',rev+':'+p],{cwd,encoding:'utf8',maxBuffer:20*1024*1024});const ast=ts.createSourceFile(p,raw,ts.ScriptTarget.Latest,true);return{printed:printer.printFile(ast),diagnostics:ast.parseDiagnostics.length}};
 const a=render(sha+'^'),b=render(sha);const equal=a.printed===b.printed;
 rows.push({sha,path:p,equal_without_comments_and_format:equal,parse_errors:[a.diagnostics,b.diagnostics]});
 if(!equal){const stem=sha+'-'+p.replaceAll('/','_');fs.writeFileSync(out+'/'+stem+'.before',a.printed);fs.writeFileSync(out+'/'+stem+'.after',b.printed);}
 }
}
fs.writeFileSync(out+'/format-comparison.json',JSON.stringify(rows,null,2)+'\n');console.log(JSON.stringify({files:rows.length,equal:rows.filter(x=>x.equal_without_comments_and_format).length,remaining:rows.filter(x=>!x.equal_without_comments_and_format)},null,2));
