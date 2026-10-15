#!/bin/sh
# Shell integration: after a mistyped command, `hf -y` must run the fixed one, and a second call must
# find nothing to fix. Runs each shell interactively in a pseudo-terminal (util-linux `script`).
set -u
export LANG=C.UTF-8 LC_ALL=C.UTF-8
fail=0
for sh in bash zsh fish; do
    command -v "$sh" >/dev/null 2>&1 || { echo "skip $sh (not installed)"; continue; }
    case $sh in
        bash) init='eval "$(hanfix --init bash)"'; run='bash --norc -i' ;;
        zsh) init='eval "$(hanfix --init zsh)"'; run='zsh -f -i' ;;
        fish) init='hanfix --init fish | source'; run='fish -N -i' ;;
    esac
    home=$(mktemp -d)
    out=$(printf '%s\nㄷ초ㅐ hanfix-ok\nhf -y\nㅗㄹ -y\nexit\n' "$init" |
        HOME=$home timeout 20 script -qec "$run" /dev/null 2>&1 | tr -d '\r')
    if printf '%s\n' "$out" | grep -qx 'hanfix-ok' && printf '%s\n' "$out" | grep -q 'nothing to fix'; then
        echo "ok   $sh"
    else
        echo "FAIL $sh"; printf '%s\n' "$out" | sed 's/^/    /'; fail=1
    fi
done
exit $fail
