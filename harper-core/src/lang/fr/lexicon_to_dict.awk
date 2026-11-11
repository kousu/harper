# Convert the Grammalecte French lexicon into Harper's `dictionary.dict` format.
#
# Input: lexique-grammalecte-fr-*.txt (tab-separated; column 3 is the word form,
# column 5 its tags, column 11 its sub-dictionary).
# Output: one `word/FLAGS` line per form; flags are defined in `annotations.json`.
#
# Forms in sub-dictionary "X" are left out, and forms in frequency class 6 or above are
# marked common. Words listed more than once get the union of their flags.

BEGIN {
	FS = "\t"

	# Grammalecte tag -> Harper flags.
	flag["nom"] = "N"
	flag["npr"] = "NO"
	flag["prn"] = "NO"
	flag["patr"] = "NO"
	flag["adj"] = "J"
	flag["adv"] = "R"
	flag["negadv"] = "R"
	flag["advint"] = "R"
	flag["det"] = "D"
	flag["detind"] = "D"
	flag["detneg"] = "D"
	flag["detpos"] = "D5"
	flag["detdem"] = "DM"
	flag["prep"] = "P"
	flag["prepv"] = "P"
	flag["cjco"] = "C"
	flag["cjsub"] = "C"
	flag["cj"] = "C"
	flag["proint"] = "I"
	flag["prorel"] = "I"
	flag["proind"] = "I"
	flag["prodem"] = "I"
	flag["proneg"] = "I"
	flag["proadv"] = "I"
	flag["propersuj"] = "Isa"
	flag["properobj"] = "Ioa"
	flag["infi"] = "b"
	flag["ppas"] = "T"
	flag["ppre"] = "6"
	flag["mas"] = "♂"
	flag["fem"] = "♀"
}

# Data lines start with a numeric id; everything before them is a header.
$1 !~ /^[0-9]+$/ || $11 ~ /^X/ { next }

{
	word = $3
	gsub(/’/, "'", word)

	n = split($5, tags, " ")
	is_pronoun = 0
	is_noun = 0
	for (i = 1; i <= n; i++) {
		t = tags[i]
		if (t ~ /^v[0-3]/) flags[word] = flags[word] "V"
		else if (t in flag) flags[word] = flags[word] flag[t]
		if (t ~ /^pro/) is_pronoun = 1
		if (t == "nom" || t == "npr" || t == "prn" || t == "patr") is_noun = 1
	}

	# Number and person mean different things for nouns and pronouns.
	for (i = 1; i <= n; i++) {
		t = tags[i]
		if (is_noun && (t == "sg" || t == "inv")) flags[word] = flags[word] "0"
		if (is_noun && (t == "pl" || t == "inv")) flags[word] = flags[word] "9"
		if (is_pronoun && t == "sg") flags[word] = flags[word] "."
		if (is_pronoun && t == "pl") flags[word] = flags[word] ":"
		if (is_pronoun && t == "1pe") flags[word] = flags[word] "1"
		if (is_pronoun && t == "2pe") flags[word] = flags[word] "2"
		if (is_pronoun && t == "3pe") flags[word] = flags[word] "3"
	}

	# Column 20 is a frequency class from 0 (rare) to 9 (very common).
	if ($20 >= 6) flags[word] = flags[word] "~"

	if (!(word in seen)) {
		seen[word] = 1
		order[++count] = word
	}
}

END {
	# Harper's French lexer keeps "l'on" whole, as the UD treebank does, so it needs an
	# entry of its own. It's the pronoun "on" with a euphonic "l'".
	if ("on" in flags && !("l'on" in seen)) {
		flags["l'on"] = flags["on"]
		order[++count] = "l'on"
	}

	print count
	for (i = 1; i <= count; i++) {
		word = order[i]
		print word (flags[word] == "" ? "" : "/" unique(flags[word]))
	}
}

# Drop repeated flags, keeping their first-seen order.
function unique(s,    out, c, i) {
	out = ""
	for (i = 1; i <= length(s); i++) {
		c = substr(s, i, 1)
		if (index(out, c) == 0) out = out c
	}
	return out
}
