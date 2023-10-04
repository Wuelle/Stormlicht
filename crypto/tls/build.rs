use std::{env, fs, io, path::Path};

use serialize::{
    json::{JsonDeserializer, JsonError},
    Deserialize,
};

const OBJECT_IDENTIFIERS_PATH: &str = "object_identifiers.json";

#[derive(Deserialize, Clone, Debug)]
struct Namespace {
    digits: Vec<usize>,
    short_name: String,
    long_name: String,
    elements: Vec<Namespace>,
}

#[derive(Debug)]
enum Error {
    IO(io::Error),
    Deserialization(JsonError),
}

impl Namespace {
    fn build_consts(&self, consts: &mut Vec<String>, path: &mut Vec<usize>) {
        path.extend_from_slice(&self.digits);

        let basename = if self
            .long_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
        {
            &self.long_name
        } else {
            &self.short_name
        };

        let basename = basename
            .to_ascii_uppercase()
            .replace([' ', '-', '.', '/'], "_");
        println!("{}", self.long_name);
        if basename.is_empty() {
            panic!();
        }

        let constant = format!("pub const {basename}: &[usize] = &{path:?};");
        consts.push(constant);

        for element in &self.elements {
            element.build_consts(consts, path);
        }

        path.truncate(path.len() - self.digits.len());
    }

    fn as_constant(&self) -> String {
        format!(
            "Namespace {{ digits: &{:?}, short_name: {:?}, long_name: {:?}, elements: &[{}] }}",
            self.digits,
            self.short_name,
            self.long_name,
            self.elements
                .iter()
                .map(|ns| ns.as_constant())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

fn main() -> Result<(), Error> {
    // NOTE: Thanks to the openssl developers for collecting so many different
    //       object identifiers and their meaning in https://github.com/openssl/openssl/blob/master/crypto/objects/objects.txt
    //       Our "object_identifiers.json" is a cleaned up version of their file.
    //       (This note can't be in the json itself because json doesn't support comments :/)

    println!("cargo:rerun-if-changed={OBJECT_IDENTIFIERS_PATH}");

    let json = String::from_utf8(fs::read(OBJECT_IDENTIFIERS_PATH)?)
        .expect("html_named_entities.json contains invalid utf-8");
    let mut deserializer = JsonDeserializer::new(&json);
    let root_namespaces: Vec<Namespace> = Vec::deserialize(&mut deserializer)?;

    // Create a constant for every known object identifier
    let mut consts = vec![];
    let mut path = vec![];
    for namespace in &root_namespaces {
        namespace.build_consts(&mut consts, &mut path);
    }
    let consts = consts.join("");

    let num_roots = root_namespaces.len();

    // Unfortunately we can't just use {:?} on root_namespaces because we need slices instead of arrays
    let namespace_constants = root_namespaces
        .iter()
        .map(|ns| ns.as_constant())
        .collect::<Vec<String>>()
        .join(",");

    let autogenerated_code = format!(
        "
        impl ::std::fmt::Debug for super::ObjectIdentifier {{
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {{
                #[derive(Debug, Clone, Copy)]
                struct Namespace {{
                    digits: &'static [usize],
                    short_name: &'static str,
                    long_name: &'static str,
                    elements: &'static [Namespace],
                }}
        
                const ROOT_NAMESPACES: [Namespace; {num_roots}] = [{namespace_constants}];
                
                // Generate a debug impl for ObjectIdentifier
                // If there is an exact match for the identifier, we display its long name
                // Otherwise, we display the number of each segment, along with its short name (if we know it)
                let mut fallback = vec![];
                let mut remaining_parts = self.parts.as_slice();
                let mut current_set_of_namespaces = ROOT_NAMESPACES.as_slice();
                loop {{
                    let matching_namespace = current_set_of_namespaces.iter().find(|ns| remaining_parts.starts_with(ns.digits));
                    match matching_namespace {{
                        Some(ns) => {{
                            fallback.push(format!(\"{{}} {{:?}}\", ns.short_name, &ns.digits));
                            remaining_parts = &remaining_parts[ns.digits.len()..];
                            if remaining_parts.is_empty() {{
                                return write!(f, \"{{}}\", ns.long_name);
                            }}
                            current_set_of_namespaces = ns.elements;
                        }},
                        None => {{
                            // We don't know this namespace (or any of its children)
                            for p in remaining_parts {{
                                fallback.push(format!(\".{{p}}\"));
                            }}
                            return write!(f, \"{{}}\", fallback.join(\".\"));
                        }}
                    }}
                }}
            }}
        }}

        pub mod exports {{
            {consts}
        }}
        "
    );

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("object_identifier.rs");
    fs::write(dest_path, autogenerated_code)?;

    Ok(())
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<JsonError> for Error {
    fn from(value: JsonError) -> Self {
        Self::Deserialization(value)
    }
}