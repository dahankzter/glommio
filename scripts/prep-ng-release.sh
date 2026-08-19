#!/usr/bin/env bash
# Prepare a "glommio-ng" stopgap release from the current tree.
#
# Applies packaging metadata only (crate name, lib name, readme path,
# description, repository/homepage), runs cargo package, and restores the
# tree afterwards. Nothing is committed; no code behaviour changes.
#
# Usage:
#   scripts/prep-ng-release.sh            # package + verify build
#   scripts/prep-ng-release.sh --dry-run  # also run cargo publish --dry-run
#   scripts/prep-ng-release.sh --publish  # package, verify, then publish

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

NG_NAME="glommio-ng"
NG_REPO="https://github.com/dahankzter/glommio"
NG_BLURB="Stopgap republish of the community glommio fork (github.com/glommio/glommio) with io_uring dependencies updated; will be deprecated if 0.10 lands under the canonical glommio name. "

MODE="${1:-package}"

restore() {
  git checkout -- glommio/Cargo.toml examples/Cargo.toml README.md 2>/dev/null || true
  rm -f glommio/README.md
}
trap restore EXIT

# --- glommio/Cargo.toml -------------------------------------------------
python3 - "$NG_NAME" "$NG_REPO" "$NG_BLURB" <<'PY'
import re, sys
name, repo, blurb = sys.argv[1:4]
p = "glommio/Cargo.toml"
s = open(p).read()

s = re.sub(r'(?m)^name = "glommio"$', f'name = "{name}"', s, count=1)
s = re.sub(r'(?m)^readme = .*$', 'readme = "README.md"', s, count=1)
s = re.sub(r'(?m)^repository = .*$', f'repository = "{repo}"', s, count=1)
s = re.sub(r'(?m)^homepage = .*$', f'homepage = "{repo}"', s, count=1)
s = re.sub(r'(?m)^description = "', f'description = "{blurb}', s, count=1)

# No [lib] rename: consumers get `use glommio::...` via cargo's own
# `package = "glommio-ng"` renaming, and a duplicate lib name would only make
# glommio and glommio-ng un-coexistable in one dependency graph.
open(p, "w").write(s)

# --- examples/Cargo.toml: path dep must follow the rename ---------------
p = "examples/Cargo.toml"
s = open(p).read()
s = s.replace('glommio        = { path = "../glommio" }',
              f'glommio        = {{ path = "../glommio", package = "{name}" }}')
open(p, "w").write(s)
PY

# --- README: packaged copy + stopgap notice -----------------------------
python3 - <<'PY'
notice = """> **Note — this is `glommio-ng`, a stopgap republish.**
>
> The canonical [`glommio`](https://crates.io/crates/glommio) crate last
> published 0.9.0 in March 2024, and 0.9.0 no longer compiles against current
> kernel headers. This crate exists so downstreams can depend on a working
> release instead of a git revision. It is published under a different crate
> name; cargo's dependency renaming keeps `use glommio::...` working, so only
> your `Cargo.toml` line differs:
>
> ```toml
> glommio = { package = "glommio-ng", version = "0.10" }
> ```
>
> `glommio-ng` and canonical `glommio` are **different crates whose types do not
> unify**: a library compiled against one cannot accept the other's executor,
> tasks or sockets, even where the source is identical. That is the cost of this
> fork existing. It will be deprecated in favour of the canonical name if 0.10
> can be published there.

"""
s = open("README.md").read()
open("glommio/README.md", "w").write(notice + s)
PY

echo "== cargo package =="
cargo package -p glommio-ng --allow-dirty

TARBALL="target/package/glommio-ng-0.10.0.crate"
echo
echo "== tarball =="
ls -lh "$TARBALL"
echo "files: $(tar tzf "$TARBALL" | wc -l)"

if [ "$MODE" = "--dry-run" ]; then
  echo "== cargo publish --dry-run =="
  cargo publish -p glommio-ng --allow-dirty --dry-run
elif [ "$MODE" = "--publish" ]; then
  echo "== cargo publish =="
  cargo publish -p glommio-ng --allow-dirty
fi
