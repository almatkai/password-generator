"""Full Draft 2020-12 schema, negative cases, immutable evidence and visibility."""
from pathlib import Path
import json,hashlib,copy,base64,tarfile,re,subprocess,tempfile
from jsonschema import Draft202012Validator
r=Path(__file__).resolve().parents[1]
schema=json.loads((r/'fixtures/beta/schemas/extension-v2.schema.json').read_text());Draft202012Validator.check_schema(schema);validator=Draft202012Validator(schema)
m=json.loads((r/'manifest.json').read_text());validator.validate(m)
assert m['id']=='com.tezbar.native.password-generator' and m['replaces']==['raycast.password-generator']
assert {x['platform']:x['state'] for x in m['verification']}=={'macOS':'not-run','Windows':'not-run'}
own=json.loads((r/'fixtures/own/package.json').read_text());assert m['preferences']==own['preferences']
assert [{k:v for k,v in x.items() if k!='runtime'} for x in m['commands']]==own['commands']
negative=[]
for key,value in [('author','invented'),('runtime','javascript'),('schemaVersion',1),('icon','/tmp/icon.png'),('id','raycast.password-generator'),('verification',[{'platform':'macOS','state':'pass'}]),('commands',[])]:
 bad=copy.deepcopy(m);bad[key]=value;negative.append(bad)
bad=copy.deepcopy(m);bad['commands'][0]['platforms']=['Solaris'];negative.append(bad)
bad=copy.deepcopy(m);bad['commands'][0]['runtime']='native';negative.append(bad)
bad=copy.deepcopy(m);bad['platforms']=['macOS','macOS'];negative.append(bad)
for bad in negative:assert list(validator.iter_errors(bad)),bad
icon=(r/'icons/password-generator.png').read_bytes();assert len(icon)==17803
assert hashlib.sha256(icon).hexdigest()=='7d9f69d316d24698d8dd643a8f2b4a0532b2a71065eb4096a7d39864e1529518'
assert icon==(r/'fixtures/original/password-generator-icon.png').read_bytes()
with tarfile.open(r/'fixtures/library/scure-bip39-2.4.0.tgz') as t:
 assert t.extractfile('package/wordlists/english.js').read()==(r/'fixtures/library/bip39-2.4.0/wordlists/english.js').read_bytes()
 assert t.extractfile('package/LICENSE').read()==(r/'fixtures/library/bip39-2.4.0/LICENSE').read_bytes()
js=(r/'fixtures/library/bip39-2.4.0/wordlists/english.js').read_text();words=js.split('`',2)[1].splitlines()
assert len(words)==2048 and '\n'.join(words)+'\n'==(r/'src/english.txt').read_text()
archive=(r/'fixtures/library/scure-bip39-2.4.0.tgz').read_bytes();sri='sha512-'+base64.b64encode(hashlib.sha512(archive).digest()).decode()
assert sri in (r/'fixtures/own/pnpm-lock.yaml').read_text()
for f in ['getrandom-0.2.17','chacha20poly1305-0.10.1','zeroize-1.9.0']:
 records=json.loads((r/'fixtures/library/cargo-pins.json').read_text());record=next(x for x in records if f==x['name']+'-'+x['version'])
 assert hashlib.sha256((r/'fixtures/library'/(f+'.crate')).read_bytes()).hexdigest()==record['checksum']
if (r/'INDEX.json').exists():
 index=json.loads((r/'INDEX.json').read_text())
 for item in index['files']:
  path=r/item['path'];data=path.read_bytes();assert len(data)==item['bytes'] and hashlib.sha256(data).hexdigest()==item['sha256'],item['path']
# Reviewable patches apply to their exact immutable source, in disposable dirs.
for patch,source in [('registration.patch','beta'),('icon-prerequisite-fe71ca8.patch','beta-current')]:
 with tempfile.TemporaryDirectory(prefix='password-patch-') as td:
  root=Path(td)
  for file in (r/'fixtures'/source).rglob('*'):
   if file.is_file():
    dest=root/file.relative_to(r/'fixtures'/source);dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(file.read_bytes())
  subprocess.run(['git','apply','--check',str(r/'integration'/patch)],cwd=td,check=True,capture_output=True)
print(f'Draft 2020-12 + {len(negative)} negatives, own metadata, original icon, dictionary/SRI, crate checksums, INDEX and both scoped patches PASS')
