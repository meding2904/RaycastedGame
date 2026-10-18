#!/usr/bin/env bash
# Downloads CC0 art packs into assets/cc0/ so they can be used instead of the generated art.
#
# Nothing in the game depends on this script: without these files the procedural generators run and
# the game looks complete. This is for people who want to drop real art in front of them.
#
#   tools/fetch_assets.sh                  download every pack in tools/cc0-manifest.txt
#   tools/fetch_assets.sh --list           the names the game looks for in assets/cc0/
#   tools/fetch_assets.sh --install FILE NAME
#                                          copy FILE to assets/cc0/NAME.png (a 64x64 PNG)
#   tools/fetch_assets.sh --clean          remove the unpacked packs, keep installed images
#
# A failed download is a warning, never a broken checkout: the script keeps going and tells you
# which pack it could not get.

set -u

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest="$root/tools/cc0-manifest.txt"
assets="$root/assets/cc0"
raw="$assets/raw"

die() {
    echo "error: $*" >&2
    exit 1
}

need() {
    command -v "$1" >/dev/null 2>&1 || die "this script needs '$1' on PATH"
}

usage() {
    sed -n '2,16p' "$0" | sed 's/^# \{0,1\}//'
}

list_names() {
    local binary="$root/target/release/frames"
    [ -x "$binary" ] || binary="$root/target/debug/frames"
    if [ -x "$binary" ]; then
        "$binary" --names
    else
        echo "build the harness first, it owns the list: cargo build -p wolf3d --bin frames" >&2
        echo "then run: $0 --list" >&2
        exit 1
    fi
}

install_file() {
    local source="$1" name="$2"
    [ -f "$source" ] || die "no such file: $source"
    [ -n "$name" ] || die "usage: $0 --install FILE NAME"
    need file

    # The loader only reads PNG, and only this game's 64x64 cells fit the atlas
    if [ "$(head -c 8 "$source" | od -An -tx1 | tr -d ' \n')" != "89504e470d0a1a0a" ]; then
        die "$source is not a PNG (convert it first: png/magick/ffmpeg)"
    fi

    mkdir -p "$assets"
    cp "$source" "$assets/$name.png"
    echo "installed assets/cc0/$name.png"
    echo "remove it to go back to the generated art."
}

fetch() {
    need curl
    need sha256sum
    [ -f "$manifest" ] || die "no manifest at $manifest"

    mkdir -p "$raw"
    local failed=0 count=0

    while read -r folder url hash license _rest; do
        case "$folder" in
            ''|'#'*) continue ;;
        esac
        count=$((count + 1))

        if [ -d "$raw/$folder" ]; then
            echo "have    $folder"
            continue
        fi

        local zip
        zip="$(mktemp "${TMPDIR:-/tmp}/cc0-XXXXXX.zip")"
        if ! curl -sSL --fail --max-time 120 -o "$zip" "$url"; then
            echo "warning: could not download $folder from $url" >&2
            rm -f "$zip"
            failed=$((failed + 1))
            continue
        fi

        local actual
        actual="$(sha256sum "$zip" | cut -d' ' -f1)"
        if [ "$actual" != "$hash" ]; then
            echo "warning: $folder changed upstream (sha256 $actual, expected $hash), skipping" >&2
            rm -f "$zip"
            failed=$((failed + 1))
            continue
        fi

        if command -v unzip >/dev/null 2>&1; then
            unzip -q -o "$zip" -d "$raw/$folder" || {
                echo "warning: could not unpack $folder" >&2
                failed=$((failed + 1))
            }
        else
            echo "warning: no unzip on PATH, left $folder as $raw/$folder.zip" >&2
            mkdir -p "$raw/$folder"
            mv "$zip" "$raw/$folder/pack.zip"
        fi
        rm -f "$zip"
        echo "fetched $folder ($license)"
    done <"$manifest"

    echo
    echo "$count pack(s) in the manifest, $failed could not be fetched."
    echo "Packs land in assets/cc0/raw/. To use one:"
    echo "  $0 --install assets/cc0/raw/<pack>/<file>.png <name>"
    echo "  $0 --list   # the names the game looks for"
    [ "$failed" -eq 0 ]
}

main() {
    case "${1:-}" in
        --help|-h) usage ;;
        --list) list_names ;;
        --install) shift; install_file "${1:-}" "${2:-}" ;;
        --clean) rm -rf "$raw"; echo "removed assets/cc0/raw" ;;
        '') fetch ;;
        *) usage; exit 1 ;;
    esac
}

main "$@"
