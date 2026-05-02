//! [`Address`] value object.

use crate::strings::{City, Country, State, Street, ZipCode};

/// A postal address.  Equality is by all five components: street, city,
/// state, country, zip code.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address {
    street: Street,
    city: City,
    state: State,
    country: Country,
    zip_code: ZipCode,
}

impl Address {
    /// Construct from components.
    #[must_use]
    pub fn new(
        street: Street,
        city: City,
        state: State,
        country: Country,
        zip_code: ZipCode,
    ) -> Self {
        Self {
            street,
            city,
            state,
            country,
            zip_code,
        }
    }

    /// Street.
    #[must_use]
    pub fn street(&self) -> &Street {
        &self.street
    }

    /// City.
    #[must_use]
    pub fn city(&self) -> &City {
        &self.city
    }

    /// State.
    #[must_use]
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Country.
    #[must_use]
    pub fn country(&self) -> &Country {
        &self.country
    }

    /// Zip code.
    #[must_use]
    pub fn zip_code(&self) -> &ZipCode {
        &self.zip_code
    }
}
