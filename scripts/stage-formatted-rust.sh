#!/usr/bin/env bash
set -euo pipefail

ROW() {
  printf '\033[48;2;0;255;255;38;2;0;0;0m  %-70s  \033[0m\n' "$1"
}

ROW "staging formatted rust files"
git diff --name-only -- '*.rs' | while IFS= read -r file; do
  [ -n "$file" ] && git add "$file"
done
ROW "done"
