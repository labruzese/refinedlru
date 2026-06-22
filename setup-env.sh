#!/usr/bin/env bash
#
# Sets up the RefinedRust + Rocq environment for this workspace. 
# Run it from the repo root or set REPO_ROOT below.
#
# It does NOT install anything, it activates an existing opam switch and
# exports the variables RefinedRust needs. 

# Run the install scripts in refinedrust-dev/scripts/ once before using this.

# config
SWITCH="${RR_SWITCH:-refinedrust}"
# Resolve repo root: dir of this script when sourced, else pwd.
if [[ -n "${BASH_SOURCE[0]:-}" ]]; then
  REPO_ROOT="${REPO_ROOT:-$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )}"
else
  REPO_ROOT="${REPO_ROOT:-$PWD}"
fi
CRATE_DIR="${RR_CRATE:-$REPO_ROOT/lru-rs}"
RR_DIR="${REFINEDRUST_ROOT:-$REPO_ROOT/refinedrust-dev}"

# opam switch
if ! command -v opam >/dev/null 2>&1; then
  echo "rr-env: opam not found on PATH." >&2
  return 1 2>/dev/null || exit 1
fi
if ! opam switch list --short 2>/dev/null | grep -qx "$SWITCH"; then
  echo "rr-env: opam switch '$SWITCH' does not exist." >&2
  echo "        Create it via the scripts in refinedrust-dev (setup-coq.sh etc.)." >&2
  return 1 2>/dev/null || exit 1
fi
eval "$(opam env --switch="$SWITCH" --set-switch)"

# RefinedRust frontend
export REFINEDRUST_ROOT="$RR_DIR"
# Use the crate's own RefinedRust.toml if present; else the toolchain default.
if [[ -f "$CRATE_DIR/RefinedRust.toml" ]]; then
  export RR_CONFIG="$CRATE_DIR/RefinedRust.toml"
fi
export PATH="$HOME/.cargo/bin:$PATH"          # cargo-refinedrust / refinedrust-rustc

# summary
echo "switch            : $SWITCH"
echo "REFINEDRUST_ROOT  : $REFINEDRUST_ROOT"
echo "RR_CONFIG         : ${RR_CONFIG:-<toolchain default>}"
echo "crate             : $CRATE_DIR"
echo "cargo-refinedrust : $(command -v cargo-refinedrust || echo MISSING)"
echo "coqc              : $(command -v coqc || echo MISSING)"
echo "coq-lsp           : $(command -v coq-lsp || echo 'not installed')"