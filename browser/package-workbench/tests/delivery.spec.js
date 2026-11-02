import {test,expect} from '@playwright/test';
import crypto from 'node:crypto';
const base=process.env.RCOUNT_URL??'http://127.0.0.1:8775/RCOUNT/';
const sha=data=>crypto.createHash('sha256').update(data).digest('hex');
test('every packaged runtime/example/source/notice file matches its custody hash',async({request})=>{
  const inventory=await(await request.get(new URL('inventory.json',base).href)).json();
  let bytes=0;for(const file of inventory.files){const response=await request.get(new URL(file.path,base).href);expect(response.status(),file.path).toBe(200);const body=await response.body();expect(body.length,file.path).toBe(file.bytes);expect(sha(body),file.path).toBe(file.sha256);bytes+=body.length;}
  expect(bytes).toBe(inventory.bytes_excluding_inventory);
  const build=await(await request.get(new URL('build.json',base).href)).json();expect(build.commit).toBe(inventory.commit);expect(build.actual_wasm_memory_max_bytes).toBe(67108864);expect(build.fixture_count).toBe(57);
  expect(sha(await(await request.get(new URL('source.zip',base).href)).body())).toBe(build.source_zip_sha256);
});
test('all original dependency and runtime notices remain independently reachable',async({request,page})=>{
  await page.route('**/worker.js',route=>route.abort());await page.goto(base);
  for(const name of ['Compiled WASM','Source snapshot ZIP','Build provenance','Original software & dependency notices','Source scope','Semantics & limits'])await expect(page.getByRole('link',{name,exact:true})).toBeVisible();
  const groups=await(await request.get(new URL('dependencies.json',base).href)).json();expect(groups.length).toBeGreaterThan(20);
  for(const group of groups){expect(group.notices.length).toBeGreaterThan(0);for(const notice of group.notices){const response=await request.get(new URL(notice.path,base).href);expect(response.status()).toBe(200);expect(sha(await response.body())).toBe(notice.sha256);}}
  const runtime=await(await request.get(new URL('rust/provenance.json',base).href)).json();expect(runtime.notices.length).toBeGreaterThan(0);
});
