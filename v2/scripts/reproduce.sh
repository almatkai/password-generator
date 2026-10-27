#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p .work/v1 .work/ui
cp -R fixtures/own/. .work/v1/
# Use immutable own lock; pnpm 11's build-script policy is bypassed only by
# explicitly disabling scripts in this disposable test installation.
pnpm --dir .work/v1 install --frozen-lockfile --ignore-scripts
(cd .work/v1 && node --import ./node_modules/tsx/dist/loader.mjs --test test/*.test.ts && node node_modules/typescript/bin/tsc --noEmit)
node scripts/oracle.cjs
cp fixtures/ui/package.json fixtures/ui/package-lock.json .work/ui/
npm ci --prefix .work/ui --ignore-scripts --no-audit --no-fund
node scripts/ui.cjs
for toolchain in stable 1.85.0; do
  cargo "+$toolchain" test --locked --offline
  cargo "+$toolchain" clippy --locked --offline --all-targets -- -D warnings
  cargo "+$toolchain" fmt --all -- --check --config skip_children=true
  rustup run "$toolchain" rustfmt --edition 2021 --check --config skip_children=true src/*.rs tests/harness/*.rs
  cargo "+$toolchain" check --locked --offline --all-targets --target x86_64-pc-windows-gnu
  cargo "+$toolchain" clippy --locked --offline --all-targets --target x86_64-pc-windows-gnu -- -D warnings
done
# Dependency installation is explicit and disposable; use this venv for schema QA.
python3 -m venv .work/python
.work/python/bin/pip install 'jsonschema==4.23.0' --disable-pip-version-check -q
.work/python/bin/python scripts/validate.py
