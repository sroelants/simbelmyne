use std::fmt::{Display, Formatter, Result, Error};

use uci::options::{OptionType, UciOption};

////////////////////////////////////////////////////////////////////////////////
//
// Openbench helper struct
//
// Wraps a UCI option and implements display so it prints the UCI option into a
// OB-compatible format
//
////////////////////////////////////////////////////////////////////////////////

pub struct OpenbenchSpsa(UciOption);

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

////////////////////////////////////////////////////////////////////////////////
//
// Weather-Factory helper struct
//
// Wraps a UCI option and implements display so it prints the UCI option into a
// WeatherFactry-compatible format
//
////////////////////////////////////////////////////////////////////////////////

pub struct WeatherFactorySpsa(UciOption);

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

