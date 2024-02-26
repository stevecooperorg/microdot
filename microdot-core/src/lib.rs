pub mod command;
pub mod exporter;
pub mod graph;
pub mod hash;
pub mod labels;
pub mod pet;
pub mod util;

macro_rules! new_string_type {
    ($id: ident) => {
        #[derive(PartialEq, Eq, Hash, Debug, Clone, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
        pub struct $id(String);

        impl $id {
            pub fn new<S: Into<String>>(str: S) -> Self {
                Self(str.into())
            }
        }

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

new_string_type!(CommandResult);
new_string_type!(Label);
new_string_type!(Id);
new_string_type!(Line);
