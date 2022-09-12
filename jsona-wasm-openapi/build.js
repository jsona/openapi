const fs = require("fs");
const p = v => require("path").resolve(__dirname, v);
const $ = v => require("child_process").execSync(v, { stdio: "inherit" });

$(`wasm-pkg-build --out-name index`);
[
  "index.d.ts",
  "index_web.d.ts",
  "package.json",
  "README.md",
].forEach(name => {
  fs.copyFileSync(p(name), p("pkg/" + name));
});
$(`node test.js`);