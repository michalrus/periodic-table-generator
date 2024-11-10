use std::collections::HashMap;
use std::fmt::Write;

mod cli;
mod elements;

fn main() {
    let args = cli::Args::parse();

    let tiles = make_tiles(&args);

    let (tiles, colors) = calculate_colors(&tiles, &args);

    println!("{}", generate_svg(&tiles, &colors, &args));
}

#[derive(Debug, Clone)]
struct Tile {
    element: elements::Element,
    graphical_x: u8,
    graphical_y: u8,
    marks: Vec<String>,
}

fn make_tiles(args: &cli::Args) -> Vec<Tile> {
    elements::ALL
        .iter()
        .map(|element| {
            let period = element.period;
            let group = element.group;
            let atomic_number = element.atomic_number;

            let (graphical_y, graphical_x) = if !args.wide {
                match (period, group) {
                    (1, Some(18)) => {
                        if args.helium_in_2 {
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
                        if args.helium_in_2 {
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

            Tile {
                element: element.clone(),
                graphical_x,
                graphical_y,
                marks: vec![],
            }
        })
        .collect()
}

fn calculate_colors(
    tiles: &Vec<Tile>,
    args: &cli::Args,
) -> (
    Vec<Tile>,
    HashMap<String /* class name */, String /* color */>,
) {
    let mut mark_counter = 0;

    // FIXME: use newtype for color and class

    let mut colors: HashMap<String /* color */, String /* class */> = HashMap::new();

    (
        tiles
            .iter()
            .map(|tile| {
                let mut tile = tile.clone();
                let el = &tile.element;

                let mut do_mark = |mrk: &cli::MarkRange| {
                    let class = colors.entry(mrk.color.clone()).or_insert_with(|| {
                        format!("mark-{}", {
                            mark_counter += 1;
                            mark_counter
                        })
                    });
                    tile.marks.push(class.clone());
                };

                for mrk in &args.mark_z {
                    if mrk.ids.contains(&(el.atomic_number as u32)) {
                        do_mark(mrk);
                    }
                }

                for mrk in &args.mark_group {
                    if let Some(group) = el.group {
                        if mrk.ids.contains(&(group as u32)) {
                            do_mark(mrk);
                        }
                    }
                }

                for mrk in &args.mark_period {
                    if mrk.ids.contains(&(el.period as u32)) {
                        do_mark(mrk);
                    }
                }

                for mrk in &args.mark_block {
                    if mrk.ids.contains(&(el.block as u32)) {
                        do_mark(mrk);
                    }
                }

                tile
            })
            .collect(),
        colors.into_iter().map(|(k, v)| (v, k)).collect(),
    )
}

fn generate_svg(tiles: &Vec<Tile>, colors: &HashMap<String, String>, args: &cli::Args) -> String {
    let width: u32 = 50;
    let stroke_width: u32 = 1;

    let max_x = tiles.iter().map(|tile| tile.graphical_x).max().unwrap_or(0);
    let max_y = tiles.iter().map(|tile| tile.graphical_y).max().unwrap_or(0);

    let (viewbox_x, viewbox_y, viewbox_width, viewbox_height) = if args.pretty_padding {
        (0, 0, (max_x as u32 + 2) * width, (max_y as u32 + 2) * width)
    } else {
        (
            width / 2,
            width * 2 / 5,
            (max_x as u32 + 2) * width - (width * 3 / 2) + stroke_width,
            (max_y as u32 + 2) * width - (width * (5 + 2) / 5) + stroke_width,
        )
    };

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="{} {} {} {}">"#,
        viewbox_x, viewbox_y, viewbox_width, viewbox_height,
    );

    writeln!(
        svg,
        r#"
  <desc>
    Created with https://github.com/michalrus/periodic-table-generator
    ❯ periodic-table-generator {}
  </desc>
  <style>
    .elements text.Z {{ font-size: {}px; text-anchor: start; alignment-baseline: before-edge; }}
    .elements text:not(Z) {{ font-size: {}px; text-anchor: middle; alignment-baseline: middle; }}
    .elements rect {{ stroke: black; stroke-width: {}; fill: white; width: {}px; height: {}px; }}
    .group-numbers text, .period-numbers text {{ font-size: {}px; fill: #808080; text-anchor: middle; alignment-baseline: middle; }}"#,
        cli::escaped_argv().replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;"),
        width / 4,
        width / 2,
        stroke_width,
        width,
        width,
        width * 3/8,
    )
    .unwrap();

    {
        let mut colors_sorted = colors.into_iter().collect::<Vec<_>>();
        colors_sorted.sort();
        for (mark, color) in colors_sorted {
            writeln!(svg, r#"    .{} {{ fill: {} !important; }}"#, mark, color).unwrap();
        }
    }

    svg.push_str("  </style>\n");
    writeln!(svg, r#"  <g class="elements">"#).unwrap();

    for tile in tiles.iter() {
        let element = &tile.element;
        let x = tile.graphical_x as u32 * width;
        let y = tile.graphical_y as u32 * width;
        write!(
            svg,
            r#"    <rect x="{}" y="{}"{}/>"#,
            x,
            y,
            if !tile.marks.is_empty() {
                format!(" class=\"{}\"", tile.marks.join(" "))
            } else {
                String::new()
            }
        )
        .unwrap();

        if !args.no_z {
            let text_x = x + (3 * width / 50);
            let text_y = y + (2 * width / 50);
            write!(
                svg,
                r#"<text x="{}" y="{}" class="Z">{}</text>"#,
                text_x, text_y, element.atomic_number
            )
            .unwrap();
        }

        if args.no_symbols {
            writeln!(svg).unwrap();
        } else {
            let text_x = x + width / 2;
            let text_y = y + width / 2 + (3 * width / 50);
            writeln!(
                svg,
                r#"<text x="{}" y="{}">{}</text>"#,
                text_x,
                text_y,
                element.symbol.to_string()
            )
            .unwrap();
        }
    }

    svg.push_str("  </g>\n");

    if !args.no_group_numbers {
        write!(svg, r#"  <g class="group-numbers">"#).unwrap();

        let locations = (1..=7).flat_map(|group| match group {
            6..=7 if !args.wide => vec![
                (group, 0 * width, group * width),
                (group, 3 * width, (group + 3) * width),
            ],
            _ => vec![(group, 0 * width, group * width)],
        });

        for (group, x, y) in locations {
            let text_x = x + width * 5 / 8;
            let text_y = y + width / 2;
            write!(
                svg,
                r#"<text x="{}" y="{}">{}</text>"#,
                text_x, text_y, group
            )
            .unwrap();
        }
        svg.push_str("</g>\n");
    }

    if !args.no_period_numbers {
        write!(svg, r#"  <g class="period-numbers">"#).unwrap();

        let locations = (1..=18).flat_map(|period| match period {
            3 if !args.wide => vec![(period, period * width, 0), (period, 18 * width, 8 * width)],
            3.. if args.wide => vec![(period, (period + 14) * width, 0)],
            _ => vec![(period, period * width, 0)],
        });

        for (period, x, y) in locations {
            let text_x = x + width / 2;
            let text_y = y + width * 5 / 8;
            write!(
                svg,
                r#"<text x="{}" y="{}">{}</text>"#,
                text_x, text_y, period
            )
            .unwrap();
        }
        svg.push_str("</g>\n");
    }

    svg.push_str("</svg>");

    svg
}

// fn generate_structs(elements: &HashMap<u8, Element>) -> String {
//     let mut rv = "".to_string();

//     let sorted_elements = {
//         let mut vals: Vec<Element> = elements.values().cloned().collect();
//         vals.sort_by(|a, b| match (a.group, b.group) {
//             (None, Some(_)) => std::cmp::Ordering::Greater,
//             (Some(_), None) => std::cmp::Ordering::Less,
//             _ => (a.group, a.atomic_number).cmp(&(b.group, b.atomic_number)),
//         });
//         vals
//     };

//     writeln!(rv, "pub static RAW_ELEMENTS: &[RawElement] = &[").unwrap();

//     // FIXME: sort them by group, not Z – with lanthanoids/actinoids at the end

//     for element in sorted_elements.into_iter() {
//         writeln!(rv, "  RawElement {{").unwrap();
//         writeln!(rv, "    atomic_number: {:?},", element.atomic_number).unwrap();
//         writeln!(rv, "    symbol: {:?},", element.symbol).unwrap();
//         writeln!(rv, "    oxidation_states_main: &[],").unwrap();
//         writeln!(rv, "    oxidation_states_other: &[],").unwrap();
//         writeln!(rv, "  }},").unwrap();
//     }

//     writeln!(rv, "];").unwrap();

//     rv
// }
