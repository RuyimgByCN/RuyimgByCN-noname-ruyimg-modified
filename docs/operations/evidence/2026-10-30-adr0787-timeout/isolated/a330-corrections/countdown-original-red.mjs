import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';
const web=path.resolve('apps/web-client');
const ts=(await import(pathToFileURL(path.join(web,'node_modules/typescript/lib/typescript.js')))).default;
const {ref,computed}=await import(pathToFileURL(path.join(web,'node_modules/vue/dist/vue.runtime.esm-bundler.js')));
const raw=fs.readFileSync(path.resolve('../evidence/a330-corrections/before/apps/web-client/src/views/BattleView.vue'),'utf8').match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
const ast=ts.createSourceFile('BattleView.ts',raw,ts.ScriptTarget.Latest,true);
const names=new Set(['countdownNow','countdownTimer','choiceSecondsLeft']);
const declarations=ast.statements.filter(s=>ts.isVariableStatement(s)&&s.declarationList.declarations.some(d=>names.has(d.name.getText(ast)))).map(s=>s.getText(ast)).join('\n');
const compiled=ts.transpileModule(declarations+'\nreturn choiceSecondsLeft;', {compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText;
const actual=[];const originalNow=Date.now;
for(const skew of [60000,-60000]){Date.now=()=>1000000+skew;const value=new Function('ref','computed','store','setInterval',compiled)(ref,computed,{choice:{deadline_unix_ms:1010000}},()=>1);actual.push({skew,seconds:value.value,expected:10});}
Date.now=originalNow;console.log(JSON.stringify({source:'preserved reviewed BattleView',actual}));assert.ok(actual.every(s=>s.seconds===s.expected),'original countdown must reflect server time');
