# Changesets

For a user-facing change, run `npm ci` and `npm run changeset` from the repository
root. Select `pace`, choose a patch, minor, or major bump, and describe the change
for users. Commit the generated Markdown file with the change. Use
`npm run changeset -- --empty` for changes that need no release note.

Changesets manages this Rust app through a private `package.json`; nothing is
published to npm. Versions in `package.json`, `Cargo.toml`, and `Cargo.lock` stay
in sync through `npm run changeset:version`.

On pushes to `main`, the Release workflow creates or updates a `chore: release`
PR on `changeset-release/main`. Review its versions and changelog, then merge it
to publish a GitHub release with Linux x86-64 and macOS Apple Silicon binaries.
Release notes come from that version's `CHANGELOG.md` entry. Both platforms must
pass tests and terminal smoke checks before publishing. The release tag points
to the release PR's merge commit.

## Repository setup

Add an Actions secret named `CHANGESETS_TOKEN`, as in FirstRecall. Use a token
with access to this repository and **Contents: read and write** and
**Pull requests: read and write** permissions. A personal access token lets
the generated release PR trigger CI. The publishing workflow uses GitHub's
built-in token and needs no npm credentials.

## Local verification

Use Node.js 24 and npm 10.9 or newer:

```sh
npm ci
npm test
npm run changeset -- status
```

To preview a release, run `npm run changeset:version` on a disposable branch or
copy, then check the three version files and `CHANGELOG.md`. This consumes
pending changesets. `node scripts/prepare-release.mjs` prints the release notes
and rejects missing notes or inconsistent Rust versions.
