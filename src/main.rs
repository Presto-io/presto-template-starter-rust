use std::io::{self, Read, Write, BufWriter};

use clap::Parser;
use pulldown_cmark::{Event, Tag, TagEnd, HeadingLevel};
use serde::Deserialize;

const MANIFEST: &str = include_str!("../manifest.json");
const EXAMPLE: &str = include_str!("../example.md");

#[derive(Parser)]
#[command(about = "Presto template: Markdown → Typst converter")]
struct Cli {
    /// Output embedded manifest.json
    #[arg(long)]
    manifest: bool,

    /// Output embedded example.md
    #[arg(long)]
    example: bool,

    /// Output version from manifest
    #[arg(long = "version")]
    version_flag: bool,
}

#[derive(Debug, Default, Deserialize)]
struct Frontmatter {
    #[serde(default)]
    title: String,
}

fn main() {
    let cli = Cli::parse();

    if cli.manifest {
        print!("{}", MANIFEST);
        return;
    }
    if cli.version_flag {
        let manifest: serde_json::Value = serde_json::from_str(MANIFEST).unwrap();
        if let Some(v) = manifest.get("version") {
            println!("{}", v.as_str().unwrap_or("unknown"));
        }
        return;
    }
    if cli.example {
        print!("{}", EXAMPLE);
        return;
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("error reading stdin");

    let (fm_str, body) = split_frontmatter(&input);

    let meta: Frontmatter = if fm_str.is_empty() {
        Frontmatter::default()
    } else {
        serde_yaml::from_str(fm_str).expect("error parsing frontmatter")
    };

    let stdout = io::stdout();
    let mut w = BufWriter::new(stdout.lock());

    write_page_setup(&mut w, &meta);
    render_body(&mut w, body);

    w.flush().expect("error flushing output");
}

/// Separates YAML frontmatter (between --- delimiters) from the body.
fn split_frontmatter(input: &str) -> (&str, &str) {
    if !input.starts_with("---\n") && !input.starts_with("---\r\n") {
        return ("", input);
    }

    let rest = if input.starts_with("---\r\n") {
        &input[5..]
    } else {
        &input[4..]
    };

    if let Some(idx) = rest.find("\n---") {
        let fm = &rest[..idx];
        let after = &rest[idx + 4..]; // skip "\n---"
        let body = if after.starts_with('\n') {
            &after[1..]
        } else if after.starts_with("\r\n") {
            &after[2..]
        } else {
            after
        };
        (fm, body)
    } else {
        ("", input)
    }
}

/// Outputs the Typst page setup and metadata.
fn write_page_setup(w: &mut impl Write, meta: &Frontmatter) {
    writeln!(w, r#"#set page(paper: "a4")"#).unwrap();
    writeln!(w, r#"#set text(font: "SimSun", size: 12pt, lang: "zh")"#).unwrap();
    writeln!(w, r#"#set par(leading: 1.5em, first-line-indent: 2em)"#).unwrap();
    writeln!(w).unwrap();

    if !meta.title.is_empty() {
        writeln!(w, r#"#let title = "{}""#, meta.title).unwrap();
        writeln!(w).unwrap();
        writeln!(
            w,
            r#"#align(center, text(size: 22pt, weight: "bold")[{}])"#,
            meta.title
        )
        .unwrap();
        writeln!(w, r#"#v(1em)"#).unwrap();
        writeln!(w).unwrap();
    }
}

/// Parses Markdown body using pulldown-cmark and outputs Typst.
fn render_body(w: &mut impl Write, source: &str) {
    let parser = pulldown_cmark::Parser::new(source);

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let n = heading_level_to_u8(level);
                write!(w, "#heading(level: {})[", n).unwrap();
            }
            Event::End(TagEnd::Heading(_)) => {
                writeln!(w, "]").unwrap();
                writeln!(w).unwrap();
            }

            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                writeln!(w).unwrap();
                writeln!(w).unwrap();
            }

            Event::Text(text) => {
                write!(w, "{}", text).unwrap();
            }
            Event::SoftBreak => {
                writeln!(w).unwrap();
            }

            Event::Start(Tag::List(_)) => {}
            Event::End(TagEnd::List(_)) => {
                writeln!(w).unwrap();
            }

            Event::Start(Tag::Item) => {
                write!(w, "- ").unwrap();
            }
            Event::End(TagEnd::Item) => {
                writeln!(w).unwrap();
            }

            Event::Start(Tag::Emphasis) => {
                write!(w, "#emph[").unwrap();
            }
            Event::End(TagEnd::Emphasis) => {
                write!(w, "]").unwrap();
            }

            Event::Start(Tag::Strong) => {
                write!(w, "#strong[").unwrap();
            }
            Event::End(TagEnd::Strong) => {
                write!(w, "]").unwrap();
            }

            Event::Rule => {
                writeln!(w, "#line(length: 100%)").unwrap();
                writeln!(w).unwrap();
            }

            Event::Code(text) => {
                write!(w, r#"#raw("{}")"#, text).unwrap();
            }

            Event::Start(Tag::CodeBlock(_)) => {
                write!(w, "```\n").unwrap();
            }
            Event::End(TagEnd::CodeBlock) => {
                writeln!(w, "```").unwrap();
                writeln!(w).unwrap();
            }

            _ => {}
        }
    }
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}
