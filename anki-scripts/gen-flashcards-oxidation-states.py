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

deck = genanki.Deck(0x61C7D114830D8F68, "All::Chemistry::Oxidation states")

bash_script_path = os.path.join(
    os.path.dirname(os.path.realpath(__file__)),
    "gen-flashcards-oxidation-state--one.sh",
)


def make_latex(chosen, bold):
    latex = []
    for state in sorted(chosen):
        if state in bold:
            latex.append(f"\\boldsymbol{{{state}}}")
        else:
            latex.append(f"{{\\scriptstyle {state}}}")
    if len(latex) == 0:
        return "\\varnothing"
    return ", ".join(latex)


notes_common = []
notes_notable = []

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

    common = set(element["oxidation_states"]["common"])
    notable = set(element["oxidation_states"]["notable"])

    question_common = (
        "Stopnie utlenienia – <b>powszechne</b>:\n\n"
        + f"""\\[\\ce{{{symbol}}}\\]\n\n"""
        + svg
    )
    question_notable = (
        "Stopnie utlenienia – <b>godne uwagi</b>:\n\n"
        + f"""\\[\\ce{{{symbol}}}\\]\n\n"""
        + f"""oprócz powszechnych: \\({make_latex(common, common)}\\)<br><br>"""
        + svg
    )

    answer_common = (
        """\\["""
        + make_latex(common, common)
        + """\\]\n\n"""
        + """\\[("""
        + make_latex(common.union(notable), common)
        + """)\\]"""
    )
    answer_notable = (
        """\\["""
        + make_latex(notable, common)
        + """\\]\n\n"""
        + """\\[("""
        + make_latex(common.union(notable), common)
        + """)\\]"""
    )

    guid_common = int(int(0x823BFDEB76E98C / 1000) * 1000 + atomic_number)
    guid_notable = int(int(0x823BFDEB76E98C / 1000) * 1000 + 200 + atomic_number)

    notes_common.append(
        genanki.Note(
            model=model,
            sort_field=f"oxidation states (common) of {symbol}",
            guid=guid_common,
            fields=[question_common, answer_common],
        )
    )
    notes_notable.append(
        genanki.Note(
            model=model,
            sort_field=f"oxidation states (notable) of {symbol}",
            guid=guid_notable,
            fields=[question_notable, answer_notable],
        )
    )

for n in notes_common:
    deck.add_note(n)
for n in notes_notable:
    deck.add_note(n)

genanki.Package(deck).write_to_file("oxidation_states.apkg")
