#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 1 || ! $1 =~ ^-?[0-9]+$ ]]; then
  echo "Usage: $0 <atomic_number>"
  exit 1
fi

atomic_number="$1"

element=$(periodic-table-generator --dump "z == $atomic_number" | jq '.[0]')

# symbol=$(jq -r '.symbol' <<<"$element")
common=$(jq -r '.oxidation_states | "{" + (.common | unique | join(",")) + "}"' <<<"$element")
notable=$(jq -r '.oxidation_states | "{" + (.notable | unique | join(",")) + "}"' <<<"$element")

echo >&2 "$element"

color_________question="hsl(230,100%,75%)"
color_________same_all="$color_________question"
color______same_common="hsl(195,100%,50%)"
color_common_in_common="hsl(175,100%,80%)"
color_______all_in_all="hsl(96,80%,85%)"
color____common_in_all="hsl( 39, 77%,88%)" #"wheat" #"hsl(175,100%,91%)"

svg_raw=$(
  periodic-table-generator \
    --mark "$color_________question: z == $atomic_number" \
    --mark "$color_________same_all: z != $atomic_number && oxidation_states.common == $common
                                                         && oxidation_states.notable == $notable" \
    --mark "$color______same_common: z != $atomic_number && oxidation_states.common == $common
                                                         && oxidation_states.notable != $notable" \
    --mark "$color_common_in_common: z != $atomic_number && $common != {}
                                                         && $common in oxidation_states.common
                                                         && $common != oxidation_states.common" \
    --mark "$color_______all_in_all: z != $atomic_number && ($common + $notable) != {}
                                                         && ($common + $notable) in (oxidation_states.common + oxidation_states.notable)
                                                         && !(oxidation_states.common == $common
                                                                && oxidation_states.notable == $notable)
                                                         && !($notable == {}
                                                                && $common in oxidation_states.common)" \
    --mark "$color____common_in_all: z != $atomic_number && $common != {}
                                                         && $common in (oxidation_states.common + oxidation_states.notable)
                                                         && !($common in oxidation_states.common)
                                                         && oxidation_states.common != $common
                                                         && !(($common + $notable) in (oxidation_states.common + oxidation_states.notable))"
)

svg=$(
  head -n -1 <<<"$svg_raw"
  echo '  <g transform="translate(240, 60)">'
  echo "    <rect x=\"0\" y=\"0\" width=\"20\" height=\"20\" fill=\"$color_________same_all\"></rect>"
  echo '    <text x="25" y="15" fill="currentColor">all = all</text>'
  echo "    <rect x=\"0\" y=\"25\" width=\"20\" height=\"20\" fill=\"$color______same_common\"></rect>"
  echo '    <text x="25" y="40" fill="currentColor">common = common</text>'
  echo "    <rect x=\"0\" y=\"50\" width=\"20\" height=\"20\" fill=\"$color_common_in_common\"></rect>"
  echo '    <text x="25" y="65" fill="currentColor">common ours ⊂ common theirs</text>'
  echo "    <rect x=\"0\" y=\"75\" width=\"20\" height=\"20\" fill=\"$color_______all_in_all\"></rect>"
  echo '    <text x="25" y="90" fill="currentColor">all ours ⊂ all theirs</text>'
  echo "    <rect x=\"0\" y=\"100\" width=\"20\" height=\"20\" fill=\"$color____common_in_all\"></rect>"
  echo '    <text x="25" y="115" fill="currentColor">common ours ⊂ all theirs</text>'
  # echo "    <text x=\"220\" y=\"15\">common: $common</text>"
  # echo "    <text x=\"220\" y=\"40\">notable: $notable</text>"
  echo '  </g>'

  echo "</svg>"
)

cat <<<"$svg"
