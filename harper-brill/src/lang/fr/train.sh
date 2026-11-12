#!/bin/sh
# Train the French tagger and chunker models in this folder.
#
# Usage: harper-brill/src/lang/fr/train.sh [-e EPOCHS] [-c CHANCE] [-r RELEASE] [-d DIR]
#   -e  number of epochs (patch rules) to train
#   -c  chance of trying each candidate patch per epoch, from 0 to 1;
#       lower is faster but may find slightly worse patches
#   -r  UD_French-GSD release tag to train on
#   -d  where to clone the treebank
#
# The treebank is UD_French-GSD (CC BY-SA 4.0):
# https://github.com/UniversalDependencies/UD_French-GSD
set -eu

while getopts e:c:r:d: opt; do
	case $opt in
	e) epochs=$OPTARG ;;
	c) chance=$OPTARG ;;
	r) release=$OPTARG ;;
	d) dir=$OPTARG ;;
	*) sed -n '2,10s/^# \{0,1\}//p' "$0" >&2; exit 2 ;;
	esac
done

: "${epochs:=100}"
: "${chance:=0.02}"
: "${release:=r2.19}"
: "${dir:=${TMPDIR:-/tmp}/UD_French-GSD-$release}"

here=$(cd "$(dirname "$0")" && pwd)
root=$(git -C "$here" rev-parse --show-toplevel)

[ -d "$dir" ] || git clone --quiet --depth 1 --branch "$release" \
	https://github.com/UniversalDependencies/UD_French-GSD.git "$dir"

train=$dir/fr_gsd-ud-train.conllu

cd "$root"
cargo build --release --bin harper-cli --features training,harper-pos-utils/threaded
cli=$root/target/release/harper-cli

"$cli" train-brill-tagger -c "$chance" "$here/trained_tagger_model.json" "$epochs" "$train"
"$cli" train-brill-chunker -c "$chance" "$here/trained_chunker_model.json" "$epochs" "$train"

echo "Trained on UD_French-GSD $release with $epochs epochs at chance $chance."
