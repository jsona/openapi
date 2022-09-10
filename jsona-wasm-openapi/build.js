const fs = require("fs");
const p = v => require("path").resolve(__dirname, v);
const $ = v => require("child_process").execSync(v, { stdio: "inherit" });

$(`wasm-pack build --out-name index`);
[
  "index.d.ts",
  "index_web.d.ts",
  "package.json",
  "README.md",
].forEach(name => {
  fs.copyFileSync(p(name), p("pkg/" + name));
});
$(`npx wasm-pack-utils node ${p("pkg/index_bg.js")} -o ${p("pkg/index.js")}`);
$(`npx wasm-pack-utils web ${p("pkg/index_bg.js")} -o ${p("pkg/index_web.js")}`);
$(`node test.js`);