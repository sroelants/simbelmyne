use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum OptionType {
    Check { default: bool },
    Spin { min: i32, max: i32, default: i32 },
    Combo { default: String, allowed: Vec<String> },
    Button,
    String { default: String },
}

impl Display for OptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Check { default } => {
                write!(f, "type check default {default}")?;
            },

            Self::Spin { min, max, default } => {
                write!(f, "type spin default {default} min {min} max {max}")?;
            }

            Self::Combo { default, allowed }=> {
                write!(f, "type combo default {default} ")?;

                for value in allowed {
                    write!(f, "var {value} ")?;
                }
            },

            Self::Button => {
                write!(f, "type button")?;
            },

            Self::String { default } => {
                write!(f, "type string default {default}")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UciOption {
    pub name: &'static str,
    pub option_type: OptionType,
}

impl Display for UciOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name {} {}", self.name, self.option_type)
    }
}
