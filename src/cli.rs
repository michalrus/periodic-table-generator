use clap::{ArgAction, Parser};
use regex::Regex;
use std::borrow::Cow;

/// Periodic table generator in the SVG format.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Hide element symbols
    #[arg(long)]
    pub no_symbols: bool,

    /// Hide atomic numbers
    #[arg(long)]
    pub no_z: bool,

    /// Hide group numbers
    #[arg(long)]
    pub no_group_numbers: bool,

    /// Hide period numbers
    #[arg(long)]
    pub no_period_numbers: bool,

    /// Draw it wide instead of separating lanthanoids and actinoids
    #[arg(long)]
    pub wide: bool,

    /// Draw helium in group 2 instead of 18 (for electron configurations)
    #[arg(long)]
    pub helium_in_2: bool,

    /// Color specific elements based on a query, can be provided multiple times.
    ///
    /// Some examples:{n}
    ///   - 'pink: z == 1'{n}
    ///   - 'pink: z >= 11 && z < 19'{n}
    ///   - 'cyan: group == 5 || (group == 15 && period <= 6)'{n}
    ///   - 'hsl(240, 100%, 80%): block == 0 || block == 1'{n}
    ///   - '#ccccff: 1 in oxidation_states.common'{n}
    ///   - 'lime: {-1, 1} in oxidation_states.common'{n}
    ///   - 'lime: 0 in (oxidation_states.common + oxidation_states.notable)'{n}
    ///   - 'lime: 1 in (oxidation_states.predicted)'{n}
    ///   - 'lime: 1 in (oxidation_states.citation_needed)'{n}
    ///   - 'wheat: (group - 10) in oxidation_states.common || group in oxidation_states.common'
    #[arg(long, value_name = "COLOR:QUERY_EXPR", value_parser = parse_mark_query, action = ArgAction::Append )]
    pub mark: Vec<MarkQuery>,

    /// Don't maximally downsize the viewbox to the bounding box of the table
    #[arg(long)]
    pub pretty_padding: bool,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}

#[derive(Debug, Clone)]
pub struct MarkQuery {
    pub color: String,
    pub query: crate::query::Query,
}

fn parse_mark_query(arg: &str) -> Result<MarkQuery, String> {
    let re = Regex::new(r#"(?ms)\s*:\s*"#).unwrap();
    let mut parts = re.splitn(arg, 2);
    let color = parts
        .next()
        .ok_or("color not found".to_string())?
        .to_string();
    let query = parts.next().ok_or("query not found".to_string())?;
    let query = crate::query::Query::new(query)?;
    Ok(MarkQuery { color, query })
}

/// Used for SVG comments (future reproducibility).
pub fn escaped_argv() -> String {
    std::env::args()
        .skip(1)
        .map(|arg| shell_escape::escape(Cow::from(arg)).to_string())
        .collect::<Vec<String>>()
        .join(" ")
}
