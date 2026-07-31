import { ESLint } from "eslint";
import { fileURLToPath } from "node:url";
import path from "node:path";

const desktop = fileURLToPath(new URL("..", import.meta.url));
const repository = path.resolve(desktop, "..", "..");
const config = path.join(desktop, "eslint.config.js");
const scopes = [
  {
    directory: desktop,
    files: ["src", "scripts", "vite.config.ts", "eslint.config.js"],
  },
  {
    directory: path.join(repository, "packages", "ui"),
    files: ["src"],
  },
  {
    directory: path.join(repository, "packages", "design-tokens"),
    files: ["generate.mjs", "src"],
  },
];

let errors = 0;
let warnings = 0;

for (const scope of scopes) {
  const eslint = new ESLint({
    cwd: scope.directory,
    overrideConfigFile: config,
  });
  const results = await eslint.lintFiles(scope.files);
  const formatter = await eslint.loadFormatter("stylish");
  const report = await formatter.format(results);

  if (report.length > 0) {
    process.stdout.write(report);
  }
  errors += results.reduce((total, result) => total + result.errorCount, 0);
  warnings += results.reduce((total, result) => total + result.warningCount, 0);
}

if (errors > 0 || warnings > 0) {
  process.exitCode = 1;
}
