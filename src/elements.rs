use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Symbol(String);

impl std::string::ToString for Symbol {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    pub atomic_number: u8,
    pub symbol: Symbol,
    pub group: Option<u8>,
    pub period: u8,
    /// 0=s, 1=p, 2=d, 4=3
    pub block: u8,
    pub oxidation_states: OxidationStates,
}

#[derive(Debug, Clone)]
pub struct OxidationStates {
    pub common: BTreeSet<i8>,
    pub notable: BTreeSet<i8>,
    pub predicted: BTreeSet<i8>,
    pub citation_needed: BTreeSet<i8>,
}

pub static ALL: Lazy<Vec<Element>> = Lazy::new(|| {
    SYMBOLS_IN_Z_ORDER
        .into_iter()
        .enumerate()
        .map(|(idx, &symbol)| {
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

            let block = match (group, period, atomic_number) {
                (Some(1..=2), _, _) | (_, _, 2) => 0,
                (Some(13..=18), _, _) => 1,
                (Some(3..=12), _, _) => 2,
                (None, _, _) => 3,
                _ => panic!("impossible"),
            };

            let symbol = Symbol(symbol.to_string());
            let oxidation_states = OXIDATION_STATES.get(&symbol).unwrap().clone();

            Element {
                atomic_number,
                symbol,
                group,
                period,
                block,
                oxidation_states,
            }
        })
        .collect()
});

static SYMBOLS_IN_Z_ORDER: &'static [&'static str] = &[
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S", "Cl",
    "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As",
    "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In",
    "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb",
    "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl",
    "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk",
    "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh",
    "Fl", "Mc", "Lv", "Ts", "Og",
];

pub static OXIDATION_STATES: Lazy<HashMap<Symbol, OxidationStates>> = Lazy::new(|| {
    #[derive(PartialEq)]
    enum Ctx {
        None,
        Common,
        Notable,
        Predicted,
    }
    let mut ctx = Ctx::None;

    let re_comments = Regex::new(r#"(?ms)<!--.*?-->"#).unwrap();
    let re_sup = Regex::new(r#"(?ms),?\s*<sup>\s*\?\s*</sup>"#).unwrap();
    let re_refs = Regex::new(r#"(?ms)<ref(\s|>).*?</ref>"#).unwrap();
    let re_noinclude = Regex::new(r#"(?ms)<noinclude>.*?</noinclude>"#).unwrap();
    let re_default = Regex::new(r#"(?m)^\|#default=.*$"#).unwrap();
    let re_infobox = Regex::new(r#"(?m)\{\{Infobox .*$"#).unwrap();

    let sources = OXIDATION_STATES_WIKIPEDIA;
    let sources = sources.replace("−", "-"); // “−” U+2212 Minus Sign
    let sources = re_comments.replace_all(&sources, "");
    let sources = re_sup.replace_all(&sources, "?");
    let sources = re_refs.replace_all(&sources, "");
    let sources = re_noinclude.replace_all(&sources, "");
    let sources = re_default.replace_all(&sources, "");
    let sources = re_infobox.replace_all(&sources, "");

    let re_captures =
        Regex::new(r#"^\|\s*([A-Z][a-z]*)\s*=((?:\s*\(?[+-]?\d*\)?\??,?)*)$"#).unwrap();
    let re_noise = Regex::new(r#"[^-?0-9]+"#).unwrap();

    let mut result = HashMap::new();

    for line in sources.lines() {
        if !line.starts_with('|') {
            continue;
        }
        if line.starts_with("|common={{") {
            ctx = Ctx::Common;
        } else if line.starts_with("|notable={{") {
            ctx = Ctx::Notable;
        } else if line.starts_with("|predicted={{") {
            ctx = Ctx::Predicted;
        } else {
            if ctx != Ctx::None {
                let cpt = re_captures.captures(line).unwrap();
                let symbol = Symbol(cpt.get(1).unwrap().as_str().to_string());
                let states = cpt.get(2).unwrap().as_str();
                let states = re_noise.replace_all(states, " ");
                let states = states.trim();

                let existing = result.entry(symbol).or_insert_with(|| OxidationStates {
                    common: BTreeSet::new(),
                    notable: BTreeSet::new(),
                    predicted: BTreeSet::new(),
                    citation_needed: BTreeSet::new(),
                });

                for state in states.split_whitespace() {
                    if state.ends_with('?') {
                        existing
                            .citation_needed
                            .insert(str::parse(&state[..state.len() - 1]).unwrap());
                    } else {
                        let state = str::parse(state).unwrap();
                        let _ = match ctx {
                            Ctx::Common => existing.common.insert(state),
                            Ctx::Notable => existing.notable.insert(state),
                            Ctx::Predicted => existing.predicted.insert(state),
                            _ => false,
                        };
                    }
                }
            }
        }
    }

    result
});

/// Source: (accessed on 2024-11-10) <https://en.wikipedia.org/w/index.php?action=edit&title=Template%3AElement-symbol-to-oxidation-state-data&mfnoscript=1> via <https://en.wikipedia.org/wiki/Oxidation_state#List_of_oxidation_states_of_the_elements>.
static OXIDATION_STATES_WIKIPEDIA: &'static str = r#"
{{ {{{os-formatter|Element-symbol-to-oxidation-state-echo}}}
|symbol={{{symbol|}}}
|common={{#switch:{{{symbol|}}}
|H=−1, +1
|He=
|Li=+1
|Be=+2
|B=+3
|C=−4, +4
|N=−3, +3, +5
|O=−2
|F=−1
|Ne=
|Na=+1
|Mg=+2
|Al=+3
|Si=−4, +4
|P=−3, +3, +5
|S=−2, +2, +4, +6
|Cl=−1, +1, +3, +5, +7
|Ar=
|K=+1
|Ca=+2
|Sc=+3
|Ti=+4
|V=+5
|Cr=+3, +6
|Mn=+2, +4, +7
|Fe=+2, +3
|Co=+2, +3
|Ni=+2
|Cu=+2
|Zn=+2
|Ga=+3
|Ge=−4, +2, +4
|As=−3, +3, +5
|Se=−2, +2, +4, +6
|Br=−1, +1, +3, +5
|Kr=+2
|Rb=+1
|Sr=+2
|Y=+3
|Zr=+4
|Nb=+5
|Mo=+4, +6
|Tc=+4, +7
|Ru=+3, +4
|Rh=+3
|Pd=+2, +4
|Ag=+1
|Cd=+2
|In=+3
|Sn=−4, +2, +4
|Sb=−3, +3, +5
|Te=−2, +2, +4, +6
|I=−1, +1, +3, +5, +7
|Xe=+2, +4, +6
|Cs=+1
|Ba=+2
|La=+3
|Ce=+3, +4
|Pr=+3
|Nd=+3
|Pm=+3
|Sm=+3
|Eu=+2, +3
|Gd=+3
|Tb=+3
|Dy=+3
|Ho=+3
|Er=+3
|Tm=+3
|Yb=+3
|Lu=+3
|Hf=+4
|Ta=+5
|W=+4, +6
|Re=+4
|Os=+4
|Ir=+3, +4
|Pt=+2, +4
|Au=+3
|Hg=+1, +2
|Tl=+1, +3
|Pb=+2, +4
|Bi=+3
|Po=−2, +2, +4
|At=−1, +1
|Rn=
|Fr=+1
|Ra=+2
|Ac=+3
|Th=+4
|Pa=+5
|U=+6
|Np=+5
|Pu=+4
|Am=+3
|Cm=+3
|Bk=+3
|Cf=+3
|Es=+3
|Fm=+3
|Md=+3
|No=+3
|Lr=+3
|Rf=+4
|Db=<!-- no common, only predicted for the rest -->
|Sg=
|Bh=
|Hs=
|Mt=
|Ds=
|Rg=
|Cn=
|Nh=
|Fl=
|Mc=
|Lv=
|Ts=
|Og=
|Uue=
|Ubn=
|Ubu=
|Ubb=
|Ubt=
|Ubq=
|Ubp=
|Ubh=
<!--- default for symbol --->
|#default={{#if:{{{symbol|}}}|{{error|[[:Template:Infobox element/symbol-to-oxidation-state]]: Symbol "{{{symbol|}}}" not known}}}}
}}
|notable={{#switch:{{{symbol|}}}
<!--- Period 1 --->
| H=
| He=
<!--- Period 2 --->
| Li=
| Be= 0,<ref name=ZeroValentBeryllium>Be(0) has been observed; see {{cite web |title= Beryllium(0) Complex Found |url= https://www.chemistryviews.org/details/news/9426001/Beryllium0_Complex_Found.html |publisher= [[Chemistry Europe]] |date= 13 June 2016}}</ref> +1<ref>{{cite web|url=http://bernath.uwaterloo.ca/media/252.pdf|title=Beryllium: Beryllium(I) Hydride compound data|access-date=2007-12-10|publisher=bernath.uwaterloo.ca}}</ref>
| B=−5,<ref>B(−5) has been observed in Al<sub>3</sub>BC, see {{cite news|url=https://d-nb.info/995006210/34|first1=Melanie|last1=Schroeder|title=Eigenschaften von borreichen Boriden und Scandium-Aluminium-Oxid-Carbiden|page=139|language=de}}</ref> −1,<ref>B(−1) has been observed in [[magnesium diboride]] (MgB<sub>2</sub>), see {{cite book|url=https://books.google.com/books?id=2RgbAgAAQBAJ&pg=PA315|title=Chemical Structure and Reactivity: An Integrated Approach|first1=James|last1=Keeler|first2=Peter|last2=Wothers|publisher=Oxford University Press|year=2014|isbn=9780199604135}}</ref> 0,<ref>{{cite journal|doi=10.1126/science.1221138|title=Ambient-Temperature Isolation of a Compound with a Boron-Boron Triple Bond|year=2012|last1=Braunschweig|first1=H.|last2=Dewhurst|first2=R. D. |last3=Hammond|first3=K.|last4=Mies|first4=J.|last5=Radacki|first5=K.|last6=Vargas|first6=A.|journal=Science|volume=336|issue=6087|pages=1420–2|pmid=22700924|bibcode=2012Sci...336.1420B|s2cid=206540959}}</ref> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref><ref>{{cite journal| url=http://bernath.uwaterloo.ca/media/125.pdf| title=Infrared Emission Spectroscopy of BF and AIF| author1=Zhang, K.Q.| author2=Guo, B.|author3=Braun, V.| author4=Dulick, M.| author5=Bernath, P.F.| journal=J. Molecular Spectroscopy| volume=170| issue=1| year=1995| page=82| doi=10.1006/jmsp.1995.1058|bibcode=1995JMoSp.170...82Z}}</ref><ref>{{cite book|url=https://www.deutsche-digitale-bibliothek.de/binary/KKUKEQ5AXZBNJVU7NJCHZB4UXT2HAGJE/full/1.pdf|author=Schroeder, Melanie |title=Eigenschaften von borreichen Boriden und Scandium-Aluminium-Oxid-Carbiden|page=139|language=de}}</ref>
| C=−3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0, +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| N=−2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<ref>[[Tetrazole]]s contain a pair of double-bonded nitrogen atoms with oxidation state 0 in the ring. A Synthesis of the parent 1H-tetrazole, {{chem2|CH2N4}} (two atoms N(0)) is given in {{cite journal | last=Henry | first=Ronald A. | last2=Finnegan | first2=William G. | title=An Improved Procedure for the Deamination of 5-Aminotetrazole | journal=Journal of the American Chemical Society | volume=76 | issue=1 | date=1954 | issn=0002-7863 | doi=10.1021/ja01630a086 | pages=290–291}}</ref> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| O= −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0, +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| F=
| Ne=
<!--- Period 3 --->
| Na=−1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0<ref>The compound [[NaCl]] has been shown in experiments to exists in several unusual [[stoichiometry|stoichiometries]] under high pressure, including Na<sub>3</sub>Cl in which contains a layer of sodium(0) atoms; see {{Cite journal |last1=Zhang |first1=W. |last2=Oganov |first2=A. R. |last3=Goncharov |first3=A. F. |last4=Zhu |first4=Q. |last5=Boulfelfel |first5=S. E. |last6=Lyakhov |first6=A. O. |last7=Stavrou |first7=E. |last8=Somayazulu |first8=M. |last9=Prakapenka |first9=V. B. |last10=Konôpková |first10=Z. |year=2013 |title=Unexpected Stable Stoichiometries of Sodium Chlorides |journal=Science |volume=342 |issue=6165 |pages=1502–1505 |arxiv=1310.7674 |bibcode=2013Sci...342.1502Z |doi=10.1126/science.1244989 |pmid=24357316|s2cid=15298372}}</ref>
| Mg= 0,<ref>Mg(0) has been synthesized in a compound containing a Na<sub>2</sub>Mg<sub>2</sub><sup>2+</sup> cluster coordinated to a bulky organic ligand; see {{cite journal |first1=B. |last1=Rösch |first2=T. X. |last2=Gentner |first3=J. |last3=Eyselein |first4=J. |last4=Langer |first5=H. |last5=Elsen |first6=W. |last6=Li |first7=S. |last7=Harder |title=Strongly reducing magnesium(0) complexes |doi=10.1038/s41586-021-03401-w |journal=Nature |volume=592 |year=2021 |issue=7856 |pages=717–721 |pmid=33911274 |bibcode=2021Natur.592..717R |s2cid=233447380 |postscript=none}}</ref> +1<ref>{{cite journal|url=http://bernath.uwaterloo.ca/media/24.pdf| title=The spectrum of magnesium hydride |author=Bernath, P. F. |author2=Black, J. H. |author3=Brault, J. W. |name-list-style=amp |bibcode=1985ApJ...298..375B | doi=10.1086/163620
|journal=Astrophysical Journal|volume=298| year=1985| page=375}}. See also [[Low valent magnesium compounds]].</ref>
| Al=−2,<sup>?</sup> −1,<sup>?</sup> 0,<ref>Unstable carbonyl of Al(0) has been detected in reaction of [[Trimethylaluminum|Al<sub>2</sub>(CH<sub>3</sub>)<sub>6</sub>]] with carbon monoxide; see {{cite journal |first1=Ramiro |last1=Sanchez |first2=Caleb |last2=Arrington |first3=C. A. |last3=Arrington Jr. |title=Reaction of trimethylaluminum with carbon monoxide in low-temperature matrixes |journal=American Chemical Society |volume=111 |issue=25 |date=December 1, 1989 |page=9110-9111 |doi=10.1021/ja00207a023 |osti=6973516 |url=https://www.osti.gov/biblio/6973516-reaction-trimethylaluminum-carbon-monoxide-low-temperature-matrices}}</ref> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref><ref>{{cite journal| title = Aluminum(I) and Gallium(I) Compounds: Syntheses, Structures, and Reactions|author1=Dohmeier, C. |author2=Loos, D. |author3=Schnöckel, H. | journal = Angewandte Chemie International Edition | year =1996| volume =35|issue=2 | pages =129–149| doi =10.1002/anie.199601291}}</ref> +2<ref>{{cite journal| author = Tyte, D. C. | title = Red (B2Π–A2σ) Band System of Aluminium Monoxide| doi = 10.1038/202383a0 | journal = Nature | volume = 202| issue = 4930 | year = 1964| page = 383 | bibcode=1964Natur.202..383T| s2cid = 4163250}}</ref>
| Si=−3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<ref name=ZeroValentTin>{{cite web |title=New Type of Zero-Valent Tin Compound |url= https://www.chemistryviews.org/details/news/9745121/New_Type_of_Zero-Valent_Tin_Compound.html |publisher=[[Chemistry Europe]] |date=27 August 2016}}</ref> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref><ref>{{cite journal|author=Ram, R. S.|title=Fourier Transform Emission Spectroscopy of the A2D–X2P Transition of SiH and SiD|url=http://bernath.uwaterloo.ca/media/184.pdf |journal=J. Mol. Spectr. |volume=190|issue=2|pages=341–352|year=1998|pmid=9668026|display-authors=etal|doi=10.1006/jmsp.1998.7582}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| P = −2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<ref>{{cite journal|doi=10.1021/ja807828t|title=Carbene-Stabilized Diphosphorus|year=2008|last1=Wang|first1=Yuzhong|last2=Xie|first2=Yaoming|last3=Wei|first3=Pingrong|last4=King|first4=R. Bruce|last5=Schaefer|first5=Iii|last6=Schleyer|first6=Paul v. R.|last7=Robinson|first7=Gregory H.|journal=Journal of the American Chemical Society|volume=130|issue=45|pages=14970–1|pmid=18937460}}</ref> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref><ref>{{cite journal|doi=10.1021/ic060186o|pmid=16903744|title=Phosphorus(I) Iodide: A Versatile Metathesis Reagent for the Synthesis of Low Oxidation State Phosphorus Compounds|year=2006|last1=Ellis|first1=Bobby D.|last2=MacDonald|first2=Charles L. B.|journal=Inorganic Chemistry|volume=45|issue=17|pages=6864–74}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| S= −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0, +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Cl= +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +6<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Ar=
<!--- Period 4 --->
| K=−1<sup>?</sup>
| Ca=+1<ref name="West">{{cite journal|last1=Krieck|first1=Sven|last2=Görls|first2=Helmar|last3=Westerhausen|first3=Matthias|title=Mechanistic Elucidation of the Formation of the Inverse Ca(I) Sandwich Complex [(thf)3Ca(μ-C6H3-1,3,5-Ph3)Ca(thf)3] and Stability of Aryl-Substituted Phenylcalcium Complexes|journal=Journal of the American Chemical Society|volume=132|issue=35|pages=12492–12501|year=2010|pmid=20718434|doi=10.1021/ja105534w}}</ref>
| Sc= 0,<ref name="Cloke1991">{{cite journal |author=Cloke, F. Geoffrey N. |author2=Khan, Karl |author3=Perutz, Robin N. |name-list-style=amp |date=1991|title=η-Arene complexes of scandium(0) and scandium(II) |journal= J. Chem. Soc., Chem. Commun.|issue=19|pages=1372–1373|doi= 10.1039/C39910001372}}</ref> +1,<ref name="Smith">{{cite journal|title=Diatomic Hydride and Deuteride Spectra of the Second Row Transition Metals|first=R. E.|last=Smith | journal=Proceedings of the Royal Society of London. Series A, Mathematical and Physical Sciences|volume=332|pages=113–127|issue=1588|year=1973|doi=10.1098/rspa.1973.0015|bibcode=1973RSPSA.332..113S |s2cid=96908213}}</ref> +2<ref name="McGuire">{{cite journal|title=Preparation and Properties of Scandium Dihydride|first=Joseph C.|last=McGuire|author2=Kempter, Charles P.|journal=Journal of Chemical Physics|volume=33|issue=5|pages=1584–1585|year=1960|doi=10.1063/1.1731452|bibcode=1960JChPh..33.1584M }}</ref>
| Ti=−2,<sup>?</sup> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<ref>{{cite journal |last1= Jilek |first1= Robert E. |last2= Tripepi |first2= Giovanna |last3= Urnezius |first3= Eugenijus |last4= Brennessel |first4= William W. |last5= Young |first5= Victor G. Jr. |last6= Ellis |first6= John E. |title= Zerovalent titanium–sulfur complexes. Novel dithiocarbamato derivatives of {{awrap|Ti(CO)<sub>6</sub>:[Ti(CO)<sub>4</sub>(S<sub>2</sub>CNR<sub>2</sub>)]<sup>−</sup>}} |journal= Chem. Commun. |issue= 25 |year= 2007 |pages= 2639–2641 |doi= 10.1039/B700808B |pmid= 17579764}}</ref> +1,<sup>?</sup> <ref>{{cite journal|author=Andersson, N.|title=Emission spectra of TiH and TiD near 938 nm|journal=J. Chem. Phys.|volume=118|issue=8|year=2003|page=10543|doi=10.1063/1.1539848|bibcode=2003JChPh.118.3543A |display-authors=etal}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| V=−3,<sup>?</sup> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<sup>?</sup> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Cr=−4,<sup>?</sup> −2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<sup>?</sup> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Mn=−3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<sup>?</sup> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>, +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>, +6<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Fe=−4,<sup>?</sup> −2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<sup>?</sup> +1,<ref>{{cite journal|last1=Ram |first1=R. S. |last2=Bernath |first2=P. F. |journal=Journal of Molecular Spectroscopy |volume=221| year=2003| page=261|bibcode=2003JMoSp.221..261R |doi=10.1016/S0022-2852(03)00225-X|title=Fourier transform emission spectroscopy of the g<sup>4</sup>Δ&ndash;a<sup>4</sup>Δ system of FeCl|issue=2}}</ref> +4,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5,<ref>{{cite journal|doi=10.1002/zaac.19824910109|title=Recent developments in the field of high oxidation states of transition elements in oxides stabilization of six-coordinated Iron(V)|year=1982|last1=Demazeau|first1=G.|journal=Zeitschrift für anorganische und allgemeine Chemie|volume=491|pages=60–66|last2=Buffat|first2=B.|last3=Pouchard|first3=M.|last4=Hagenmuller|first4=P.}}</ref> +6,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +7<ref>{{cite journal|doi= 10.1039/C6CP06753K|pmid=27812577|title=Experimental and theoretical identification of the Fe(VII) oxidation state in FeO<sub>4</sub><sup>−</sup>|year=2016|last1=Lu|first1=J.|journal=Physical Chemistry Chemical Physics|volume=18|issue=45|pages=31125–31131|last2=Jian|first2=J.|last3=Huang|first3=W.|last4=Lin|first4=H.|last5=Li|first5=J|last6=Zhou|first6=M.|bibcode=2016PCCP...1831125L}}</ref>
| Co=−3,<sup>?</sup> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<sup>?</sup> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5<ref name=greenwood>{{Greenwood&Earnshaw2nd|pages=1117–1119}}</ref>
| Ni=−2,<sup>?</sup> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<sup>?</sup> +1,<ref>{{cite journal| title=A Dinuclear Nickel(I) Dinitrogen Complex and its Reduction in Single-Electron Steps| journal=Angewandte Chemie International Edition| year=2009| volume=48 | issue=18| pages=3357–61| doi=10.1002/anie.200805862| last1=Pfirrmann| first1=Stefan| last2=Limberg| first2=Christian| last3=Herwig| first3=Christian| last4=Stößer| first4=Reinhard| last5=Ziemer| first5=Burkhard| pmid=19322853}}</ref> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4<ref>{{cite journal| title=A Stable Tetraalkyl Complex of Nickel(IV)| journal=Angewandte Chemie International Edition| year=2009| volume=48 | issue=2| pages=290–4| doi=10.1002/anie.200804435| last1=Carnes| first1=Matthew| last2=Buccella| first2=Daniela| last3=Chen| first3=Judy Y.-C.| last4=Ramirez| first4=Arthur P.| last5=Turro| first5=Nicholas J.| last6=Nuckolls| first6=Colin| last7=Steigerwald| first7=Michael| pmid=19021174}}</ref>
| Cu=−2,<sup>?</sup> 0,<ref>{{cite journal|first1=Marc-Etienne|last1=Moret|first2=Limei|last2=Zhang|first3=Jonas C.|last3=Peters|title=A Polar Copper–Boron One-Electron σ-Bond|journal=J. Am. Chem. Soc|year=2013|volume=135|issue=10|pages=3792–3795|doi=10.1021/ja4006578|pmid=23418750|url=https://authors.library.caltech.edu/37931/ }}</ref> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Zn=−2,<sup>?</sup> 0,<sup>?</sup> +1<sup>?</sup>
| Ga=−5,<sup>?</sup> −4,<sup>?</sup> −3,<ref>Ga(−3) has been observed in LaGa, see {{cite journal|lang=de|first1=Ines|last1=Dürr|first2=Britta|last2=Bauer|first3=Caroline|last3=Röhr|title=Lanthan-Triel/Tetrel-ide La(Al,Ga)<sub>''x''</sub>(Si,Ge)<sub>1-''x''</sub>. Experimentelle und theoretische Studien zur Stabilität intermetallischer 1:1-Phasen|journal=Z. Naturforsch.|year=2011|volume=66b|pages=1107–1121|url=http://www.znaturforsch.com/s66b/s66b1107.pdf}}</ref> −2,<sup>?</sup> −1,<sup>?</sup> 0,<sup>?</sup> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref><ref>{{cite thesis|url=http://www.uni-kassel.de/upress/online/frei/978-3-7281-2597-2.volltext.frei.pdf|author=Hofmann, Patrick |title=Colture. Ein Programm zur interaktiven Visualisierung von Festkörperstrukturen sowie Synthese, Struktur und Eigenschaften von binären und ternären Alkali- und Erdalkalimetallgalliden|page=72|language=de|date=1997|publisher=PhD Thesis, ETH Zurich|doi=10.3929/ethz-a-001859893|hdl=20.500.11850/143357 |isbn=978-3728125972}}</ref>
| Ge=−4,<sup>?</sup> −3,<sup>?</sup> −2,<sup>?</sup> −1,<sup>?</sup> 0,<ref name=ZeroValentTin2>{{cite web |title=New Type of Zero-Valent Tin Compound |url= https://www.chemistryviews.org/details/news/9745121/New_Type_of_Zero-Valent_Tin_Compound.html |publisher=[[Chemistry Europe]] |date=27 August 2016}}</ref> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| As=−2,<sup>?</sup> −1,<sup>?</sup> 0,<ref>{{cite journal|doi=10.1002/chem.200902840|title=Carbene Stabilization of Diarsenic: From Hypervalency to Allotropy|year=2010|last1=Abraham|first1=Mariham Y.|last2=Wang|first2=Yuzhong|last3=Xie|first3=Yaoming|last4=Wei|first4=Pingrong|last5=Shaefer III|first5=Henry F.|last6=Schleyer|first6=P. von R.|last7=Robinson|first7=Gregory H.|journal=Chemistry: A European Journal|volume=16|issue=2|pages=432–5|pmid=19937872}}</ref> +1,<ref>{{cite journal|doi=10.1021/ic049281s|pmid=15360247|title=Stabilized Arsenic(I) Iodide: A Ready Source of Arsenic Iodide Fragments and a Useful Reagent for the Generation of Clusters|year=2004|last1=Ellis|first1=Bobby D.|last2=MacDonald|first2=Charles L. B.|journal=Inorganic Chemistry|volume=43|issue=19|pages=5981–6}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4<sup>?</sup>
| Se=−1,<sup>?</sup> 0,<ref>A Se(0) atom has been identified using DFT in [ReOSe(2-pySe)<sub>3</sub>]; see {{cite journal|doi=10.1016/j.inoche.2014.04.003|title=Synthesis and structure of [ReOSe(2-Se-py)3]: A rhenium(V) complex with selenium(0) as a ligand|year=2014|last1=Cargnelutti|first1=Roberta|last2=Lang|first2=Ernesto S.|last3=Piquini|first3=Paulo|last4=Abram|first4=Ulrich|journal=Inorganic Chemistry Communications|volume=45|pages=48–50|issn=1387-7003}}</ref> +1,<ref>{{Greenwood&Earnshaw}}</ref> +3,<sup>?</sup> +5<sup>?</sup>
| Br= +2,<ref>Br(II) is known to occur in bromine monoxide [[Radical (chemistry)|radical]]; see [https://pubs.acs.org/doi/10.1021/j100382a032 Kinetics of the bromine monoxide radical + bromine monoxide radical reaction]</ref> +4,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +7<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Kr=+1,<sup>?</sup> +2
<!--- Period 5 --->
| Rb=−1<sup>?</sup>
| Sr=+1<ref>{{cite journal|url=http://bernath.uwaterloo.ca/media/149.pdf|title=High-Resolution Infrared Emission Spectrum of Strontium Monofluoride|journal=J. Molecular Spectroscopy| volume=175|issue=1|page=158|year=1996|doi=10.1006/jmsp.1996.0019|bibcode=1996JMoSp.175..158C|last1=Colarusso|first1=P.|last2=Guo|first2=B.|last3=Zhang|first3=K.-Q.|last4=Bernath|first4=P. F. }}</ref>
| Y= 0,<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref>  +1,<sup>?</sup> +2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Zr= +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2,<ref name="Calderazzo">{{Cite journal |last=Calderazzo |first=Fausto |last2=Pampaloni |first2=Guido |date=January 1992 |title=Organometallics of groups 4 and 5: Oxidation states II and lower |url=https://linkinghub.elsevier.com/retrieve/pii/0022328X92831263 |journal=Journal of Organometallic Chemistry |language=en |volume=423 |issue=3 |pages=307–328 |doi=10.1016/0022-328X(92)83126-3}}</ref><ref name="Ma">{{Cite journal |last1=Ma |first1=Wen |last2=Herbert |first2=F. William |last3=Senanayake |first3=Sanjaya D. |last4=Yildiz |first4=Bilge |date=2015-03-09 |title=Non-equilibrium oxidation states of zirconium during early stages of metal oxidation |url=https://pubs.aip.org/apl/article/106/10/101603/236409/Non-equilibrium-oxidation-states-of-zirconium |journal=Applied Physics Letters |language=en |volume=106 |issue=10 |doi=10.1063/1.4914180 |bibcode=2015ApPhL.106j1603M |issn=0003-6951|hdl=1721.1/104888 |hdl-access=free }}</ref> +3<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Nb= −3,<sup>?</sup> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<sup>?</sup> +1,<sup>?</sup> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Mo= −4,<sup>?</sup> −2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<sup>?</sup> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Tc= −3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +6<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Ru= −4,<sup>?</sup> −2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>  +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +6,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +7,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +8<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Rh= −3,<ref>Ellis J E. Highly Reduced Metal Carbonyl Anions: Synthesis, Characterization, and Chemical Properties. Adv. Organomet. Chem, 1990, 31: 1-51.</ref> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +6,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +7<ref>Rh(VII) is known in the RhO<sub>3</sub><sup>+</sup> cation, see {{cite journal |title=The Highest Oxidation State of Rhodium: Rhodium(VII) in [RhO3]+ |journal=Angew. Chem. Int. Ed. |date=2022 |doi=10.1002/anie.202207688|last1=Da Silva Santos |first1=Mayara |last2=Stüker |first2=Tony |last3=Flach |first3=Max |last4=Ablyasova |first4=Olesya S. |last5=Timm |first5=Martin |last6=von Issendorff |first6=Bernd |last7=Hirsch |first7=Konstantin |last8=Zamudio‐Bayer |first8=Vicente |last9=Riedel |first9=Sebastian |last10=Lau |first10=J. Tobias |volume=61 |issue=38 |pages=e202207688 |pmid=35818987 |pmc=9544489 }}</ref>
| Pd= +1,<sup>?</sup> +3,<sup>?</sup> +5<ref>Palladium(V) has been identified in complexes with organosilicon compounds containing pentacoordinate palladium; see {{cite journal |first1=Shigeru |last1=Shimada |first2=Yong-Hua |last2=Li |first3=Yoong-Kee |last3=Choe |first4=Masato |last4=Tanaka |first5=Ming |last5=Bao |first6=Tadafumi |last6=Uchimaru |title=Multinuclear palladium compounds containing palladium centers ligated by five silicon atoms |doi=10.1073/pnas.0700450104 |journal=Proceedings of the National Academy of Sciences |volume=104 |year=2007 |issue=19 |pages=7758–7763|pmid=17470819 |pmc=1876520 |doi-access=free }}</ref>
| Ag= −2,<sup>?</sup> −1,<sup>?</sup> 0,<ref>Ag(0) has been observed in carbonyl complexes in low-temperature matrices: see {{cite journal|doi=10.1021/ja00427a018|title=Synthesis using metal vapors. Silver carbonyls. Matrix infrared, ultraviolet-visible, and electron spin resonance spectra, structures, and bonding of silver tricarbonyl, silver dicarbonyl, silver monocarbonyl, and disilver hexacarbonyl|year=1976|last1=McIntosh|first1=D.|last2=Ozin|first2=G. A.|journal=J. Am. Chem. Soc.|volume=98|issue=11|pages=3167–75}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Cd= −2,<sup>?</sup> +1<sup>?</sup>
| In= −5,<sup>?</sup> −2,<sup>?</sup> −1,<sup>?</sup> 0,<ref>Unstable In(0) carbonyls and clusters have been detected, see [https://www.researchgate.net/profile/Anthony-Downs-2/publication/6589844_Development_of_the_Chemistry_of_Indium_in_Formal_Oxidation_States_Lower_than_3/links/5a82db2a0f7e9bda869fb52c/Development-of-the-Chemistry-of-Indium-in-Formal-Oxidation-States-Lower-than-3.pdf?origin=publication_detail], p. 6.</ref> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> <ref>{{cite journal|doi=10.1021/ic951378e|title=Synthesis, Structure, and Bonding of Two Lanthanum Indium Germanides with Novel Structures and Properties|year=1996|last1=Guloy|first1=A. M.|last2=Corbett|first2=J. D.|journal=Inorganic Chemistry|volume=35|issue=9|pages=2616–22|pmid=11666477}}</ref>
| Sn= −3,<sup>?</sup> −2,<sup>?</sup> −1,<sup>?</sup> 0,<ref name=ZeroValentTin3>{{cite web |title=New Type of Zero-Valent Tin Compound |url= https://www.chemistryviews.org/details/news/9745121/New_Type_of_Zero-Valent_Tin_Compound.html |publisher=[[Chemistry Europe]] |date=27 August 2016}}</ref> +1,<ref>{{cite web|title=HSn|url=http://webbook.nist.gov/cgi/cbook.cgi?ID=C13940255&Units=SI|work=NIST Chemistry WebBook|publisher=National Institute of Standards and Technology|access-date=23 January 2013}}</ref> +3<ref>{{cite web|title=SnH3|url=http://webbook.nist.gov/cgi/cbook.cgi?ID=B1001467&Units=SI|work=NIST Chemistry WebBook|publisher=National Institure of Standards and Technology|access-date=23 January 2013}}</ref>
| Sb= −2,<sup>?</sup> −1,<sup>?</sup> 0,<ref>{{cite news|url=https://pdfs.semanticscholar.org/8817/39f9dfc007d7f77dd7baa63fe12e6079f8ef.pdf|author=Anastas Sidiropoulos|title=Studies of N-heterocyclic Carbene (NHC) Complexes of the Main Group Elements|year=2019 |page=39|doi=10.4225/03/5B0F4BDF98F60|s2cid=132399530}}
</ref> +1,<sup>?</sup> +2,<sup>?</sup> +4<sup>?</sup>
| Te= −1,<sup>?</sup> 0,<sup>?</sup> +1,<sup>?</sup> +3,<sup>?</sup> +5<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| I= +2,<ref>I(II) is known to exist in monoxide (IO); see {{cite journal|last1=Nikitin|first1=I V|title=Halogen monoxides|journal=Russian Chemical Reviews|date=31 August 2008|volume=77|issue=8|pages=739–749|doi=10.1070/RC2008v077n08ABEH003788|bibcode=2008RuCRv..77..739N|s2cid=250898175 }}</ref> +4,<sup>?</sup> +6<sup>?</sup>
| Xe= +8<ref name="Harding-2002">{{cite book|author=Harding, Charlie|author2=Johnson, David Arthur|author3= Janes, Rob |title = Elements of the ''p'' block |pages=93–94|publisher=Royal Society of Chemistry |location=Great Britain|date=2002 |isbn=0-85404-690-9 |url= https://books.google.com/books?id=W0HW8wgmQQsC&pg=PA93}}</ref>
<!--- Period 6 --->
| Cs=−1<ref name="caeside2">{{cite journal|journal = [[Angewandte Chemie|Angewandte Chemie International Edition]]|year = 1979|first = J. L.|last = Dye|title = Compounds of Alkali Metal Anions|volume = 18|issue = 8|pages = 587–598|doi = 10.1002/anie.197905871}}</ref>
| Ba=+1<sup>?</sup>
| La= 0,<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref> +1,<ref name=LnI>La(I), Pr(I), Tb(I), Tm(I), and Yb(I) have been observed in MB<sub>8</sub><sup>−</sup> clusters; see {{cite journal|title=Monovalent lanthanide(I) in borozene complexes |journal=Nature Communications |volume=12 |page=6467 |year=2021 |last1=Li |first1=Wan-Lu |doi=10.1038/s41467-021-26785-9 |last2=Chen |first2=Teng-Teng |last3=Chen |first3=Wei-Jia |last4=Li |first4=Jun |last5=Wang |first5=Lai-Sheng|issue=1 |pmid=34753931 |pmc=8578558 }}</ref> +2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Ce= +1,<sup>?</sup> +2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Pr= 0,<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref> +1,<ref>{{cite journal|title=Lanthanides with Unusually Low Oxidation States in the PrB<sup>3–</sup> and PrB<sup>4–</sup> Boride Clusters|journal=Inorganic Chemistry|volume=58|issue=1|pages=411–418|last1=Chen|first1=Xin|display-authors=etal|date=2019-12-13|doi=10.1021/acs.inorgchem.8b02572|pmid=30543295|s2cid=56148031 }}</ref> +2,<sup>?</sup> +4,<sup>?</sup> +5
| Nd= 0,<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4
| Pm= +2<sup>?</sup>
| Sm= 0,<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref> +1,<ref>SmB<sub>6</sub><sup>-</sup> cluster anion has been reported and contains Sm in rare oxidation state of +1; see {{cite journal| title=SmB<sub>6</sub><sup>–</sup> Cluster Anion: Covalency Involving f Orbitals
|first1=J. Robinson |last1=Paul |first2=Zhang |last2=Xinxing |first3=McQueen |last3=Tyrel |first4=H. Bowen |last4=Kit |first5=N. Alexandrova |last5=Anastassia |journal=J. Phys. Chem. A 2017,<sup>?</sup> 121,<sup>?</sup> 8,<sup>?</sup> 1849–1854 |year = 2017|volume = 121|issue = 8|pages = 1849–1854|doi=10.1021/acs.jpca.7b00247 |pmid=28182423 |s2cid=3723987 |url=https://pubs.acs.org/doi/abs/10.1021/acs.jpca.7b00247#}}.</ref> +2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Eu= 0<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref>
| Gd= 0,<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Tb= 0,<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref> +1,<ref name=LnI>La(I), Pr(I), Tb(I), Tm(I), and Yb(I) have been observed in MB<sub>8</sub><sup>−</sup> clusters; see {{cite journal|title=Monovalent lanthanide(I) in borozene complexes |journal=Nature Communications |volume=12 |page=6467 |year=2021 |last1=Li |first1=Wan-Lu |doi=10.1038/s41467-021-26785-9 |last2=Chen |first2=Teng-Teng |last3=Chen |first3=Wei-Jia |last4=Li |first4=Jun |last5=Wang |first5=Lai-Sheng|issue=1 |pmid=34753931 |pmc=8578558 }}</ref> +2,<sup>?</sup> +4<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Dy= 0,<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref> +1,<sup>?</sup> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4
| Ho= 0,<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref> +1,<sup>?</sup> +2<sup>?</sup>
| Er= 0,<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref> +1,<sup>?</sup> +2<sup>?</sup>
| Tm= 0,<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref> +1,<ref name=LnI>La(I), Pr(I), Tb(I), Tm(I), and Yb(I) have been observed in MB<sub>8</sub><sup>−</sup> clusters; see {{cite journal|title=Monovalent lanthanide(I) in borozene complexes |journal=Nature Communications |volume=12 |page=6467 |year=2021 |last1=Li |first1=Wan-Lu |doi=10.1038/s41467-021-26785-9 |last2=Chen |first2=Teng-Teng |last3=Chen |first3=Wei-Jia |last4=Li |first4=Jun |last5=Wang |first5=Lai-Sheng|issue=1 |pmid=34753931 |pmc=8578558 }}</ref> +2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Yb= 0,<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref> +1,<ref name=LnI>La(I), Pr(I), Tb(I), Tm(I), and Yb(I) have been observed in MB<sub>8</sub><sup>−</sup> clusters; see {{cite journal|title=Monovalent lanthanide(I) in borozene complexes |journal=Nature Communications |volume=12 |page=6467 |year=2021 |last1=Li |first1=Wan-Lu |doi=10.1038/s41467-021-26785-9 |last2=Chen |first2=Teng-Teng |last3=Chen |first3=Wei-Jia |last4=Li |first4=Jun |last5=Wang |first5=Lai-Sheng|issue=1 |pmid=34753931 |pmc=8578558 }}</ref> +2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Lu= 0,<ref name="Cloke1993">Yttrium and all lanthanides except Ce and Pm have been observed in the oxidation state 0 in bis(1,3,5-tri-t-butylbenzene) complexes, see {{cite journal |journal=Chem. Soc. Rev. |date=1993 |volume=22 |pages=17–24 |first=F. Geoffrey N. |last=Cloke |title=Zero Oxidation State Compounds of Scandium, Yttrium, and the Lanthanides |doi=10.1039/CS9932200017}} and {{cite journal|last1=Arnold|first1=Polly L.|last2=Petrukhina|first2=Marina A.|last3=Bochenkov|first3=Vladimir E.|last4=Shabatina|first4=Tatyana I.|last5=Zagorskii|first5=Vyacheslav V.|last6=Cloke|first9=F. Geoffrey N.|date=2003-12-15|title=Arene complexation of Sm, Eu, Tm and Yb atoms: a variable temperature spectroscopic investigation|journal=Journal of Organometallic Chemistry|volume=688|issue=1–2|pages=49–55|doi=10.1016/j.jorganchem.2003.08.028}}</ref> +1,<sup>?</sup> +2<sup>?</sup>
| Hf= −2,<sup>?</sup> 0,<sup>?</sup> +1,<sup>?</sup> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>, +3<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Ta= −3,<sup>?</sup> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<sup>?</sup> +1,<sup>?</sup> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| W= −4,<sup>?</sup> −2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<sup>?</sup> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Re= −3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0,<sup>?</sup> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>, +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +6,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +7<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Os= −4,<sup>?</sup> −2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −1,<sup>?</sup> 0,<sup>?</sup> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +6,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +7,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +8<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Ir= −3,<sup>?</sup> −2,<sup>?</sup> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +6,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +7,<sup>?</sup> +8,<sup>?</sup> +9<ref name=IrIX>{{cite journal |last1=Wang |first1=Guanjun |last2=Zhou |first2=Mingfei |last3=Goettel |first3=James T. |last4=Schrobilgen |first4=Gary G. |last5=Su |first5=Jing |last6=Li |first6=Jun |last7=Schlöder |first7=Tobias |last8=Riedel |first8=Sebastian |date=2014 |title=Identification of an iridium-containing compound with a formal oxidation state of IX |journal=Nature |volume=514 |issue=7523 |pages=475–477 |doi=10.1038/nature13795 |pmid=25341786|bibcode=2014Natur.514..475W |s2cid=4463905 }}</ref>
| Pt= −3,<sup>?</sup> −2,<sup>?</sup> −1,<sup>?</sup> 0,<sup>?</sup> +1,<sup>?</sup> +3,<sup>?</sup> +5,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +6<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Au= −3,<sup>?</sup> −2,<sup>?</sup> −1,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> 0, +1,<ref>{{cite journal |doi= 10.1002/(SICI)1521-3773(19991102)38:21<3194::AID-ANIE3194>3.0.CO;2-O |title= Gold(I) and Gold(0) Complexes of Phosphinine‐Based Macrocycles|first1=Nicolas |last1= Mézaille |first2=Narcis |last2=Avarvari |first3=Nicole |last3=Maigrot|first4=Louis |last4=Ricard|first5=François |last5=Mathey|first6=Pascal |last6=Le Floch|first7=Laurent|last7=Cataldo|first8=Théo|last8=Berclaz|first9=Michel|last9=Geoffroy|journal= Angewandte Chemie International Edition|year= 1999 |volume= 38|issue= 21|pages= 3194–3197|pmid= 10556900}}</ref> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Hg= −2<ref name="Brauer-1936">{{Cite journal |last=Brauer |first=G. |last2=Haucke |first2=W. |date=1936-06-01 |title=Kristallstruktur der intermetallischen Phasen MgAu und MgHg |url=https://www.degruyter.com/document/doi/10.1515/zpch-1936-3327/html |journal=Zeitschrift für Physikalische Chemie |language=en |volume=33B |issue=1 |pages=304–310 |doi=10.1515/zpch-1936-3327 |issn=2196-7156 |quote=MgHg then lends itself to an oxidation state of +2 for Mg and -2 for Hg because it consists entirely of these polar bonds with no evidence of electron unpairing. (translated)}}</ref>
| Tl= −5,<ref>{{cite journal|doi=10.1021/ic960014z|title=Na<sub>23</sub>K<sub>9</sub>Tl<sub>15.3</sub>: An Unusual Zintl Compound Containing Apparent Tl<sub>5</sub><sup>7−</sup>, Tl<sub>4</sub><sup>8−</sup>, Tl<sub>3</sub><sup>7−</sup>, and Tl<sup>5−</sup> Anions|year=1996|last1=Dong|first1=Z.-C.|last2=Corbett|first2=J. D.|journal=Inorganic Chemistry|volume=35|issue=11|pages=3107–12|pmid=11666505 }}</ref><!-- is about a Tl 7- then? --> −2,<sup>?</sup> −1,<sup>?</sup> +2<sup>?</sup>
| Pb= −4,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −2,<sup>?</sup> −1,<sup>?</sup> 0,<ref>Pb(0) carbonyls have been observered in reaction between lead atoms and [[carbon monoxide]]; see {{cite journal|url=https://aip.scitation.org/doi/10.1063/1.1834915 | title= Observation of the lead carbonyls Pb<sub>n</sub>CO (n=1–4): Reactions of lead atoms and small clusters with carbon monoxide in solid argon
|first1=Jiang |last1=Ling |first2=Xu |last2=Qiang |journal=The Journal of Chemical Physics. 122 (3): 034505 |year = 2005|volume = 122|issue = 3|page = 34505|doi=10.1063/1.1834915 |pmid = 15740207|bibcode = 2005JChPh.122c4505J|issn=0021-9606}}</ref> +1,<sup>?</sup> +3<sup>?</sup>
| Bi= −3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> −2,<sup>?</sup> −1,<sup>?</sup> 0,<ref>Bi(0) state exists in a [[Heterocyclic compound|N-heterocyclic carbene]] complex of dibismuthene; see {{cite journal |first1=Rajesh |last1=Deka |first2=Andreas |last2=Orthaber |title=Carbene chemistry of arsenic, antimony, and bismuth: origin, evolution and future prospects |journal=Royal Society of Chemistry |issue=22 |date=May 9, 2022 |volume=51 |pages=8540–8556 |doi=10.1039/d2dt00755j |pmid=35578901|s2cid=248675805 }}</ref> +1,<sup>?</sup> +2,<sup>?</sup> +4,<sup>?</sup> +5<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Po= +5<ref name="Thayer p78">{{cite journal |last1=Thayer |first1=John S. |journal=Relativistic Methods for Chemists |title=Relativistic Effects and the Chemistry of the Heavier Main Group Elements |series=Challenges and Advances in Computational Chemistry and Physics |year=2010 |volume=10 |page=78 |doi=10.1007/978-1-4020-9975-5_2|isbn=978-1-4020-9974-8 }}</ref> +6,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| At= +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +7<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Rn= +2,<sup>?</sup> +6
<!--- Period 7 --->
| Fr=
| Ra=
| Ac=
| Th=−1,<ref name="Th(-1), U(-1)">Th(-I) and U(-I) have been detected in the gas phase as octacarbonyl anions; see {{cite journal| title=Octacarbonyl Ion Complexes of Actinides [An(CO)<sub>8</sub>]<sup>+/−</sup> (An=Th, U) and the Role of f Orbitals in Metal–Ligand Bonding
|first1=Chi |last1=Chaoxian |first2=Pan |last2=Sudip |first3=Jin |last3=Jiaye |first4=Meng |last4=Luyan |first5=Luo |last5=Mingbiao |first6=Zhao |last6=Lili |first7=Zhou |last7=Mingfei |first8=Frenking |last8=Gernot |journal=Chemistry (Weinheim an der Bergstrasse, Germany). 25 (50): 11772–11784 |year = 2019|volume = 25|issue = 50|pages = 11772–11784|doi=10.1002/chem.201902625 |issn=0947-6539 |pmc=6772027
|pmid=31276242}}</ref> +1,<sup>?</sup> +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +3<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Pa=+2,<sup>?</sup> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| U=−1,<ref name="Th(-1), U(-1)">Th(-I) and U(-I) have been detected in the gas phase as octacarbonyl anions; see {{cite journal| title=Octacarbonyl Ion Complexes of Actinides [An(CO)<sub>8</sub>]<sup>+/−</sup> (An=Th, U) and the Role of f Orbitals in Metal–Ligand Bonding
|first1=Chi |last1=Chaoxian |first2=Pan |last2=Sudip |first3=Jin |last3=Jiaye |first4=Meng |last4=Luyan |first5=Luo |last5=Mingbiao |first6=Zhao |last6=Lili |first7=Zhou |last7=Mingfei |first8=Frenking |last8=Gernot |journal=Chemistry (Weinheim an der Bergstrasse, Germany). 25 (50): 11772–11784 |year = 2019|volume = 25|issue = 50|pages = 11772–11784|doi=10.1002/chem.201902625 |issn=0947-6539 |pmc=6772027
|pmid=31276242}}</ref> +1,<sup>?</sup> +2,<sup>?</sup> +3,<ref>{{cite book|title=The Chemistry of the Actinide and Transactinide Elements|edition=3rd|editor1=Morss, L.R. |editor2=Edelstein, N.M. |editor3=Fuger, J. |place=Netherlands|publisher=Springer|year=2006|isbn=978-9048131464}}</ref> +4,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Np= +2,<sup>?</sup> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4,<ref name="Dutkiewicz2017">Np(II), (III) and (IV) have been observed, see {{cite journal|doi=10.1039/C7SC00034K|pmid=28553487|pmc=5431675|year=2017|title=Reduction chemistry of neptunium cyclopentadienide complexes: from structure to understanding|first1=Michał S.|last1=Dutkiewicz|first2=Christos|last2=Apostolidis|first3=Olaf|last3=Walter|first4=Polly L|last4=Arnold|volume=8|issue=4|journal=Chem. Sci.|pages=2553–2561}}</ref> +6,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +7<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Pu= +2,<sup>?</sup> +3,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>, +5,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +6,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +7,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +8
| Am= +2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +6,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +7
| Cm= +4,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5,<ref name="Kovács2018">{{cite journal |last1=Kovács |first1=Attila |last2=Dau |first2=Phuong D. |last3=Marçalo |first3=Joaquim |last4=Gibson |first4=John K. |date=2018 |title=Pentavalent Curium, Berkelium, and Californium in Nitrate Complexes: Extending Actinide Chemistry and Oxidation States |journal=Inorg. Chem. |volume=57 |issue=15 |pages=9453–9467 |publisher=American Chemical Society |doi=10.1021/acs.inorgchem.8b01450 |pmid=30040397 |osti=1631597 |s2cid=51717837 }}</ref> +6<ref name="CmO3">{{cite journal |last1=Domanov |first1=V. P. |last2=Lobanov |first2=Yu. V. |date=October 2011 |title=Formation of volatile curium(VI) trioxide CmO<sub>3</sub> |journal=Radiochemistry |volume=53 |issue=5 |pages=453–6 |publisher=SP MAIK Nauka/Interperiodica |doi=10.1134/S1066362211050018|s2cid=98052484 }}</ref>
| Bk=+2,<sup>?</sup> +4,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5<ref name="Kovács2018">{{cite journal |last1=Kovács |first1=Attila |last2=Dau |first2=Phuong D. |last3=Marçalo |first3=Joaquim |last4=Gibson |first4=John K. |date=2018 |title=Pentavalent Curium, Berkelium, and Californium in Nitrate Complexes: Extending Actinide Chemistry and Oxidation States |journal=Inorg. Chem. |volume=57 |issue=15 |pages=9453–9467 |publisher=American Chemical Society |doi=10.1021/acs.inorgchem.8b01450 |pmid=30040397 |osti=1631597 |s2cid=51717837 }}</ref>
| Cf=+2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +5<ref>{{Greenwood&Earnshaw2nd|page=1265}}</ref><ref name="Kovács2018">{{cite journal |last1=Kovács |first1=Attila |last2=Dau |first2=Phuong D. |last3=Marçalo |first3=Joaquim |last4=Gibson |first4=John K. |date=2018 |title=Pentavalent Curium, Berkelium, and Californium in Nitrate Complexes: Extending Actinide Chemistry and Oxidation States |journal=Inorg. Chem. |volume=57 |issue=15 |pages=9453–9467 |publisher=American Chemical Society |doi=10.1021/acs.inorgchem.8b01450 |pmid=30040397 |osti=1631597 |s2cid=51717837 }}</ref>
| Es=+2,<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref> +4
| Fm=+2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Md=+2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| No=+2<ref name=GE28>{{Greenwood&Earnshaw2nd|page=28}}</ref>
| Lr=
<!-- default to blank -->
}}
|predicted={{#switch:{{{symbol|}}}
| Rf=(+3), (+4)<!-- source for 3,4 --><ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref>
| Db= (+3), (+4), (+5)<!-- source for 3,4,5 --><ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref>
| Sg= (+3), (+4), (+5), (+6)<!-- source for 3,4,5,6 --><ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref>
| Bh= (+3), (+4), (+5), (+7)<!-- source for 3,4,5,7 --><ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref>
| Hs= (+3), (+4), (+6), (+8)<!-- source for 3,4,6,8 --><ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref>
| Mt= (+1), (+3), (+6)<!-- source for 1,3,6 --><ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref>
| Ds= (+2), (+4), (+6)<!-- source for 2,4,6 --><ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref>
| Rg= (−1), (+3), (+5)<!-- source for -1,+3,+5 --><ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref>
| Cn= (+2), (+4)<!-- source for 2,4 --><ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref>
| Nh=
| Fl=
| Mc=
| Lv= (−2),<ref name="Thayer p83">{{cite journal |last1=Thayer |first1=John S. |journal=Relativistic Methods for Chemists |title=Relativistic Effects and the Chemistry of the Heavier Main Group Elements |series=Challenges and Advances in Computational Chemistry and Physics |year=2010 |volume=10 |page=83 |doi=10.1007/978-1-4020-9975-5_2|isbn=978-1-4020-9974-8 }}</ref>   (+4)
| Ts= (−1), (+5)
| Og= (−1),<ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref> (+1),<ref name=hydride>{{cite journal|journal=Journal of Chemical Physics |volume=112|issue=6|year=2000|title=Spin–orbit effects on the transactinide p-block element monohydrides MH (M=element 113–118)|first1=Young-Kyu|last1=Han|first2=Cheolbeom |last2=Bae|first3=Sang-Kil |last3=Son|first4=Yoon Sup|last4=Lee|doi=10.1063/1.480842|page=2684|bibcode=2000JChPh.112.2684H}}</ref> (+2),<ref name=Kaldor/> (+4),<ref name=Kaldor>{{cite book|title=Theoretical Chemistry and Physics of Heavy and Superheavy Elements|first1=Uzi |last1=Kaldor |first2=Stephen |last2=Wilson |page=105 |year=2003 |publisher=Springer|isbn=978-1402013713|url=https://books.google.com/books?id=0xcAM5BzS-wC&q=element+118+properties|access-date=2008-01-18}}</ref> (+6)<ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref>
<!--- Period 8 --->
| Uue= (+1), (+3), (+5)<ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref><ref name=Cao>{{cite journal |last1=Cao |first1=Chang-Su |last2=Hu |first2=Han-Shi |last3=Schwarz |first3=W. H. Eugen |last4=Li |first4=Jun |date=2022 |title=Periodic Law of Chemistry Overturns for Superheavy Elements |type=preprint |url=https://chemrxiv.org/engage/chemrxiv/article-details/63730be974b7b6d84cfdda35 |journal=[[ChemRxiv]] |volume= |issue= |pages= |doi=10.26434/chemrxiv-2022-l798p |access-date=16 November 2022}}</ref>
| Ubn= (+2),<ref name=Thayer>{{cite journal |last1=Thayer |first1=John S. |journal=Relativistic Methods for Chemists |title=Relativistic Effects and the Chemistry of the Heavier Main Group Elements |series=Challenges and Advances in Computational Chemistry and Physics |year=2010 |volume=10 |page=84 |doi=10.1007/978-1-4020-9975-5_2|isbn=978-1-4020-9974-8 }}</ref> (+4), (+6)<ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref><ref name=Cao>{{cite journal |last1=Cao |first1=Chang-Su |last2=Hu |first2=Han-Shi |last3=Schwarz |first3=W. H. Eugen |last4=Li |first4=Jun |date=2022 |title=Periodic Law of Chemistry Overturns for Superheavy Elements |type=preprint |url=https://chemrxiv.org/engage/chemrxiv/article-details/63730be974b7b6d84cfdda35 |journal=[[ChemRxiv]] |volume= |issue= |pages= |doi=10.26434/chemrxiv-2022-l798p |access-date=16 November 2022}}</ref>
| Ubu= (+3)<ref name=Haire>{{cite book| title=The Chemistry of the Actinide and Transactinide Elements| editor1-last=Morss| editor2-first=Norman M.| editor2-last=Edelstein| editor3-last=Fuger| editor3-first=Jean| last1=Hoffman| first1=Darleane C.| last2=Lee| first2=Diana M.| last3=Pershina| first3=Valeria| chapter=Transactinides and the future elements| publisher=[[Springer Science+Business Media]]| year=2006| isbn=978-1-4020-3555-5| location=Dordrecht, The Netherlands| edition=3rd| ref=CITEREFHaire2006}}</ref><ref name=Amador>{{cite journal |last1=Amador |first1=Davi H. T. |last2=de Oliveira |first2=Heibbe C. B. |first3=Julio R. |last3=Sambrano |first4=Ricardo |last4=Gargano |first5=Luiz Guilherme M. |last5=de Macedo |date=12 September 2016 |title=4-Component correlated all-electron study on Eka-actinium Fluoride (E121F) including Gaunt interaction: Accurate analytical form, bonding and influence on rovibrational spectra |journal=Chemical Physics Letters |volume=662 |pages=169–175 |doi=10.1016/j.cplett.2016.09.025|bibcode=2016CPL...662..169A |hdl=11449/168956 }}</ref>
| Ubb= (+4)<ref name="Pyykkö2011">{{Cite journal|last1=Pyykkö|first1=Pekka|author-link=Pekka Pyykkö|title=A suggested periodic table up to Z ≤ 172, based on Dirac–Fock calculations on atoms and ions|journal=Physical Chemistry Chemical Physics|volume=13|issue=1|pages=161–8|year=2011|pmid=20967377|doi=10.1039/c0cp01575j|bibcode = 2011PCCP...13..161P }}</ref>
| Ubt=
| Ubq= (+6)<ref name="Pyykkö2011">{{Cite journal|last1=Pyykkö|first1=Pekka|author-link=Pekka Pyykkö|title=A suggested periodic table up to Z ≤ 172, based on Dirac–Fock calculations on atoms and ions|journal=Physical Chemistry Chemical Physics|volume=13|issue=1|pages=161–8|year=2011|pmid=20967377|doi=10.1039/c0cp01575j|bibcode = 2011PCCP...13..161P }}</ref>
| Ubp=(+6), (+7)<ref name="Pyykkö2011">{{Cite journal|last1=Pyykkö|first1=Pekka|author-link=Pekka Pyykkö|title=A suggested periodic table up to Z ≤ 172, based on Dirac–Fock calculations on atoms and ions|journal=Physical Chemistry Chemical Physics|volume=13|issue=1|pages=161–8|year=2011|pmid=20967377|doi=10.1039/c0cp01575j|bibcode = 2011PCCP...13..161P }}</ref><!-- for +6 and +7-->
| Ubh=(+4), (+6), (+8)<ref name="Pyykkö2011">{{Cite journal|last1=Pyykkö|first1=Pekka|author-link=Pekka Pyykkö|title=A suggested periodic table up to Z ≤ 172, based on Dirac–Fock calculations on atoms and ions|journal=Physical Chemistry Chemical Physics|volume=13|issue=1|pages=161–8|year=2011|pmid=20967377|doi=10.1039/c0cp01575j|bibcode = 2011PCCP...13..161P }}</ref><!-- for +4 +6 +8-->
| Ubo=(){{Infobox element/symbol-to-oxidation-state/comment|comment=predicted|engvar={{{engvar|}}}}}<ref name="Pyykkö2011">{{Cite journal|last1=Pyykkö|first1=Pekka|author-link=Pekka Pyykkö|title=A suggested periodic table up to Z ≤ 172, based on Dirac–Fock calculations on atoms and ions|journal=Physical Chemistry Chemical Physics|volume=13|issue=1|pages=161–8|year=2011|pmid=20967377|doi=10.1039/c0cp01575j|bibcode = 2011PCCP...13..161P }}</ref>
| Ube=(), (){{Infobox element/symbol-to-oxidation-state/comment|comment=predicted|engvar={{{engvar|}}}}}<ref name="Pyykkö2011">{{Cite journal|last1=Pyykkö|first1=Pekka|author-link=Pekka Pyykkö|title=A suggested periodic table up to Z ≤ 172,<sup>?</sup> based on Dirac–Fock calculations on atoms and ions|journal=Physical Chemistry Chemical Physics|volume=13|issue=1|pages=161–8|year=2011|pmid=20967377|doi=10.1039/c0cp01575j|bibcode = 2011PCCP...13..161P }}</ref>
| Uts=

<!-- default to blank -->
}}
}}<!--

--><noinclude>{{documentation}}</noinclude>
"#;
