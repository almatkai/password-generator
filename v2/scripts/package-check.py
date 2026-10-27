"""Audit the actual Cargo archive and rebuild it without absolute host paths."""
from pathlib import Path
import subprocess,tarfile,tempfile,json,hashlib,os
r=Path(__file__).resolve().parents[1]
subprocess.run(['cargo','package','--manifest-path',str(r/'Cargo.toml'),'--locked','--allow-dirty','--no-verify'],check=True)
crate=r/'target/package/tezbar-password-generator-v2-0.1.0.crate'
index=json.loads((r/'INDEX.json').read_text())
with tempfile.TemporaryDirectory(prefix='password-package-offline-') as tmp:
 dest=Path(tmp)
 with tarfile.open(crate) as t:
  names=t.getnames();prefix='tezbar-password-generator-v2-0.1.0/'
  assert all(n.startswith(prefix) and '..' not in Path(n).parts for n in names)
  files={n[len(prefix):] for n in names}
  assert not any(p.startswith(('.work/','target/','artifacts/')) or '/node_modules/' in p or '/dist/' in p for p in files)
  assert not any(p.endswith('/Cargo.toml') for p in files),'nested manifests must be .fixture templates'
  for item in index['files']:
   name=item['path'];archived='Cargo.toml.orig' if name=='Cargo.toml' else name
   assert archived in files,('missing from archive',name)
   raw=t.extractfile(prefix+archived).read();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'],name
  for member in t.getmembers():
   assert member.isfile(),('unexpected archive entry',member.name)
   target=dest/member.name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(t.extractfile(member).read())
 extracted=dest/'tezbar-password-generator-v2-0.1.0'
 # Path-free artifacts: code/module/include paths are relative, independent of
 # original host, original repo, NODE_PATH, npm installation, or Git root.
 env=os.environ.copy();env.pop('NODE_PATH',None);env['CARGO_TARGET_DIR']=str(dest/'build')
 for toolchain in ['stable','1.85.0']:
  for args in [['test'],['clippy','--all-targets','--','-D','warnings']]:
   cmd=['cargo','+'+toolchain]+args[:1]+['--locked','--offline']+args[1:]
   subprocess.run(cmd,cwd=extracted,env=env,check=True)
print('Actual archive visibility/content and extracted stable + Rust 1.85 locked offline test/strict Clippy PASS')
