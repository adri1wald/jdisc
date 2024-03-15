use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct JDiscCli {
    #[command(subcommand)]
    command: JDiscCommand,
}

#[derive(Subcommand, Debug)]
enum JDiscCommand {
    // Discover the schema of a JSON file
    Discover {
        /// The path to the input JSON file
        #[arg(short, long, value_name = "INPUT FILE")]
        input: String,
        /// The path to the output JSON file
        #[arg(short, long, value_name = "OUTPUT FILE")]
        output: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = JDiscCli::parse();

    match &cli.command {
        JDiscCommand::Discover { input, output } => {
            // get file from input path relative to the current working directory
            // and load into json
            let input_file = std::fs::File::open(input)?;
            let input_json: serde_json::Value = serde_json::from_reader(input_file)?;
            let output_json = discover_schema(&input_json);
            serde_json::to_writer_pretty(std::fs::File::create(output)?, &output_json)?;
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
enum Schema {
    Null,
    Boolean,
    Number,
    String,
    Array(ArraySchema),
    Object(ObjectSchema),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
struct ArraySchema {
    items: Vec<Schema>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
struct ObjectSchema {
    properties: BTreeMap<String, Vec<Schema>>,
}

/// Discover the schema of a JSON file
fn discover_schema(input_json: &serde_json::Value) -> Schema {
    match input_json {
        serde_json::Value::Null => Schema::Null,
        serde_json::Value::Bool(_) => Schema::Boolean,
        serde_json::Value::Number(_) => Schema::Number,
        serde_json::Value::String(_) => Schema::String,
        serde_json::Value::Array(arr) => {
            let mut item_schemas: Vec<Schema> = vec![];
            for item in arr {
                let item_schema = discover_schema(item);
                if !item_schemas.contains(&item_schema) {
                    item_schemas.push(item_schema);
                }
            }
            Schema::Array(ArraySchema {
                items: item_schemas,
            })
        }
        serde_json::Value::Object(obj) => {
            let mut property_schemas: BTreeMap<String, Vec<Schema>> = BTreeMap::new();
            for (key, value) in obj {
                let property_schema = discover_schema(value);
                if !property_schemas.contains_key(key) {
                    property_schemas.insert(key.clone(), vec![property_schema]);
                } else {
                    property_schemas.get_mut(key).unwrap().push(property_schema);
                }
            }
            Schema::Object(ObjectSchema {
                properties: property_schemas,
            })
        }
    }
}
