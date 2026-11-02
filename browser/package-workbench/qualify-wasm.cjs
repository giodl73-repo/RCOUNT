// Node executes the actual compiled Rust WASM. Browser worker checks are separate.
const fs = require('node:fs');
const engine = require('./.cache/node/rcount_package_workbench.js');
const cases = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
const failures = [];
for (const test of cases) {
  const actual = engine.evaluate(test.request);
  if (actual !== test.native) {
    failures.push({name: test.name, native: test.native, wasm: actual});
  }
}
fs.writeFileSync(process.argv[3], JSON.stringify({cases: cases.length, failures}, null, 2) + '\n');
console.log(JSON.stringify({cases: cases.length, mismatches: failures.map(f => f.name)}));
if (failures.length) process.exit(1);
