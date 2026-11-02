"""Rebuild the exact delivered source outside Git with isolated test profiles."""
import argparse,hashlib,json,os,re,subprocess,tempfile,zipfile
from pathlib import Path

p=argparse.ArgumentParser();p.add_argument('--bindgen',required=True,type=Path);a=p.parse_args()
B=Path(__file__).resolve().parent;R=B.parents[1];C=B/'.cache'
expected=json.loads((C/'native-controls.json').read_text(encoding='utf-8'))
env=os.environ.copy();env.setdefault('CARGO_TARGET_DIR',str(R/'target/cargo'));env.pop('RCOUNT_URL',None)
profile=['--config','profile.rcount-replay.inherits="dev"','--profile','rcount-replay']

with tempfile.TemporaryDirectory(prefix='rcount delivered source ') as tmp:
    tmp=Path(tmp).resolve();assert tmp.parent==Path(tempfile.gettempdir()).resolve()
    with zipfile.ZipFile(R/'target/package-release/RCOUNT/source.zip') as archive:
        for member in archive.namelist():
            assert not Path(member).is_absolute() and '..' not in Path(member).parts and '\\' not in member,member
        archive.extractall(tmp)
    root=tmp/'RCOUNT-source';record=json.loads((root/'.source-record.json').read_text(encoding='utf-8'))
    assert not list(root.rglob('.git'))
    for file in record['files']:assert hashlib.sha256((root/file['path']).read_bytes()).hexdigest()==file['sha256'],file['path']
    for file in root.rglob('*.rs'):file.touch()
    def run(argv,cwd=root,log=None):
        if log:
            result=subprocess.run(list(map(str,argv)),cwd=cwd,env=env,capture_output=True)
            (C/log).write_bytes(result.stdout+result.stderr)
            assert result.returncode==0,f'Extracted source command failed; see {C/log}'
            return result.stdout.decode('utf-8',errors='replace')
        subprocess.run(list(map(str,argv)),cwd=cwd,env=env,check=True)
    native=run(['cargo','+1.95.0','test','--workspace','--locked',*profile],log='source-native.log')
    assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed;',native)))==279
    adapter=run(['cargo','+1.95.0','test','--manifest-path','browser/package-workbench/Cargo.toml','--all-targets','--locked',*profile],log='source-adapter.log')
    assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed;',adapter)))==12
    run(['python',root/'browser/package-workbench/build-prototype.py','--bindgen',a.bindgen.resolve(),'--native-profile','rcount-replay'],log='source-build.log')
    companion=root/'browser/package-workbench';cache=companion/'.cache'
    actual=json.loads((cache/'native-controls.json').read_text(encoding='utf-8'))
    assert [(r['name'],r['native']) for r in actual]==[(r['name'],r['native']) for r in expected],'Complete native report drift from delivered source'
    assert json.loads((cache/'wasm-parity.json').read_text(encoding='utf-8'))=={'cases':57,'failures':[]}
    npm='npm.cmd' if os.name=='nt' else 'npm';run([npm,'ci','--ignore-scripts'],cwd=companion,log='source-npm.log')
    browser=run(['node','node_modules/@playwright/test/cli.js','test','workbench.spec.js','--reporter=json'],cwd=companion,log='source-browser.log')
    browser=json.loads(browser);stats=browser['stats']
    assert stats['expected']==17 and not any(stats.get(k,0) for k in ['unexpected','flaky','skipped']),stats
    (C/'source-replay.json').write_text(json.dumps(dict(commit=record['commit'],source_files=len(record['files']),
        root_native_tests=279,adapter_tests=12,original_cli_controls=84,complete_native_wasm_reports=57,browser_checks=17,
        cargo_profile='rcount-replay',temporary_root_outside_git=str(root),binary_identity_claim=False,
        input_byte_identity_claim=False,scope='Exact committed source bytes; full output parity. Windows checkout line endings can differ from canonical delivered Git text, while raw source evidence stays exact.'),indent=2)+'\n',encoding='utf-8',newline='\n')
