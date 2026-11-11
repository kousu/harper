#!/bin/sh
# Build the French `dictionary.dict` from the Grammalecte lexicon.
#
# Usage: harper-core/dictionaries/fr/build.sh [-v VERSION] [-s SHA256] [-d DIR]
#   -v  Grammalecte lexicon version to download
#   -s  expected SHA-256 of the downloaded zip; pass -s '' to skip the check
#   -d  where to download and unpack the lexicon
#
# The lexicon is published by Grammalecte (https://grammalecte.net/) under MPL 2.0.
set -eu

while getopts v:s:d: opt; do
	case $opt in
	v) version=$OPTARG ;;
	s) sha256=$OPTARG ;;
	d) dir=$OPTARG ;;
	*) sed -n '2,8s/^# \{0,1\}//p' "$0" >&2; exit 2 ;;
	esac
done

: "${version:=7.7}"
: "${sha256=bdccf2065e252010253171000a38de84581407896b6bc60aca7d4d69dab7e4d2}"
: "${dir:=${TMPDIR:-/tmp}/grammalecte-lexique-fr-v$version}"

here=$(cd "$(dirname "$0")" && pwd)
zip=$dir/lexique-grammalecte-fr-v$version.zip
lexicon=$dir/lexique-grammalecte-fr-v$version.txt

mkdir -p "$dir"
[ -f "$zip" ] || curl -fsSL -o "$zip" "https://grammalecte.net/dic/lexique-grammalecte-fr-v$version.zip"
[ -z "$sha256" ] || echo "$sha256  $zip" | sha256sum -c -
[ -f "$lexicon" ] || unzip -q -o "$zip" -d "$dir"

LC_ALL=C.UTF-8 awk -f "$here/lexicon_to_dict.awk" "$lexicon" >"$here/dictionary.dict"

echo "Wrote $(tail -n +2 "$here/dictionary.dict" | wc -l) words from Grammalecte lexicon v$version."
