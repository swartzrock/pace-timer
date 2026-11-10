import { readFileSync, writeFileSync } from "node:fs";

const { version } = JSON.parse(readFileSync("package.json", "utf8"));
const packageVersion = /(name = "pace"\nversion = ")[^"]+("\n)/;

for (const file of ["Cargo.toml", "Cargo.lock"]) {
  const source = readFileSync(file, "utf8");
  if (!packageVersion.test(source)) {
    throw new Error(`Missing pace package version in ${file}`);
  }
  writeFileSync(file, source.replace(packageVersion, (_, before, after) => `${before}${version}${after}`));
}
