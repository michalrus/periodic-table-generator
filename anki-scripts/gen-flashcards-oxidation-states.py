#! /usr/bin/env python3

import os
import subprocess
import hashlib
import json
import re
import inspect
import genanki

model = genanki.Model(
    0x44164A7A753F1DBA,  # printf '0x%s\n' $(head -c 8 /dev/urandom | xxd -p)
    "periodic-table-generator (Basic)",
    fields=[
        {"name": "Front"},
        {"name": "Back"},
    ],
    templates=[
        {
            "name": "Card 1",
            "qfmt": """<div class="front">\n{{Front}}\n</div>""",
            "afmt": """<div class="back">\n{{FrontSide}}\n\n<hr id="answer">\n\n{{Back}}\n</div>""",
        },
    ],
    css=inspect.cleandoc(
        """
        body {
          margin: 10px 10px 0 10px;
          padding: 0;
        }

        .card {
            font-family: arial;
            font-size: 20px;
            text-align: center;
            color: black;
        }
    """
    ),
)

deck = genanki.Deck(0x61C7D114830D8F68, "All::Chemistry::Oxidation levels")

bash_script_path = os.path.join(
    os.path.dirname(os.path.realpath(__file__)),
    "gen-flashcards-oxidation-state--one.sh",
)

for atomic_number in range(1, 118 + 1):
    result = subprocess.run(
        [bash_script_path, str(atomic_number)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=True,
    )
    svg = result.stdout
    svg = re.sub(r"(\.mark-(?!0(?!\d))\d+ { fill:)", r".back \1", svg)

    element = json.loads(result.stderr)

    symbol = f"""_{element["atomic_number"]}{element["symbol"]}"""

    question = "Stopnie utlenienia\n\n" + f"""\\[\\ce{{{symbol}}}\\]\n\n""" + svg

    common = set(element["oxidation_states"]["common"])
    notable = set(element["oxidation_states"]["notable"])
    combined = sorted(common.union(notable))

    latex = []
    for state in combined:
        if state in common:
            latex.append(f"\\boldsymbol{{{state}}}")
        else:
            latex.append(f"{{\\scriptstyle {state}}}")

    latex = ", ".join(latex)

    if latex == "":
        latex = "\\varnothing"

    answer = f"""\\[{latex}\\]"""

    guid = int(int(0x823BFDEB76E98C / 1000) * 1000 + atomic_number)

    note = genanki.Note(
        model=model,
        sort_field=f"oxidation states of {symbol}",
        guid=guid,
        fields=[question, answer],
    )
    deck.add_note(note)

genanki.Package(deck).write_to_file("oxidation_states.apkg")
