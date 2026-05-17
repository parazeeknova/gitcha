#!/usr/bin/env bash
set -euo pipefail

ROW() {
  printf '\033[48;2;0;255;255;38;2;0;0;0m  %-70s  \033[0m\n' "$1"
}

version="$(sed -nE 's/^version = "([0-9]+\.[0-9]+\.[0-9]+)"/\1/p' Cargo.toml | head -1)"
if [ -z "$version" ]; then
  ROW "error: could not read version from Cargo.toml"
  exit 1
fi

ROW "reading version ${version}"

tag="v${version}"

ROW "checking for dirty working tree"
git diff --quiet || {
  ROW "error: working tree is dirty; commit changes before tagging ${tag}"
  exit 1
}

ROW "creating tag ${tag}"
git tag -a "$tag" -m "Release ${tag}"
ROW "created ${tag}; push with: git push origin ${tag}"
