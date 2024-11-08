#!/usr/bin/env bash

set -euo pipefail

usage() {
  echo >&2 "Usage: chemfig2svg [--atom-sep <NUMBER>] [--margin <NUMBER>] <chemfig expression>" >&2
  exit 1
}

if ! options=$(getopt -o '' --long atom-sep:,margin: -- "$@"); then
  usage
fi

eval set -- "$options"

# For later reproduction:
escaped_args=""
for arg in "$@"; do
  escaped_arg="'${arg//\'/\'\\\'\'}'"
  escaped_args+="$escaped_arg "
done

atom_sep=25
margin=5

while true; do
  case "$1" in
  --atom-sep)
    atom_sep="$2"
    shift 2
    ;;
  --margin)
    margin="$2"
    shift 2
    ;;
  --)
    shift
    break
    ;;
  *)
    usage
    ;;
  esac
done

if [ $# -ne 1 ]; then
  usage
fi

chemfig_expr="$1"

temp_dir=$(mktemp -d)
trap 'rm -rf "$temp_dir"' EXIT

cd "$temp_dir"

cat >chemfig_expr.tex <<EOL
\documentclass[margin=${margin}]{standalone}
\usepackage{chemfig}
\begin{document}
\setchemfig{atom sep=${atom_sep}pt}
\chemfig{
${chemfig_expr}
}
\end{document}
EOL

if ! the_log=$(latex 2>&1 chemfig_expr.tex); then
  cat >&2 <<<"$the_log"
  exit 1
fi

if ! the_log=$(dvisvgm 2>&1 --bbox="${margin}:dvi" chemfig_expr.dvi); then
  cat >&2 <<<"$the_log"
  exit 1
fi

desc_content="
      Created with https://github.com/michalrus/periodic-table-generator
      ❯ chemfig2svg $escaped_args
"

xmlstarlet >&2 ed -L -N ns="http://www.w3.org/2000/svg" \
  -d '/ns:svg/@width' \
  -d '/ns:svg/@height' \
  -d '/ns:svg/ns:defs/ns:font' \
  -i "/ns:svg/*[1]" -t elem -n "desc" -v "" \
  -s "/ns:svg/desc" -t elem -n "text" -v "" \
  -r "/ns:svg/desc/text" -v "![CDATA[$desc_content    ]]" \
  chemfig_expr.svg

sed >&2 -r 's/font-family:[^;]*;/font-family:sans-serif;/g' -i chemfig_expr.svg

# An xmlstarlet bug:
sed >&2 -r 's/  \]\]\/>/  ]]>/g' -i chemfig_expr.svg

tail -n +3 chemfig_expr.svg