use clap::{ArgAction, Parser};
use std::collections::HashSet;

/// Periodic table generator that can be used to generate SVGs for spaced-repetition study in Anki.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Hide element symbols
    #[arg(long)]
    pub no_symbols: bool,

    /// Hide atomic numbers
    #[arg(long)]
    pub no_z: bool,

    /// Draw it wide instead of separate lanthanoids and actinoids
    #[arg(long)]
    pub wide: bool,

    /// Draw helium in group 2 instead of 18 (for electron configurations)
    #[arg(long)]
    pub helium_in_2: bool,

    /// Color certain atomic numbers with an SVG- or CSS-compatible color
    ///
    /// A pink f-block example: --mark-z '#ffcccc:57-70,89-102'.
    ///
    /// Can be given multiple times, which are OR’ed together.
    #[arg(long, value_name = "COLOR:RANGE1[,RANGE2…]", value_parser = parse_mark_range(1,118), action = ArgAction::Append )]
    pub mark_z: Vec<MarkRange>,

    /// Like --mark-z but for groups
    #[arg(long, value_name = "COLOR:RANGE1[,RANGE2…]", value_parser = parse_mark_range(1,18), action = ArgAction::Append )]
    pub mark_group: Vec<MarkRange>,

    /// Like --mark-z but for periods
    #[arg(long, value_name = "COLOR:RANGE1[,RANGE2…]", value_parser = parse_mark_range(1,7), action = ArgAction::Append )]
    pub mark_period: Vec<MarkRange>,

    /// Like --mark-z but for blocks (1=s, 2=p, 3=d, 4=f)
    #[arg(long, value_name = "COLOR:RANGE1[,RANGE2…]", value_parser = parse_mark_range(1,4), action = ArgAction::Append )]
    pub mark_block: Vec<MarkRange>,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}

#[derive(Debug, Clone)]
pub struct MarkRange {
    pub color: String,
    pub ids: HashSet<u32>,
}

fn parse_mark_range(min: u32, max: u32) -> impl Fn(&str) -> Result<MarkRange, String> + Clone {
    move |arg: &str| -> Result<MarkRange, String> {
        let parts: Vec<&str> = arg.split(':').collect();

        if parts.len() != 2 {
            return Err("missing colon".to_string());
        }

        let color = parts[0].to_string();

        let parse_id = |s: &str| -> Result<u32, String> {
            let id = s
                .parse::<u32>()
                .map_err(|_| format!("invalid number: {}", s))?;
            if id < min || id > max {
                Err(format!("{} is out of range ({}-{})", id, min, max))
            } else {
                Ok(id)
            }
        };

        let ids = parts[1]
            .split(',')
            .map(|s| {
                if let Some((start, end)) = s.split_once('-') {
                    let start = parse_id(start)?;
                    let end = parse_id(end)?;
                    let (min, max) = (std::cmp::min(start, end), std::cmp::max(start, end));
                    Ok((min..=max).collect::<Vec<u32>>())
                } else {
                    let id = parse_id(s)?;
                    Ok::<Vec<u32>, String>(vec![id])
                }
            })
            .collect::<Result<Vec<_>, String>>()?
            .into_iter()
            .flatten()
            .collect::<HashSet<u32>>();

        Ok(MarkRange { color, ids })
    }
}
