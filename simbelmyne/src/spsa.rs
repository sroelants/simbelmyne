use std::fmt::{Display, Formatter, Result, Error};
use uci::options::{OptionType, UciOption};
use engine::search::params::SPSA_UCI_OPTIONS;

////////////////////////////////////////////////////////////////////////////////
//
// Openbench helper struct
//
// Wraps a UCI option and implements display so it prints the UCI option into a
// OB-compatible format
//
////////////////////////////////////////////////////////////////////////////////

pub struct OpenbenchSpsa(pub UciOption);

const L_RATE: f32 = 0.002;

impl Display for OpenbenchSpsa {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use OptionType::*;
        let name = self.0.name;
        let Spin { min, max, default, step } = self.0.option_type else {
            return Result::Err(Error)
        };

        write!(f, "{name}, int, {default}, {min}, {max}, {step}, {L_RATE}")
    }
}

// Print out the full set of SPSA-tunable parameters in OB format
pub fn run_openbench() {
    for option in SPSA_UCI_OPTIONS {
        println!("{}", OpenbenchSpsa(option));
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Weather-Factory helper struct
//
// Wraps a UCI option and implements display so it prints the UCI option into a
// WeatherFactry-compatible format
//
////////////////////////////////////////////////////////////////////////////////

pub struct WeatherFactorySpsa(pub UciOption);

impl Display for WeatherFactorySpsa {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use OptionType::*;
        let name = self.0.name;
        let Spin { min, max, default, step } = self.0.option_type else {
            return Result::Err(Error)
        };

        write!(f, r#""{name}": {{ "value": {default}, "min_value": {min}, "max_value": {max}, "step": {step} }}"#)
    }
}

// Print out the full set of SPSA-tunable parameters in WF format
pub fn run_weatherfactory() {
    println!("{{");

    for (i, option) in SPSA_UCI_OPTIONS.into_iter().enumerate() {
        print!("{}", WeatherFactorySpsa(option));

        // If there is another option left to go, add a trailing comma
        if i + 1 < SPSA_UCI_OPTIONS.len() {
            println!(",");
        } else {
            println!("");
        }
    }

    println!("}}");
}
