use std::io::{self, Read, Write};

use clap::Parser;
use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};
use serde::{Deserialize, Serialize};

const MANIFEST: &str = include_str!("../manifest.json");
const EXAMPLE: &str = include_str!("../example.md");
const MAX_INPUT_BYTES: u64 = 10 * 1024 * 1024;

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

    /// Output document info JSON
    #[arg(long)]
    info: bool,
}

#[derive(Debug, Default, Deserialize)]
struct Frontmatter {
    #[serde(default)]
    title: String,
}

#[derive(Debug, Serialize)]
struct OutputInfo {
    #[serde(rename = "schemaVersion")]
    schema_version: u8,
    #[serde(rename = "outputBaseName")]
    output_base_name: String,
    #[serde(rename = "previewTitle")]
    preview_title: String,
    document: DocumentInfo,
}

#[derive(Debug, Serialize)]
struct DocumentInfo {
    title: String,
    keywords: Vec<String>,
    language: String,
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

    let input = match read_stdin_limited() {
        Ok(input) => input,
        Err(err) => {
            eprintln!("error reading stdin: {err}");
            std::process::exit(1);
        }
    };

    let (fm_str, body) = split_frontmatter(&input);

    let meta: Frontmatter = if fm_str.is_empty() {
        Frontmatter::default()
    } else {
        match serde_yaml::from_str(fm_str) {
            Ok(meta) => meta,
            Err(err) => {
                eprintln!("error parsing frontmatter: {err}");
                std::process::exit(1);
            }
        }
    };

    if cli.info {
        println!("{}", serde_json::to_string(&output_info(&meta)).unwrap());
        return;
    }

    let mut output: Vec<u8> = Vec::new();

    write_page_setup(&mut output, &meta);
    render_body(&mut output, body);

    let output = match String::from_utf8(output) {
        Ok(output) => output,
        Err(err) => {
            eprintln!("error converting template: output is not UTF-8: {err}");
            std::process::exit(1);
        }
    };
    if let Err(err) = ensure_non_blank_typst(&output) {
        eprintln!("error converting template: {err}");
        std::process::exit(1);
    }
    print!("{output}");
}

fn ensure_non_blank_typst(output: &str) -> Result<(), &'static str> {
    if output.trim().is_empty() {
        return Err("converter produced empty Typst output");
    }
    Ok(())
}

fn output_info(meta: &Frontmatter) -> OutputInfo {
    let title = if meta.title.trim().is_empty() {
        "output".to_string()
    } else {
        meta.title.trim().to_string()
    };
    OutputInfo {
        schema_version: 1,
        output_base_name: clean_filename_base(&title),
        preview_title: title.clone(),
        document: DocumentInfo {
            title,
            keywords: vec!["模板".to_string()],
            language: "zh-CN".to_string(),
        },
    }
}

fn clean_filename_base(value: &str) -> String {
    let cleaned = value
        .trim()
        .chars()
        .map(|ch| if "/\\:*?\"<>|".contains(ch) { '_' } else { ch })
        .collect::<String>();
    if cleaned.is_empty() {
        "output".to_string()
    } else {
        cleaned
    }
}

fn read_stdin_limited() -> io::Result<String> {
    let stdin = io::stdin();
    let mut limited = stdin.lock().take(MAX_INPUT_BYTES + 1);
    let mut input = String::new();
    limited.read_to_string(&mut input)?;
    if input.len() as u64 > MAX_INPUT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("input exceeds {} bytes", MAX_INPUT_BYTES),
        ));
    }
    Ok(input)
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
        writeln!(w, r#"#let title = "{}""#, escape_typst_string(&meta.title)).unwrap();
        writeln!(w).unwrap();
        writeln!(
            w,
            r#"#align(center, text(size: 22pt, weight: "bold")[{}])"#,
            escape_typst_content(&meta.title)
        )
        .unwrap();
        writeln!(w, r#"#v(1em)"#).unwrap();
        writeln!(w).unwrap();
    }
}

/// Parses Markdown body using pulldown-cmark and outputs Typst.
fn render_body(w: &mut impl Write, source: &str) {
    let parser = pulldown_cmark::Parser::new(source);
    let mut code_block: Option<String> = None;

    for event in parser {
        match event {
            Event::Text(text) if code_block.is_some() => {
                if let Some(content) = &mut code_block {
                    content.push_str(&text);
                }
            }
            Event::SoftBreak if code_block.is_some() => {
                if let Some(content) = &mut code_block {
                    content.push('\n');
                }
            }
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
                write!(w, "{}", escape_typst_content(&text)).unwrap();
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
                write!(w, r#"#raw("{}")"#, escape_typst_string(&text)).unwrap();
            }

            Event::Start(Tag::CodeBlock(_)) => {
                code_block = Some(String::new());
            }
            Event::End(TagEnd::CodeBlock) => {
                let content = code_block.take().unwrap_or_default();
                write!(w, "{}", typst_raw_block(content.trim_end_matches('\n'))).unwrap();
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

fn escape_typst_string(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('#', "\\#")
}

fn escape_typst_content(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace(']', "\\]")
        .replace('#', "\\#")
}

fn typst_raw_block(content: &str) -> String {
    let fence = "`".repeat(std::cmp::max(3, max_backtick_run(content) + 1));
    format!("{fence}\n{content}\n{fence}\n\n")
}

fn max_backtick_run(text: &str) -> usize {
    let mut max_run = 0;
    let mut current = 0;
    for ch in text.chars() {
        if ch == '`' {
            current += 1;
            max_run = std::cmp::max(max_run, current);
        } else {
            current = 0;
        }
    }
    max_run
}

#[cfg(test)]
mod tests {
    use super::ensure_non_blank_typst;

    #[test]
    fn rejects_blank_typst_output() {
        assert!(ensure_non_blank_typst(" \n\t").is_err());
    }

    #[test]
    fn accepts_non_blank_typst_output() {
        assert!(ensure_non_blank_typst("#set page()\n").is_ok());
    }
}
