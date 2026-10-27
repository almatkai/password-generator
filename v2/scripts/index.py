"""Rebuild a byte index of every deliverable, never .work/target/artifacts."""
from pathlib import Path
import json,hashlib
r=Path(__file__).resolve().parents[1]
files=[]
for p in sorted(r.rglob('*')):
 rel=p.relative_to(r)
 if not p.is_file() or rel.parts[0] in ['target','.work','artifacts'] or rel.as_posix()=='INDEX.json':continue
 data=p.read_bytes();files.append({'path':rel.as_posix(),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
(r/'INDEX.json').write_text(json.dumps({'algorithm':'sha256','excludes':['INDEX.json','.work/','target/','artifacts/'],'files':files},indent=2)+'\n')
print('Indexed',len(files),'deliverable files')
