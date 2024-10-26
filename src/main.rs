use std::collections::HashMap;
use std::fmt::Write;

mod cli;

fn main() {
    let args = crate::cli::Args::parse();

    let elements = new_elements(args.wide, args.helium_in_2);

    println!("{}", generate_svg(&elements, args.no_symbols, args.no_z));
}

#[derive(Debug)]
struct Element {
    atomic_number: u8,
    symbol: String,
    group: Option<u8>,
    period: u8,
    graphical_x: u8,
    graphical_y: u8,
}

fn new_elements(wide: bool, helium_in_2: bool) -> HashMap<u8, Element> {
    [
        "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S",
        "Cl", "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga",
        "Ge", "As", "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd",
        "Ag", "Cd", "In", "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm",
        "Sm", "Eu", "Gd", "Tb", "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os",
        "Ir", "Pt", "Au", "Hg", "Tl", "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa",
        "U", "Np", "Pu", "Am", "Cm", "Bk", "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg",
        "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh", "Fl", "Mc", "Lv", "Ts", "Og",
    ]
    .into_iter()
    .enumerate()
    .map(|(idx, symbol)| {
        let atomic_number: u8 = (idx + 1).try_into().unwrap();

        let (period, group) = match atomic_number {
            1 => (1, Some(1)),
            2 => (1, Some(18)),
            3..=10 => (
                2,
                Some(match atomic_number {
                    3 => 1,
                    4 => 2,
                    _ => atomic_number + 8,
                }),
            ),
            11..=18 => (
                3,
                Some(match atomic_number {
                    11 => 1,
                    12 => 2,
                    _ => atomic_number - 0,
                }),
            ),
            19..=36 => (
                4,
                Some(match atomic_number {
                    19 => 1,
                    20 => 2,
                    _ => atomic_number - 18,
                }),
            ),
            37..=54 => (
                5,
                Some(match atomic_number {
                    37 => 1,
                    38 => 2,
                    _ => atomic_number - 36,
                }),
            ),
            55..=86 => (
                6,
                match atomic_number {
                    55 => Some(1),
                    56 => Some(2),
                    57..=70 => None,
                    _ => Some(atomic_number - 68),
                },
            ),
            87..=118 => (
                7,
                match atomic_number {
                    87 => Some(1),
                    88 => Some(2),
                    89..=102 => None,
                    _ => Some(atomic_number - 100),
                },
            ),
            _ => (0, None), // 0, 0 for invalid atomic numbers
        };

        let (graphical_y, graphical_x) = if !wide {
            match (period, group) {
                (1, Some(18)) => {
                    if helium_in_2 {
                        (1, 2)
                    } else {
                        (1, 18)
                    }
                }
                (6, None | Some(3)) => (period + 3, 4 + atomic_number - 57),
                (7, None | Some(3)) => (period + 3, 4 + atomic_number - 89),
                (p, Some(g)) => (p, g),
                _ => (0, 0),
            }
        } else {
            match (period, group) {
                (1, Some(18)) => {
                    if helium_in_2 {
                        (1, 2)
                    } else {
                        (1, 32)
                    }
                }
                (p, Some(g)) if g <= 2 => (p, g),
                (p, Some(g)) => (p, g + 14),
                (6, None) => (period, 3 + atomic_number - 57),
                (7, None) => (period, 3 + atomic_number - 89),
                _ => (0, 0),
            }
        };

        (
            atomic_number,
            Element {
                atomic_number,
                symbol: symbol.to_string(),
                group,
                period,
                graphical_x,
                graphical_y,
            },
        )
    })
    .collect()
}

fn generate_svg(elements: &HashMap<u8, Element>, no_symbols: bool, no_z: bool) -> String {
    let width: u32 = 50;

    let max_x = elements
        .values()
        .map(|element| element.graphical_x)
        .max()
        .unwrap_or(0);
    let max_y = elements
        .values()
        .map(|element| element.graphical_y)
        .max()
        .unwrap_or(0);

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{}" height="{}">"#,
        (max_x as u32 + 2) * width,
        (max_y as u32 + 2) * width,
    );

    writeln!(
        svg,
        r#"
  <style>
    .elements text.Z {{ font-size: {}px; text-anchor: start; alignment-baseline: before-edge; }}
    .elements text:not(Z) {{ font-size: {}px; text-anchor: middle; alignment-baseline: middle; }}
    .elements rect {{ stroke: black; stroke-width: 2; fill: white; width: {}px; height: {}px; }}
  </style>"#,
        width / 4,
        width / 2,
        width,
        width
    )
    .unwrap();

    writeln!(svg, r#"  <g class="elements">"#).unwrap();

    let sorted_keys = {
        let mut keys: Vec<u8> = elements.keys().cloned().collect();
        keys.sort();
        keys
    };

    for atomic_number in sorted_keys.into_iter() {
        if let Some(element) = elements.get(&atomic_number) {
            let x = element.graphical_x as u32 * width;
            let y = element.graphical_y as u32 * width;
            write!(svg, r#"    <rect x="{}" y="{}"/>"#, x, y).unwrap();

            if !no_z {
                let text_x = x + (3 * width / 50);
                let text_y = y + (2 * width / 50);
                write!(
                    svg,
                    r#"<text x="{}" y="{}" class="Z">{}</text>"#,
                    text_x, text_y, atomic_number
                )
                .unwrap();
            }

            if no_symbols {
                writeln!(svg).unwrap();
            } else {
                let text_x = x + width / 2;
                let text_y = y + width / 2 + (3 * width / 50);
                writeln!(
                    svg,
                    r#"<text x="{}" y="{}">{}</text>"#,
                    text_x, text_y, element.symbol
                )
                .unwrap();
            }
        }
    }

    svg.push_str("  </g>\n");
    svg.push_str("</svg>\n");

    svg
}
