// build.rs
use serde::Deserialize;
use serde_json;
use std::collections::HashSet;
use std::env;
use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("ldtk_constants.rs");

    // Path to your LDTK project file
    let ldtk_path = "assets/ldtk/project.ldtk"; // Adjust this path as needed

    // Tell cargo to rerun if the LDTK file changes
    println!("cargo:rerun-if-changed={}", ldtk_path);

    match generate_constants(ldtk_path, &dest_path) {
        Ok(_) => println!("Generated LDTK constants successfully"),
        Err(e) => {
            eprintln!("Failed to generate LDTK constants: {}", e);
            std::process::exit(1);
        }
    }
}

#[derive(Debug, Deserialize)]
struct LdtkProject {
    defs: Defs,
    levels: Vec<Level>,
}

#[derive(Debug, Deserialize)]
struct Defs {
    entities: Vec<EntityDef>,
    tilesets: Vec<TilesetDef>,
    enums: Vec<EnumDef>,
}

#[derive(Debug, Deserialize)]
struct EntityDef {
    identifier: String,
}

#[derive(Debug, Deserialize)]
struct TilesetDef {
    identifier: String,
}

#[derive(Debug, Deserialize)]
struct EnumDef {
    identifier: String,
    values: Vec<EnumValue>,
}

#[derive(Debug, Deserialize)]
struct EnumValue {
    id: String,
}

#[derive(Debug, Deserialize)]
struct Level {
    identifier: String,
    #[serde(rename = "layerInstances")]
    layer_instances: Vec<LayerInstance>,
}

#[derive(Debug, Deserialize)]
struct LayerInstance {
    #[serde(rename = "__identifier")]
    identifier: String,
}

fn generate_constants(
    ldtk_path: &str,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_content = read_to_string(ldtk_path)?;
    let project: LdtkProject = serde_json::from_str(&json_content)?;

    let mut output = File::create(output_path)?;

    writeln!(output, "// Auto-generated LDTK constants - DO NOT EDIT")?;
    writeln!(output)?;

    // Generate layer constants
    generate_layer_constants(&mut output, &project)?;

    // Generate entity constants
    generate_entity_constants(&mut output, &project)?;

    // Generate tileset constants
    generate_tileset_constants(&mut output, &project)?;

    // Generate enum constants
    generate_enum_constants(&mut output, &project)?;

    // Generate level constants
    generate_level_constants(&mut output, &project)?;

    Ok(())
}

fn generate_layer_constants(
    output: &mut File,
    project: &LdtkProject,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(output, "#[allow(dead_code)]")?;
    writeln!(output, "pub mod layers {{")?;

    let mut layer_identifiers = HashSet::new();

    // Extract layer identifiers from all levels
    for level in &project.levels {
        for layer in &level.layer_instances {
            layer_identifiers.insert(&layer.identifier);
        }
    }

    for identifier in layer_identifiers {
        let const_name = to_screaming_snake_case(identifier);
        writeln!(
            output,
            "    pub const {}: &str = \"{}\";",
            const_name, identifier
        )?;
    }

    writeln!(output, "}}")?;
    writeln!(output)?;

    Ok(())
}

fn generate_entity_constants(
    output: &mut File,
    project: &LdtkProject,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(output, "#[allow(dead_code)]")?;
    writeln!(output, "pub mod entities {{")?;

    for entity in &project.defs.entities {
        let const_name = to_screaming_snake_case(&entity.identifier);
        writeln!(
            output,
            "    pub const {}: &str = \"{}\";",
            const_name, entity.identifier
        )?;
    }

    writeln!(output, "}}")?;
    writeln!(output)?;

    Ok(())
}

fn generate_tileset_constants(
    output: &mut File,
    project: &LdtkProject,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(output, "#[allow(dead_code)]")?;
    writeln!(output, "pub mod tilesets {{")?;

    for tileset in &project.defs.tilesets {
        let const_name = to_screaming_snake_case(&tileset.identifier);
        writeln!(
            output,
            "    pub const {}: &str = \"{}\";",
            const_name, tileset.identifier
        )?;
    }

    writeln!(output, "}}")?;
    writeln!(output)?;

    Ok(())
}

fn generate_enum_constants(
    output: &mut File,
    project: &LdtkProject,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(output, "#[allow(dead_code)]")?;
    writeln!(output, "pub mod enums {{")?;

    for enum_def in &project.defs.enums {
        let enum_mod_name = to_snake_case(&enum_def.identifier);
        writeln!(output, "    pub mod {} {{", enum_mod_name)?;

        for value in &enum_def.values {
            let const_name = to_screaming_snake_case(&value.id);
            writeln!(
                output,
                "        pub const {}: &str = \"{}\";",
                const_name, value.id
            )?;
        }

        writeln!(output, "    }}")?;
    }

    writeln!(output, "}}")?;
    writeln!(output)?;

    Ok(())
}

fn generate_level_constants(
    output: &mut File,
    project: &LdtkProject,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(output, "#[allow(dead_code)]")?;
    writeln!(output, "pub mod levels {{")?;

    for level in &project.levels {
        let const_name = to_screaming_snake_case(&level.identifier);
        writeln!(
            output,
            "    pub const {}: &str = \"{}\";",
            const_name, level.identifier
        )?;
    }

    writeln!(output, "}}")?;
    writeln!(output)?;

    Ok(())
}

fn to_screaming_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_lower = false;

    for ch in s.chars() {
        if ch.is_ascii_uppercase() && prev_was_lower {
            result.push('_');
        }
        result.push(ch.to_ascii_uppercase());
        prev_was_lower = ch.is_ascii_lowercase();
    }

    result
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_lower = false;

    for ch in s.chars() {
        if ch.is_ascii_uppercase() && prev_was_lower {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
        prev_was_lower = ch.is_ascii_lowercase();
    }

    result
}
