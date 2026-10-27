// Executes the immutable OWN source with deterministic crypto interception; never live secrets.
const fs = require('node:fs');
const path = require('node:path');
const Module = require('node:module');
const root = path.resolve(__dirname, '..');
const work = path.join(root, '.work/v1');
const resolve = p => require.resolve(p, { paths: [work] });
const esbuild = require(require.resolve('esbuild', {paths:[path.dirname(resolve('tsx'))]}));
const crypto = require('node:crypto');
const originalRandomInt = crypto.randomInt;
let trace = [], stream = [], counter = 0, events = [], preferences = {};
crypto.randomInt = upper => {
  const choice = stream.length ? stream.shift() : (counter++ * 17 + 3) % upper;
  if (!(choice >= 0 && choice < upper)) throw new Error('bad fixture choice');
  trace.push([upper, choice]); return choice;
};
let injectError = false;
const api = {
  getPreferenceValues: () => preferences,
  Clipboard: { copy: async text => { events.push(['copy', text]); if (injectError) throw new Error('fixture copy failure'); } },
  Toast: { Style: { Success: 'success', Failure: 'failure' } },
  PopToRootType: { Suspended: 'suspended' },
  showHUD: async (...args) => { events.push(['hud', ...args]); },
  showToast: async (...args) => { events.push(['toast', ...args]); },
  Form: Object.assign(function Form(){}, { TextField: 'Form.TextField', Checkbox: 'Form.Checkbox', Description: 'Form.Description' }),
  ActionPanel: 'ActionPanel', Action: { SubmitForm: 'Action.SubmitForm' },
};
const oldLoad = Module._load;
Module._load = function(name, parent, main) {
  if (name === '@raycast/api') return api;
  if (name === 'react/jsx-runtime') return oldLoad(require.resolve(name,{paths:[fs.realpathSync(path.join(work,'node_modules/@raycast/api'))]}),parent,main);
  if (name.startsWith('@scure/')) return oldLoad(resolve(name), parent, main);
  return oldLoad.apply(this, arguments);
};
const compiled = path.join(root, '.work/oracle'); fs.mkdirSync(path.join(compiled, 'lib'), { recursive: true });
for (const file of ['lib/passwords.ts','generate-random-password.tsx','generate-memorable-password.tsx','copy-random-password.ts']) {
  const source = fs.readFileSync(path.join(root, 'fixtures/own/src', file), 'utf8');
  const out = esbuild.transformSync(source, { loader: file.endsWith('tsx') ? 'tsx':'ts', format:'cjs', jsx:'automatic' }).code;
  fs.writeFileSync(path.join(compiled,file.replace(/\.tsx?$/,'.js')),out);
}
const lib = require(path.join(compiled,'lib/passwords.js'));
const vectors = [];
function vector(kind, input, choices = []) {
  trace=[]; counter=0; stream=[...choices];
  let result;
  try { result = { value: kind==='length' ? lib.parsePasswordLength(input) : kind==='random' ? lib.generatePassword(input.length,input.options) : lib.generateMemorablePassword(input) }; }
  catch (e) { result = { error: e.message }; }
  vectors.push({kind,input,choices,trace,...result});
}
for (const numbers of [false,true]) for (const symbols of [false,true]) for (const length of [5,20,64,4,65,5.5]) vector('random',{length,options:{numbers,symbols}});
for (const s of ['20',' 20 ','\uFEFF00005\u00A0','00000000000000000000000000000000000000000000000000000000000000005','4','65','12.5','1e1','0x20','+20','２０','\u008520','\u200b20','', '9'.repeat(400)]) vector('length',s);
for (const s of ['', '{word:capitalize}-{word}-{word}-{number:3}', '{word:uppercase}-{word:lowercase}-{word:capitalize}', '{word:wrong}', '{number:0}', '{number:65}', '{number:1e1}', '{number:0x10}', '{number:0b101}', '{number:0o10}', '{number:00005}', '{number:0x}', '{number:1_0}', '{number:Infinity}', '{number:NaN}', '{number:0xg}', '{number:0b2}', '{number:1e0}', '{number:-1}', '{number:1.0}', '{number:é}', '{word:}', '{{word}}', 'hello{unknown}', '{symbol:5}', '{random:64}', '{number:64}{number:64}', 'a'.repeat(129), 'a'.repeat(257), '😀'.repeat(64), '😀'.repeat(65), '\uFEFF{word}\u0085']) vector('memorable',s);
vector('memorable','{word:lowercase}-{word:lowercase}-{word}',[0,0,0,1,1,2,1]);
fs.writeFileSync(path.join(root,'fixtures/oracle/engine.json'),JSON.stringify(vectors,null,2)+'\n');
const flows=[];
function describe(element) {
  if (!element) return null;
  const p=element.props;
  return {type: typeof element.type === "function" ? "Form" : element.type,...Object.fromEntries(Object.entries(p).filter(([k,v])=>k!=='children'&&k!=='actions'&&typeof v!=='function')),children: [p.children].flat(Infinity).filter(Boolean).map(describe),actions:p.actions?describe(p.actions):undefined};
}
(async()=>{
 for (const command of ['generate-random-password','copy-random-password','generate-memorable-password']) {
  const mod = require(path.join(compiled,command+'.js'));
  const cases=command==='generate-random-password' ? [ [{}, {length:'20',useNumbers:true,useSymbols:true}], [{hideAfterCopy:true,storePasswordLength:false,rememberCharacterGroups:false},{length:' 5 ',useNumbers:false,useSymbols:false}], [{},{length:'oops',useNumbers:true,useSymbols:false}]] : command==='generate-memorable-password' ? [[{}, {format:'   '}], [{hideAfterCopy:true,rememberCustomFormat:false},{format:' {word:capitalize}-{number:3} '}],[{}, {format:'{word:bad}'}]] : [[{},null],[{length:'5',useNumbers:false,useSymbols:false},null],[{length:'no'},null]];
  for(const [prefs,values] of cases) {
   for(const fail of [false,true]) {
    preferences=prefs;events=[];trace=[];counter=0;stream=[];injectError=fail;
    let form, error;
    try { if(values) { const el=mod.default(); form=describe(el); await el.props.actions.props.children.props.onSubmit(values); } else await mod.default(); }
    catch(e) {error=e.message;}
    flows.push({command,preferences:prefs,values,copyFailure:fail,form,trace,events,error});
   }
  }
 }
 fs.writeFileSync(path.join(root,'fixtures/oracle/commands.json'),JSON.stringify(flows,null,2)+'\n');
 crypto.randomInt=originalRandomInt;Module._load=oldLoad;
 console.log(`Recorded ${vectors.length} engine vectors and ${flows.length} command flows from OWN source.`);
})().catch(e=>{throw e});
