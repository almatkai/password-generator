"""Create reviewable additive beta registration patch + test-only harness transforms."""
from pathlib import Path
import json, difflib, re
root=Path(__file__).resolve().parents[1]
base=root/'fixtures/beta/src-tauri/src/native_core/extensions'
out=root/'integration/generated';out.mkdir(parents=True,exist_ok=True)
manifest=json.loads((root/'manifest.json').read_text())
changes={}
s=(base/'v2_runtime.rs').read_text()
s=s.replace('"amphetamine" | "audio-device"', '"password-generator" | "amphetamine" | "audio-device"')
s=s.replace('match slug {', 'match slug {\n        "password-generator" => super::v2_password_generator::commands(),',1)
pos=s.index('    match slug {',s.index('pub(super) async fn handle'))
s=s[:pos]+s[pos:].replace('    match slug {','    match slug {\n        "password-generator" => super::v2_password_generator::handle(ctx, command, operation, payload, state).await,',1)
changes['v2_runtime.rs']=s
catalog=json.loads((base/'v2_catalog.json').read_text());assert not any(x['slug']==manifest['slug'] for x in catalog);catalog.append(manifest)
changes['v2_catalog.json']=json.dumps(catalog,indent=2,ensure_ascii=False)+'\n'
changes['v2_catalog.rs']=(base/'v2_catalog.rs').read_text().replace('items.len(), 11','items.len(), 12')
changes['mod.rs']=(base/'mod.rs').read_text().replace('mod v2_sdk;', 'mod v2_sdk;\n#[path = "../../../native-extensions/password-generator/src/adapter.rs"]\nmod v2_password_generator;')
patch=[]
for f,s in changes.items():
 (out/f).write_text(s)
 patch.extend(difflib.unified_diff((base/f).read_text().splitlines(True),s.splitlines(True),fromfile='a/src-tauri/src/native_core/extensions/'+f,tofile='b/src-tauri/src/native_core/extensions/'+f))
(root/'integration/registration.patch').write_text(''.join(patch))
# Test-only copies: disable upstream cfg(test) blocks to avoid unrelated OS tests.
# Their runtime functions are unchanged. NativeSessions dispatch remains real.
for f in ['v2_runtime.rs','v2_catalog.rs','native.rs','registry.rs','v2_sdk.rs']:
 s=changes.get(f,(base/f).read_text()).replace('#[cfg(test)]','#[cfg(any())]')
 (out/('harness_'+f)).write_text(s)
s=(root/'fixtures/beta/src-tauri/src/native_core/common.rs').read_text().replace('#[cfg(test)]','#[cfg(any())]')
(out/'harness_common.rs').write_text(s)
# Unrelated compiled adapters are explicit nonexecuting seams. They preserve the
# existing catalog command inventory; this harness asserts only the new adapter.
lines=[]
for entry in catalog[:-1]:
 slug=entry['slug'];module='v2_'+slug.replace('-','_')
 commands=[c['name'] for c in entry['commands']]
 lines.append(f'''pub mod {module} {{
 pub fn commands() -> &'static [&'static str] {{ &{json.dumps(commands)} }}
 pub async fn handle(_: &crate::native_core::common::CoreContext,_: &str,_: &str,_: &serde_json::Value,_: &mut serde_json::Value)->crate::native_core::common::CoreResult {{ panic!("unrelated adapter seam: {slug}") }}
}}''')
(out/'unrelated.rs').write_text('\n'.join(lines)+'\n')
print('Prepared additive registration patch (12 pinned catalog entries), exact runtime harness transforms.')
