use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use heck::ToPascalCase;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IconMetadata {
    encoded_code: String,
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    generate_icons().expect("failed to generate icons");
}

fn generate_icons() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=assets/fonts/info.json");
    println!("cargo:rerun-if-changed=assets/fonts/echoinput-icons.ttf");

    let reader = File::open("assets/fonts/info.json")?;
    let reader = BufReader::new(reader);
    let icons: BTreeMap<String, IconMetadata> = serde_json::from_reader(reader)?;

    let mut entries = Vec::new();
    for (name, metadata) in icons {
        let hex = metadata
            .encoded_code
            .strip_prefix('\\')
            .ok_or("invalid codepoint format")?;
        let codepoint = u32::from_str_radix(hex, 16)?;
        entries.push((codepoint, name.to_pascal_case()));
    }

    let out_dir = std::env::var_os("OUT_DIR").ok_or("OUT_DIR is not set")?;
    let out_path = Path::new(&out_dir).join("icons.rs");
    let mut out = File::create(out_path)?;

    writeln!(out, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]")?;
    writeln!(out, "pub enum Icon {{")?;
    for (_, name) in entries.iter() {
        writeln!(out, "    {name},")?;
    }
    writeln!(out, "}}")?;
    writeln!(out)?;
    writeln!(out, "impl From<Icon> for char {{")?;
    writeln!(out, "    fn from(icon: Icon) -> char {{")?;
    writeln!(out, "        match icon {{")?;
    for (codepoint, name) in entries.iter() {
        writeln!(out, "            Icon::{name} => '\\u{{{codepoint:x}}}',")?;
    }
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;

    Ok(())
}
