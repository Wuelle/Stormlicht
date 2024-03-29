use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Write,
    fs, io,
    path::Path,
};

use proc_macro2::{Delimiter, TokenTree};

#[derive(Default, Debug)]
struct TypeJournal {
    /// Contains all the types that should be modeled
    types: HashSet<String>,
    /// Mapping from parent type -> types that inherit from it
    children: HashMap<String, Vec<String>>,
    /// The structs that don't inherit from anything
    roots: Vec<String>,
}

impl TypeJournal {
    /// Given the name of a type, find all it's parent types
    ///
    /// This returns a list of Strings, where the first string is the root type (which does not inherit from anything)
    /// and the last string is `typename`'s superclass.
    fn find_inheritance_chain_to(&self, typename: &str) -> Vec<String> {
        self.find_inheritance_chain_to_recursive(&self.roots, typename)
            .expect("type not accessible via any known root")
    }

    fn find_inheritance_chain_to_recursive(
        &self,
        parents: &[String],
        typename: &str,
    ) -> Option<Vec<String>> {
        for parent in parents {
            if parent == typename {
                return Some(vec![]);
            }

            if let Some(children) = self.children.get(parent) {
                if let Some(mut path) = self.find_inheritance_chain_to_recursive(children, typename)
                {
                    path.push(parent.clone());
                    return Some(path);
                }
            }
        }
        None
    }
}

fn search_for_derived_struct_in_file<P: AsRef<Path>>(
    path: P,
    type_journal: &mut TypeJournal,
) -> Result<(), io::Error> {
    let file_contents = fs::read_to_string(path)?;
    let ast = syn::parse_file(&file_contents).unwrap();

    for item in ast.items {
        if let syn::Item::Struct(struct_def) = item {
            // Check if the struct defines an "inherit" attribute
            let mut inherits_from = None;
            for attr in struct_def.attrs {
                if let Some(ident) = attr.path.get_ident() {
                    if ident == "inherit" {
                        if attr.tokens.is_empty() {
                            // This is a root object, it does not inherit from anything but the type system
                            // still needs to know about it
                            let root_name = struct_def.ident.to_string();
                            if !type_journal.types.insert(root_name.clone()) {
                                panic!("{root_name:?} was declared twice");
                            }
                            type_journal.roots.push(root_name);

                            break;
                        }

                        let mut tokens = attr.tokens.into_iter();
                        let group = match tokens.next() {
                            Some(TokenTree::Group(group))
                                if group.delimiter() == Delimiter::Parenthesis =>
                            {
                                group
                            },
                            _ => panic!("Invalid inherit attribute"),
                        };

                        let mut argument_tokens = group.stream().into_iter();
                        let parent_type_name = match argument_tokens.next() {
                            Some(TokenTree::Ident(ident)) => ident,

                            _ => panic!("Invalid inherit attribute"),
                        };
                        if argument_tokens.next().is_some() {
                            panic!("Invalid inherit attribute, are you trying to specify a full path (like foo::bar)? Because that's not allowed.")
                        }

                        inherits_from = Some(parent_type_name.to_string());
                        break;
                    }
                }
            }

            if let Some(parent_name) = inherits_from {
                let struct_name = struct_def.ident.to_string();
                if !type_journal.types.insert(struct_name.clone()) {
                    panic!("{struct_name:?} was declared twice");
                }

                type_journal
                    .children
                    .entry(parent_name)
                    .or_default()
                    .push(struct_name);
            }
        }
    }

    Ok(())
}

const DOM_OBJECT_PATH: &str = "src/dom/dom_objects";
const DOM_OBJECT_MODULE_PATH: &str = "crate::dom::dom_objects";

pub fn generate() -> Result<(), io::Error> {
    // Rerun if any DOM object changes
    // TODO: Since this is probably going to take a considerable amount of time
    // if the number of DOM object grows, we should consider caching and only updating
    // the files that changed.
    println!("cargo:rerun-if-changed={DOM_OBJECT_PATH}");

    // Used to keep track of who derives from where
    let mut type_journal = TypeJournal::default();

    // Search for inherited structs in each file inside src/dom_objects
    for dir_entry_or_error in fs::read_dir(DOM_OBJECT_PATH)? {
        let dir_entry = dir_entry_or_error?;

        if dir_entry.file_type()?.is_file() {
            search_for_derived_struct_in_file(dir_entry.path(), &mut type_journal)?;
        } else {
            println!("cargo:warning=Found directory {}, files in subdirectories are NOT considered by the inheritance system!", dir_entry.path().display())
        }
    }

    // Generate the required enums & trait impls
    let typenames: String = type_journal
        .types
        .iter()
        .fold(String::new(), |mut output, n| {
            let _ = write!(output, "{n},");
            output
        });

    let domtype_layout_match_arms: String = type_journal
        .types
        .iter()
        .fold(String::new(), |mut output, typename| {
            let _ = write!(output, "Self::{typename} => ::std::alloc::Layout::new::<{DOM_OBJECT_MODULE_PATH}::{typename}>(),");
            output
        });

    // For every known type, find the list of its parent types
    let mut parents = HashMap::with_capacity(type_journal.types.len());
    for typename in type_journal.types.iter() {
        parents.insert(typename, type_journal.find_inheritance_chain_to(typename));
    }

    let inheritance_trait_impls =
        type_journal
            .types
            .iter()
            .fold(String::new(), |mut output, typename| {
                let _ = write!(
                    output,
                    "impl DomTyped for {DOM_OBJECT_MODULE_PATH}::{typename} {{
                        fn as_type() -> DomType {{
                            DomType::{typename}
                        }}
                    }}"
                );
                output
            });

    let cast_type_match_arms: String =
        type_journal
            .types
            .iter()
            .fold(String::new(), |mut output, typename| {
                let path = parents.get(typename).unwrap();
                let any_parent_type: String =
                    path.iter().fold(String::new(), |mut output, typename| {
                        let _ = write!(output, "|DomType::{typename}");
                        output
                    });

                let _ = write!(
                output,
                "DomType::{typename} => matches!(other, DomType::{typename} {any_parent_type}),"
            );
                output
            });

    let is_a_impls = parents
        .iter()
        .fold(String::new(), |mut output, (typename, parents)| {
            for parent in parents {
                let _ = write!(output, "impl IsA<{DOM_OBJECT_MODULE_PATH}::{parent}> for {DOM_OBJECT_MODULE_PATH}::{typename} {{}}");
            }
            output
        });

    let autogenerated_code = format!(
        "
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum DomType {{
            {typenames}
        }}

        impl DomType {{
            pub const fn layout(&self) -> ::std::alloc::Layout {{
                match self {{
                    {domtype_layout_match_arms}
                }}
            }}

            pub const fn is_a(&self, other: Self) -> bool {{
                match self {{
                    {cast_type_match_arms}
                }}
            }}
        }}

        pub trait DomTyped {{
            fn as_type() -> DomType;
        }}

        /// Marker trait indicating parent classes
        /// 
        /// Upcasts into parent classes are infallible.
        pub trait IsA<S> {{}}
        
        {inheritance_trait_impls}

        {is_a_impls}
        "
    );

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("inheritance_autogenerated.rs");
    fs::write(dest_path, autogenerated_code)?;

    Ok(())
}
