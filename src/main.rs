use std::iter::repeat;
use std::process::{Command, Stdio};

use cargo_metadata::diagnostic::{DiagnosticLevel, DiagnosticSpan};
use cargo_metadata::MetadataCommand;
use colored::Colorize;
use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize)]
struct OurMetadata {
    tips: Vec<Tip>,
}

#[derive(Deserialize)]
struct Tip {
    #[serde(with = "serde_regex")]
    code_pattern: Regex,
    #[serde(with = "serde_regex")]
    message_pattern: Regex,
    #[serde(with = "serde_regex")]
    span_pattern: Regex,
    tip: String,
}

fn main() {
    Command::new("cargo")
        .args(&["check"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let metadata = MetadataCommand::new().exec().unwrap();
    let mut our_metadata = OurMetadata { tips: Vec::new() };

    for package in &metadata.packages {
        let tips_field = package.metadata["tips"].clone();
        if tips_field.is_array() {
            our_metadata.tips = serde_json::from_value(tips_field).unwrap();
        }
    }

    let tips = our_metadata.tips;

    let mut command = Command::new("cargo")
        .args(&["check", "--message-format=json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let reader = std::io::BufReader::new(command.stdout.take().unwrap());
    let mut diagnostics: Vec<(DiagnosticSpan, String)> = Vec::new();
    for message in cargo_metadata::Message::parse_stream(reader) {
        let message = message.unwrap();
        match message {
            cargo_metadata::Message::CompilerMessage(compiler_message) => {
                if compiler_message.message.level == DiagnosticLevel::Error {
                    for mapping in &tips {
                        let mut is_code_match = false;
                        let is_message_match = mapping
                            .message_pattern
                            .is_match(&compiler_message.message.message);
                        if let Some(code) = compiler_message.message.code.as_ref().map(|c| &c.code)
                        {
                            is_code_match = mapping.code_pattern.is_match(code);
                        }
                        if is_message_match || is_code_match {
                            let span = compiler_message.message.spans.iter().find(|span| {
                                span.label
                                    .as_ref()
                                    .map(|l| mapping.span_pattern.is_match(l))
                                    .unwrap_or(false)
                            });
                            if let Some(span) = span {
                                diagnostics.push((span.clone(), mapping.tip.clone()));
                            }
                        }
                    }
                }
            },
            _ => {},
        }
    }
    for (diagnostic, tip) in diagnostics {
        let line_range = diagnostic.line_start..=diagnostic.line_end;
        let lines = line_range.zip(diagnostic.text.into_iter());
        for (line_number, line) in lines {
            let mut line_gutter = line_number.to_string();
            line_gutter.extend(repeat(' ').take((3 - line_gutter.len()).max(3)));
            line_gutter.push_str("| ");

            let mut col_separator = String::new();
            col_separator.extend(repeat(' ').take(line_gutter.len() - 3));
            println!(
                "{col_separator}{} {}:{}",
                "-->".bright_blue().bold(),
                diagnostic.file_name,
                diagnostic.line_start,
            );

            let line_content = line.text;
            println!("{}{line_content}", line_gutter.bright_blue().bold());

            let mut pad = String::new();
            pad.extend(repeat(' ').take(line_gutter.len()));
            pad.extend(repeat(' ').take(line.highlight_start - 1));
            print!("{pad}");

            let mut highlight = String::new();
            highlight.extend(repeat('^').take(line.highlight_end - line.highlight_start));
            println!("{}", format!("{highlight} {tip}").bright_cyan().bold());
        }
    }
}
