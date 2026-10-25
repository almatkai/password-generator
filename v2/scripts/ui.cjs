// Executes actual pinned FormRuntime + ExtensionRuntimeSurface with real React.
// Chrome, icon rendering, other surfaces, and OS hooks are explicit seams.
const fs=require('node:fs'),path=require('node:path'),Module=require('node:module'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),v1=path.join(root,'.work/v1'),ui=path.join(root,'.work/ui');
const resolve=(s,p=v1)=>require.resolve(s,{paths:[p]});
const React=require(resolve('react',ui)),Renderer=require(resolve('react-test-renderer',ui));
const esbuild=require(require.resolve('esbuild',{paths:[path.dirname(resolve('tsx'))]}));
const work=path.join(root,'.work/ui-compiled');fs.mkdirSync(work,{recursive:true});
for(const file of ['form-runtime.tsx','index.tsx','action-runtime-registry.tsx']){
 const source=fs.readFileSync(path.join(root,'fixtures/beta/src/renderer/src/raycast-api',file),'utf8');
 fs.writeFileSync(path.join(work,file.replace('.tsx','.js')),esbuild.transformSync(source,{loader:'tsx',format:'cjs',jsx:'automatic'}).code);
}
global.IS_REACT_ACT_ENVIRONMENT=true;
global.window={addEventListener(){},removeEventListener(){},clearTimeout(){}};
const Context=React.createContext({});const no=()=>{};
const old=Module._load;
Module._load=function(name,parent,main){
 if(name==='react'||name==='react/jsx-runtime')return old(resolve(name,ui),parent,main);
 if(name==='./form-runtime'||name==='./action-runtime-registry')return old(path.join(work,name.slice(2)+'.js'),parent,main);
 if(name==='../../ui/primitives')return {Hint:no,HintBar:no,Kbd:no,ViewHeader:no};
 if(name==='./detail-runtime')return {NativeColorWheel:no,parseNativeColorWheel:()=>null,DetailRuntime:()=>{throw Error('unrelated detail surface')}};
 if(name==='./grid-runtime')return {GridRuntime:()=>{throw Error('unrelated grid surface')}};
 if(name==='./list-runtime')return {ListRuntime:()=>{throw Error('unrelated list surface')}};
 if(name==='./action-runtime')return {Action:no,ActionPanel:no,ActionPanelOverlay:props=>React.createElement('overlay',props)};
 if(name==='./hooks')return {NavigationContext:Context,setRuntimeContext:no};
 return old.apply(this,arguments);
};
const {FormRuntime,reconcileFormValues}=require(path.join(work,'form-runtime.js'));
const {ExtensionRuntimeSurface}=require(path.join(work,'index.js'));
const {act}=Renderer;
const node=(id,type,value,extra={})=>({type,props:{id,title:id,value,...extra}});
const initial={type:'Form',children:[node('length','Form.TextField','20'),node('useNumbers','Form.Checkbox',true),node('useSymbols','Form.Checkbox',true)]};
(async()=>{
 let tree,submits=[],changes=[];
 const props={sessionId:'fixture-session',extensionId:'com.tezbar.native.password-generator',commandName:'generate-random-password',title:'Password Generator',root:initial,actions:[{id:'generate-1',kind:'submit-form',title:'Generate and Copy Password'}],onBack:no,onSearchTextChanged:no,onLoadMore:no,onInvokeAction:(...args)=>submits.push(args)};
 await act(async()=>{tree=Renderer.create(React.createElement(ExtensionRuntimeSurface,props));});
 const inputs=()=>tree.root.findAllByType('input');
 assert.equal(inputs()[0].props.value,'20');assert.equal(inputs()[1].props.checked,true);assert.equal(inputs()[2].props.checked,true);
 await act(async()=>{inputs()[0].props.onChange({target:{value:'32'}});inputs()[1].props.onChange({target:{checked:false}});});
 assert.equal(submits.length,0,'submit-only edits have no side effects');
 await act(async()=>tree.root.findByType('form').props.onSubmit({preventDefault:no}));
 assert.deepEqual(submits.pop(),['generate-1',{length:'32',useNumbers:false,useSymbols:true}]);
 // A queued server response repeats original values; a newer local draft survives.
 await act(async()=>inputs()[0].props.onChange({target:{value:'48'}}));
 const next={...props,root:JSON.parse(JSON.stringify(initial)),actions:[{id:'retry-2',kind:'action',title:'Retry Copy'},{id:'generate-2',kind:'submit-form',title:'Generate and Copy Password'}]};
 await act(async()=>tree.update(React.createElement(ExtensionRuntimeSurface,next)));
 assert.equal(inputs()[0].props.value,'48');assert.equal(inputs()[1].props.checked,false);
 await act(async()=>tree.root.findByType('form').props.onSubmit({preventDefault:no}));assert.deepEqual(submits.pop(),['generate-2',{length:'48',useNumbers:false,useSymbols:true}]);
 await act(async()=>tree.unmount());
 const ref=React.createRef();let submitted;
 const rootNode={type:'Form',children:[node('text','Form.TextField',undefined,{defaultValue:'ignored'}),node('check','Form.Checkbox',undefined,{defaultValue:true}),{type:'Form.Dropdown',props:{id:'pick',value:'b'},children:[{type:'Form.Dropdown.Item',props:{title:'A',value:'a'}},{type:'Form.Dropdown.Item',props:{title:'B',value:'b'}}]}]};
 await act(async()=>{tree=Renderer.create(React.createElement(FormRuntime,{ref,root:rootNode,title:'Collector',onBack:no,onOpenActions:no,onInvokeAction:no,onSubmitForm:v=>submitted=v,onChangeField:(...a)=>changes.push(a)}));});
 assert.deepEqual(ref.current.getValues(),{text:'',check:false,pick:'b'},'actual beta collector consumes value, not defaultValue/storeValue');
 await act(async()=>tree.root.findByType('select').props.onChange({target:{value:'a'}}));
 await act(async()=>tree.root.findByType('form').props.onSubmit({preventDefault:no}));assert.deepEqual(submitted,{text:'',check:false,pick:'a'});
 assert.deepEqual(changes,[]);
 // Surface callback payload is exactly {value}, verified with actual component.
 await act(async()=>{tree.unmount();});
 const controlled={...props,root:{type:'Form',children:[node('check','Form.Checkbox',true,{actionId:'changed'})]}};
 await act(async()=>{tree=Renderer.create(React.createElement(ExtensionRuntimeSurface,controlled));});
 await act(async()=>tree.root.findByType('input').props.onChange({target:{checked:false}}));assert.deepEqual(submits.pop(),['changed',{value:false}]);
 await act(async()=>tree.unmount());
 Module._load=old;
 console.log('Actual pinned FormRuntime/Surface: defaults, checkbox, dropdown, complete submission, callbacks and queued draft reconciliation PASS. OS/browser QA not run.');
})().catch(e=>{console.error(e);process.exitCode=1});
