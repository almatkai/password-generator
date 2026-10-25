"""Copy licenses and selected primary-source crate evidence from the locked cache."""
from pathlib import Path
import json,hashlib,re,tarfile,shutil
r=Path(__file__).resolve().parents[1]
meta=json.loads((r/'.work/cargo-metadata.json').read_text());pins={}
for block in (r/'Cargo.lock').read_text().split('[[package]]')[1:]:
 p=dict(re.findall(r'^([a-z]+) = \"([^\"]*)\"',block,re.M))
 pins[p['name'],p['version']]=p
records=[]
for pkg in meta['packages']:
 if not pkg.get('source'):continue
 src=Path(pkg['manifest_path']).parent;name=pkg['name'];version=pkg['version'];pin=pins[name,version];licenses=[]
 for f in src.iterdir():
  if f.is_file() and (f.name.upper().startswith(('LICENSE','LICENCE','COPYING','NOTICE'))):
   dst=r/'fixtures/library/crates'/f'{name}-{version}'/f.name;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(f,dst);licenses.append(dst.relative_to(r).as_posix())
 records.append({'name':name,'version':version,'source':pkg['source'],'checksum':pin['checksum'],'declaredLicense':pkg['license'],'licenseFiles':licenses})
 if (name,version) in [('getrandom','0.2.17'),('chacha20poly1305','0.10.1'),('zeroize','1.9.0')]:
  cache=src.parents[1].parent/'cache'/src.parent.name/f'{name}-{version}.crate'
  assert hashlib.sha256(cache.read_bytes()).hexdigest()==pin['checksum']
  dst=r/'fixtures/library'/cache.name;shutil.copyfile(cache,dst)
  for f in ['Cargo.toml','README.md','src/lib.rs','src/macos.rs','src/windows.rs','src/linux_android.rs','src/util_libc.rs']:
   if (src/f).exists():
    out=r/'fixtures/library/crates'/f'{name}-{version}'/(f+'.fixture' if f=='Cargo.toml' else f);out.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src/f,out)
(r/'fixtures/library/cargo-pins.json').write_text(json.dumps(records,indent=2)+'\n')
print(f'Copied license evidence for {len(records)} locked crates; three selected crate archives match Cargo checksums.')
