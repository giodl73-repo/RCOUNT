const $=id=>document.getElementById(id);
let worker,ready=false,busy=false,job=0,selection=0,timer,request=null,reportRaw='',verificationRaw='',replayRaw='';
function update(){ $('run').disabled=!ready||busy||!request;$('cancel').disabled=!busy;
  $('save-bundle').disabled=!request;$('original-file').disabled=!request;$('save-original').disabled=!request;
  $('save-report').disabled=!reportRaw;$('save-verification').disabled=!verificationRaw;$('save-replay').disabled=!replayRaw; }
function clearReport(message='Package changed. Verify and replay to refresh findings.'){
  reportRaw=verificationRaw=replayRaw='';$('summary').replaceChildren();$('checks').replaceChildren();$('replays').replaceChildren();$('raw').textContent='No report yet.';$('result-status').textContent=message;update();
}
function stopWork(message){job++;clearTimeout(timer);busy=false;ready=false;worker?.terminate();$('engine-status').textContent=message;update();}
function fail(message){stopWork('Rust engine unavailable. Restart to retry.');$('result-status').textContent=message;}
function restart(){stopWork('Loading Rust engine…');const current=new Worker(new URL('./worker.js',import.meta.url),{type:'module'});worker=current;
  timer=setTimeout(()=>{if(worker===current)fail('Engine startup timed out. Source and download links remain available.');},15000);
  current.onerror=()=>{if(worker===current)fail('The Rust worker could not run. Restart the engine or use the native CLI.');};
  current.onmessage=({data})=>{if(worker!==current)return;
    if(data.type==='ready'){clearTimeout(timer);ready=true;$('engine-status').textContent='Rust engine ready · work stays in your browser';update();return;}
    if(data.type==='failed'){fail(data.message||'Rust worker failed.');return;}
    if(data.type==='result'&&data.id===job){clearTimeout(timer);busy=false;render(data.report);update();}
  };
}
function text(tag,content,className){const el=document.createElement(tag);el.textContent=content;if(className)el.className=className;return el;}
function badge(status){return text('span',status==='boundary'?'BOUNDARY · NOT REPLAYED':status.toUpperCase(),`badge ${status}`);}
function render(raw){let report;try{report=JSON.parse(raw);}catch{fail('Rust returned an unreadable report.');return;}
  clearReport();reportRaw=raw;$('raw').textContent=raw;
  if(report.kind!=='report'){$('result-status').textContent=`${report.kind==='refused'?'Browser limit':'Input error'}: ${report.message}`;update();return;}
  verificationRaw=report.verification_json;replayRaw=report.replay.transcripts_json||'';
  const v=report.verification;$('summary').append(badge(v.status),text('span',`${v.checks.length} verification checks reported`));
  for(const check of v.checks){const item=text('li','',check.status);item.append(badge(check.status),text('strong',check.equation_id));
    if(check.contest_id||check.reporting_unit_id)item.append(text('p',[check.contest_id,check.reporting_unit_id].filter(Boolean).join(' · ')));
    if(check.error)item.append(text('p',check.error));$('checks').append(item);}
  let boundaries=0,failures=0;
  if(report.replay.kind==='loaded'){
    if(!report.replay.transcripts.length)$('replays').append(text('p','No audit algorithm runs declared in this package. Seeded ballot-sample replay, when declared, is included in the verification checks above.'));
    for(const replay of report.replay.transcripts){if(replay.status==='boundary')boundaries++;if(replay.status==='fail')failures++;
      const card=text('article','','replay');card.append(badge(replay.status),text('h3',replay.method_id),text('p',replay.run_id));
      if(replay.boundary)card.append(text('p',replay.boundary));
      card.append(text('p',`Declared decision: ${replay.decision}. Computed decision: ${replay.computed_decision??'not available'}.`));$('replays').append(card);}
  }else $('replays').append(text('p',`Audit replay unavailable: ${report.replay.message}`));
  $('result-status').textContent=`Package consistency: ${v.status}. Replay failures: ${failures}; boundaries: ${boundaries}. These findings do not certify an election.`;
  update();$('result-title').focus();
}
function beginSelection(){selection++;if(busy){stopWork('Cancelling previous computation…');restart();}request=null;clearReport();$('package-status').textContent='Opening package…';$('original-file').replaceChildren();update();return selection;}
function setPackage(value,label,token){if(token!==selection)return;
  if(!Array.isArray(value?.files)||value.files.length<1||value.files.length>64)throw new Error('Choose 1–64 package files.');
  let total=0;for(const file of value.files){if(typeof file.path!=='string'||!Array.isArray(file.bytes)||file.bytes.length>131072||file.bytes.some(b=>!Number.isInteger(b)||b<0||b>255))throw new Error('Invalid file bytes or file exceeds 128 KiB.');total+=file.bytes.length;}
  if(total>524288)throw new Error('Package exceeds 512 KiB.');
  request=value;$('package-status').textContent=`${label}: ${value.files.length} files, ${total.toLocaleString()} original bytes. Ready to verify.`;
  for(const [index,file]of value.files.entries()){const option=text('option',file.path);option.value=String(index);$('original-file').append(option);}update();
}
function selectionError(error,token){if(token!==selection)return;request=null;$('package-status').textContent=`Could not open package: ${error.message}`;update();}
$('load-example').onclick=async()=>{const token=beginSelection();try{const selected=$('example').value;const response=await fetch(`examples/${selected}.json`,{cache:'no-store'});if(!response.ok)throw new Error('Example could not be downloaded.');const raw=await response.text();if(raw.length>4194304)throw new Error('Example bundle exceeds 4 MiB.');setPackage(JSON.parse(raw),`Synthetic example · ${selected}`,token);}catch(error){selectionError(error,token);}};
$('folder').onchange=async event=>{const files=[...event.target.files];event.target.value='';if(!files.length)return;const token=beginSelection();try{
  if(files.length>64||files.some(f=>f.size>131072)||files.reduce((n,f)=>n+f.size,0)>524288)throw new Error('Limits: 64 files, 128 KiB per file, 512 KiB total.');
  const opened=[];for(const file of files){const path=file.webkitRelativePath?.split('/').slice(1).join('/')||file.name;opened.push({path,bytes:[...new Uint8Array(await file.arrayBuffer())]});}
  setPackage({files:opened},'Local package',token);}catch(error){selectionError(error,token);}};
$('bundle').onchange=async event=>{const file=event.target.files[0];event.target.value='';if(!file)return;const token=beginSelection();try{if(file.size>4194304)throw new Error('Bundle exceeds 4 MiB.');setPackage(JSON.parse(await file.text()),file.name,token);}catch(error){selectionError(error,token);}};
$('run').onclick=()=>{if(!ready||busy||!request)return;clearReport('Running native Rust verification and audit replay…');busy=true;const id=++job;update();timer=setTimeout(()=>{if(id===job)fail('Computation timed out. Inputs are retained; restart the engine or use the native CLI.');},10000);worker.postMessage({id,request:JSON.stringify(request)});};
$('cancel').onclick=()=>{stopWork('Computation cancelled. Inputs retained.');clearReport('Computation cancelled. No findings from the cancelled run are shown.');restart();};
$('restart').onclick=()=>{clearReport('Engine restarted. Verify and replay to refresh findings.');restart();};
function download(bytes,name,type='application/json'){const url=URL.createObjectURL(new Blob([bytes],{type}));const a=document.createElement('a');a.href=url;a.download=name;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
$('save-bundle').onclick=()=>request&&download(JSON.stringify(request),'rcount-package-bundle.json');
$('save-original').onclick=()=>{const file=request?.files[Number($('original-file').value)];if(file)download(new Uint8Array(file.bytes),file.path.split('/').at(-1),'application/octet-stream');};
$('save-report').onclick=()=>reportRaw&&download(reportRaw,'rcount-workbench-report.json');
$('save-verification').onclick=()=>verificationRaw&&download(verificationRaw,'verify-transcript.json');
$('save-replay').onclick=()=>replayRaw&&download(replayRaw,'audit-replay-transcripts.json');
restart();update();
