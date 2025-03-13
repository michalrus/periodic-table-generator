#!/usr/bin/env bash

set -euo pipefail

usage() {
  echo >&2 "Usage: chemfig2svg [--atom-sep <NUM_PT>] [--line-width <NUM_PT>]
                   [--margin <NUM_PT>]
                   [--left-margin <NUM_PX>] [--right-margin <NUM_PX>]
                   [--no-recolor-invisible-paths]
                   <CHEMFIG_EXPR>"
  exit 1
}

if ! options=$(getopt -o '' --long atom-sep:,line-width:,margin:,left-margin:,right-margin:,no-recolor-invisible-paths -- "$@"); then
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

atom_sep=25
line_width=0.8
margin=5
left_margin=0
right_margin=0
no_recolor_invisible_paths=

while true; do
  case "$1" in
  --atom-sep)
    atom_sep="$2"
    shift 2
    ;;
  --line-width)
    line_width="$2"
    shift 2
    ;;
  --margin)
    margin="$2"
    shift 2
    ;;
  --left-margin)
    left_margin="$2"
    shift 2
    ;;
  --right-margin)
    right_margin="$2"
    shift 2
    ;;
  --no-recolor-invisible-paths)
    no_recolor_invisible_paths=1
    shift 1
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
\usetikzlibrary{decorations.pathmorphing}
\usepackage[dvipsnames]{xcolor}

\pgfdeclaredecoration{complete sines}{initial}{
  \state{initial}[
    width=+0pt,
    next state=sine,
    persistent precomputation={
      \pgfmathsetmacro\matchinglength{
        \pgfdecoratedinputsegmentlength /
        int(\pgfdecoratedinputsegmentlength/\pgfdecorationsegmentlength)
      }
      \setlength{\pgfdecorationsegmentlength}{\matchinglength pt}
    }]{}
  \state{sine}[width=\pgfdecorationsegmentlength]{
      \pgfpathsine{
        \pgfpoint
          {0.25\pgfdecorationsegmentlength}
          {0.5\pgfdecorationsegmentamplitude}
      }
      \pgfpathcosine{
        \pgfpoint
          {0.25\pgfdecorationsegmentlength}
          {-0.5\pgfdecorationsegmentamplitude}
      }
      \pgfpathsine{
        \pgfpoint
          {0.25\pgfdecorationsegmentlength}
          {-0.5\pgfdecorationsegmentamplitude}
      }
      \pgfpathcosine{
        \pgfpoint
          {0.25\pgfdecorationsegmentlength}
          {0.5\pgfdecorationsegmentamplitude}
      }
  }
  \state{final}{}
}

\tikzset{wv/.style={decorate,decoration=complete sines}}

\begin{document}
\setchemfig{atom sep=${atom_sep}pt, bond style={line width=${line_width}pt}}
\chemfig{
${chemfig_expr}
}
\end{document}
EOL

if ! the_log=$(latex </dev/null 2>&1 chemfig_expr.tex); then
  cat >&2 <<<"$the_log"
  exit 1
fi

if ! the_log=$(dvisvgm </dev/null 2>&1 --currentcolor --optimize=all --bbox="${margin}:dvi" chemfig_expr.dvi); then
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
  -i "/ns:svg/*[1]" -t elem -n "desc" -v "$desc_content  " \
  chemfig_expr.svg

# IUPAC recommends sans-serif:
sed >&2 -r 's/font-family:[^;]*;/font-family:sans-serif;/g' -i chemfig_expr.svg

# An xmlstarlet bug:
sed >&2 -r 's/  \]\]\/>/  ]]>/g' -i chemfig_expr.svg

if [ "$left_margin" != 0 ] || [ "$right_margin" != 0 ]; then
  viewBox=$(xmlstarlet sel -t -v "//@viewBox" chemfig_expr.svg)
  read -r vb_min_x vb_min_y vb_width vb_height <<<"$viewBox"
  vb_min_x_new=$(echo "$vb_min_x - $left_margin" | bc)
  vb_width_new=$(echo "$vb_width + $left_margin + $right_margin" | bc)
  xmlstarlet >&2 ed -L \
    -u "//@viewBox" -v "$vb_min_x_new $vb_min_y $vb_width_new $vb_height" \
    chemfig_expr.svg
fi

# Especially the 3D bonds are filled with a <path/> without any fill= or stroke=, and are not visible at night.
if [ -z "$no_recolor_invisible_paths" ]; then
  sed >&2 -r 's,(<path\s+d="[^"]*")(\s*/>),\1 fill="currentColor"\2,g' -i chemfig_expr.svg
fi

tail -n +3 chemfig_expr.svg
