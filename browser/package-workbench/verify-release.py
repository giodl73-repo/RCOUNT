"""Run browser task/recovery/delivery checks against the exact packaged site."""
import os,socket,subprocess,sys,time
from pathlib import Path
B=Path(__file__).resolve().parent;R=B.parents[1]
with socket.socket() as listener:listener.bind(('127.0.0.1',0));port=listener.getsockname()[1]
server=subprocess.Popen([sys.executable,'-m','http.server',str(port),'--directory',str(R/'target/package-release'),'--bind','127.0.0.1'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
try:
    for _ in range(100):
        try:
            with socket.create_connection(('127.0.0.1',port),timeout=.1):break
        except OSError:time.sleep(.1)
    else:raise RuntimeError('Packaged test server did not start')
    env=os.environ.copy();env['RCOUNT_URL']=f'http://127.0.0.1:{port}/RCOUNT/'
    subprocess.run(['npm.cmd' if os.name=='nt' else 'npm','run','test:release'],cwd=B,env=env,check=True)
finally:server.terminate();server.wait(timeout=10)
