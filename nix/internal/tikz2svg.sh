#!/usr/bin/env bash

set -euo pipefail

usage() {
  echo >&2 "Usage: tikz2svg [--margin <NUM_PT>] <TIKZ_EXPR>" >&2
  exit 1
}

if ! options=$(getopt -o '' --long margin: -- "$@"); then
  usage
fi

eval set -- "$options"

# For later reproduction:
escaped_args=""
for arg in "$@"; do
  if [ "$escaped_args" == "" ] && [ "$arg" == "--" ]; then continue; fi
  escaped_arg="'${arg//\'/\'\\\'\'}'"
  escaped_args+="$escaped_arg "
done

margin=5

while true; do
  case "$1" in
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

tikz_expr="$1"

temp_dir=$(mktemp -d)
trap 'rm -rf "$temp_dir"' EXIT

cd "$temp_dir"

cat >tikz_expr.tex <<EOL
\documentclass[margin=${margin}]{standalone}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
${tikz_expr}
\end{tikzpicture}
\end{document}
EOL

if ! the_log=$(latex </dev/null 2>&1 tikz_expr.tex); then
  cat >&2 <<<"$the_log"
  exit 1
fi

if ! the_log=$(dvisvgm </dev/null 2>&1 --currentcolor --optimize=all --bbox="${margin}:dvi" tikz_expr.dvi); then
  cat >&2 <<<"$the_log"
  exit 1
fi

desc_content="
      Created with https://github.com/michalrus/periodic-table-generator
      ❯ tikz2svg $escaped_args
"

xmlstarlet >&2 ed -L -N ns="http://www.w3.org/2000/svg" \
  -d '/ns:svg/@width' \
  -d '/ns:svg/@height' \
  -d '/ns:svg/ns:defs/ns:font' \
  -i "/ns:svg/*[1]" -t elem -n "desc" -v "" \
  -s "/ns:svg/desc" -t elem -n "text" -v "" \
  -r "/ns:svg/desc/text" -v "![CDATA[$desc_content    ]]" \
  tikz_expr.svg

# IUPAC recommends sans-serif:
sed >&2 -r 's/font-family:[^;]*;/font-family:sans-serif;/g' -i tikz_expr.svg

# An xmlstarlet bug:
sed >&2 -r 's/  \]\]\/>/  ]]>/g' -i tikz_expr.svg

tail -n +3 tikz_expr.svg
