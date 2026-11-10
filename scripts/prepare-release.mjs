import { readFileSync } from "node:fs";

const { version } = JSON.parse(readFileSync("package.json", "utf8"));
for (const file of ["Cargo.toml", "Cargo.lock"]) {
  const rustVersion = readFileSync(file, "utf8").match(/name = "pace"\nversion = "([^"]+)"/)?.[1];
  if (version !== rustVersion) {
    throw new Error(`Release version mismatch: package.json is ${version}, ${file} is ${rustVersion}`);
  }
}
const changelog = readFileSync("CHANGELOG.md", "utf8").replace(/\r\n/g, "\n");
const section = changelog.split(/^## /m).find((entry) => entry.startsWith(`${version}\n`));
const notes = section?.split("\n").slice(1).join("\n").trim();
if (!notes) throw new Error(`Missing or empty CHANGELOG.md entry for ${version}`);
process.stdout.write(`${notes}\n`);
