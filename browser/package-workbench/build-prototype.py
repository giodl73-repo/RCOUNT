"""Build the pinned native/WASM prototype and complete worktree source snapshot."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import zipfile
from controls import golden_requests, package_request

p=argparse.ArgumentParser()
p.add_argument('--bindgen',required=True,type=Path)
p.add_argument('--native-profile',choices=['dev','rcount-replay'],default='dev')
a=p.parse_args()
B=Path(__file__).resolve().parent
R=B.parents[1]
C=B/'.cache'; C.mkdir(exist_ok=True)
env=os.environ.copy()
target=Path(env.get('CARGO_TARGET_DIR',R/'target/cargo')).resolve()
env['CARGO_TARGET_DIR']=str(target)
env['CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS']='-C link-arg=--max-memory=67108864'
def run(argv): subprocess.run(list(map(str,argv)),cwd=R,env=env,check=True)
assert subprocess.check_output([str(a.bindgen.resolve()),'--version']).decode().strip()=='wasm-bindgen 0.2.127'
run(['python',R/'scripts/check-native-foundation.py'])
run(['python',B/'verify-shared-lock.py'])
profile=[] if a.native_profile=='dev' else ['--config','profile.rcount-replay.inherits="dev"','--profile','rcount-replay']
run(['cargo','+1.95.0','build','-p','rcount-cli','--locked',*profile])
for extra in [[],['--lib','--target','wasm32-unknown-unknown']]:
    run(['cargo','+1.95.0','build','--manifest-path',B/'Cargo.toml','--release','--locked',*extra])
suffix='.exe' if os.name=='nt' else ''
examples=C/'synthetic-examples'
if examples.exists():
    assert examples.resolve()==(B/'.cache/synthetic-examples').resolve()
    shutil.rmtree(examples)
run([target/'release'/('rcount-workbench-examples'+suffix),examples])
run(['python',B/'qualify-native.py','--binary',target/'release'/('rcount-workbench-control'+suffix),'--original',target/('debug' if a.native_profile=='dev' else a.native_profile)/('rcount'+suffix),'--examples',examples,'--output',C/'native-controls.json'])
run(['python',R/'scripts/check-original-cli.py',target/('debug' if a.native_profile=='dev' else a.native_profile)/('rcount'+suffix)])
site=R/'target/package-prototype/RCOUNT'
if site.exists():
    assert site.resolve().is_relative_to(R.resolve()/'target')
    shutil.rmtree(site)
site.mkdir(parents=True,exist_ok=True)
shutil.copytree(B/'web',site,dirs_exist_ok=True)
for name,request in golden_requests(R):
    path=site/'examples'/f'{name}.json'; path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text(json.dumps(request,separators=(',',':'))+'\n',encoding='utf-8',newline='\n')
for folder in sorted(examples.iterdir()):
    path=site/'examples/synthetic'/f'{folder.name}.json'; path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text(json.dumps(package_request(folder),separators=(',',':'))+'\n',encoding='utf-8',newline='\n')
for binding,output in [('web',site/'pkg'),('nodejs',C/'node')]:
    run([a.bindgen.resolve(),target/'wasm32-unknown-unknown/release/rcount_package_workbench.wasm','--target',binding,'--out-dir',output])
    for path in output.glob('*.d.ts'): path.unlink()
    (output/'package.json').write_text(json.dumps({'type':'module' if binding=='web' else 'commonjs'})+'\n',encoding='utf-8',newline='\n')
run(['node',B/'qualify-wasm.cjs',C/'native-controls.json',C/'wasm-parity.json'])
# Includes original full native workspace and evidence, not just the adapter.
record_path=R/'.source-record.json'
if record_path.exists():
    record=json.loads(record_path.read_text(encoding='utf-8'))
    paths=[file['path'] for file in record['files']]
    for file in record['files']: assert hashlib.sha256((R/file['path']).read_bytes()).hexdigest()==file['sha256'],file['path']
    commit=record['commit'];dirty=False
else:
    assert Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],cwd=R).decode().strip()).resolve()==R.resolve()
    paths=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=R).decode('utf-8').split('\0')
    commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=R).decode().strip()
    dirty=bool(subprocess.check_output(['git','status','--porcelain'],cwd=R))
with zipfile.ZipFile(site/'source.zip','w',zipfile.ZIP_DEFLATED) as archive:
    for name in sorted(filter(None,paths)):
        path=R/name
        if path.is_file(): archive.write(path,'RCOUNT/'+name)
wasm=site/'pkg/rcount_package_workbench_bg.wasm'
def leb(data,index):
    value=shift=0
    while True:
        byte=data[index];index+=1;value|=(byte&127)<<shift
        if not byte&128:return value,index
        shift+=7
data=wasm.read_bytes();index=8;memories=[]
while index<len(data):
    section=data[index];index+=1;length,index=leb(data,index);end=index+length
    if section==5:
        count,index=leb(data,index)
        for _ in range(count):
            flags,index=leb(data,index);minimum,index=leb(data,index)
            maximum,index=leb(data,index) if flags&1 else (None,index)
            memories.append((flags,minimum,maximum))
    index=end
assert len(memories)==1 and memories[0][0]==1 and memories[0][2]==1024,memories
provenance=dict(commit=commit,dirty=dirty,
    rust='1.95.0',wasm_bindgen='0.2.127',wasm_sha256=hashlib.sha256(wasm.read_bytes()).hexdigest(),
    wasm_bytes=wasm.stat().st_size,source_sha256=hashlib.sha256((site/'source.zip').read_bytes()).hexdigest(),
    maximum_memory_bytes=67_108_864,status='prototype; hosted release qualification incomplete')
(site/'build.json').write_text(json.dumps(provenance,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(site=str(site),**provenance)))
