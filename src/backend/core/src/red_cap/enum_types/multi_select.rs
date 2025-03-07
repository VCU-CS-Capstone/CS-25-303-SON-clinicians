use crate::red_cap::RedCapExportDataType;
use crate::red_cap::{MultiSelectType, RedCapDataSet, RedCapEnum, RedCapType, utils::is_all_none};
use crate::utils::InvalidVariant;
use chumsky::container::Seq;
use cs25_303_macros::RedCapEnum;
use serde::Serialize;
use strum::EnumIter;
use tracing::{debug, warn};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Default)]
pub struct RedCapRace {
    pub race: Option<Vec<Race>>,
    pub race_other: Option<String>,
    pub race_multiracial_other: Option<String>,
}
impl RedCapRace {
    pub fn has_race(&self, race: &Race) -> bool {
        if let Some(races) = &self.race {
            races.contains(race)
        } else {
            false
        }
    }
}
impl RedCapType for RedCapRace {
    fn read<D: RedCapDataSet>(data: &D) -> Option<Self>
    where
        Self: Sized,
    {
        let race = data.get_enum_multi_select("race");
        let race_other = data.get_string("race_other");
        let race_multiracial_other = data.get_string("race_multiracial_other");
        is_all_none!(race, race_other, race_multiracial_other);
        Some(Self {
            race,
            race_other,
            race_multiracial_other,
        })
    }

    fn write<D: RedCapDataSet>(&self, data: &mut D) {
        let has_race_other = self.has_race(&Race::IdentifyOther);
        let has_race_multiracial_other = self.has_race(&Race::Multiracial);
        let multi_select_race = self
            .race
            .as_ref()
            .map(|value| Race::create_multiselect("race", value));
        debug!(?multi_select_race, "Multi Select Race");
        if let Some(race) = multi_select_race {
            data.insert("race", race.into());
        }
        // Prevent race_other and race_multiracial_other from having values without correct multi select values because it makes RedCap mad
        if has_race_other {
            data.insert("race_other", self.race_other.clone().into());
        } else {
            if self.race_other.is_some() {
                warn!(
                    ?self,
                    "Race Other Value Present without having IdentifyOther selected"
                );
            }
            data.insert("race_other", RedCapExportDataType::Null);
        }
        if has_race_multiracial_other {
            data.insert(
                "race_multiracial_other",
                self.race_multiracial_other.clone().into(),
            );
        } else {
            if self.race_multiracial_other.is_some() {
                warn!(
                    ?self,
                    "Race race_multiracial_other Value Present without having Multiracial selected"
                );
            }
            data.insert("race_multiracial_other", RedCapExportDataType::Null);
        }
    }
}

/// Please Note
///
/// Race is a multi select field in RedCap with two extra fields `race_other` and `race_multiracial_other`
///
/// `race_other` is a text field that is only shown if the user selects `IdentifyOther`
/// `race_multiracial_other` is a text field that is only shown if the user selects `Multiracial`
///
/// However, you can select. Multiracial and then select multiple other races. So keep that in mind.
#[derive(Debug, Clone, PartialEq, Eq, RedCapEnum, ToSchema, EnumIter)]
pub enum Race {
    #[red_cap(enum_index = 3)]
    NativeAmerican,
    #[red_cap(enum_index = 4)]
    Asian,
    #[red_cap(enum_index = 2)]
    Black,
    #[red_cap(enum_index = 5)]
    Hispanic,
    #[red_cap(enum_index = 10)]
    MiddleEasternOrNorthAfrican,
    #[red_cap(enum_index = 7)]
    NativeHawaiianOrOtherPacificIslander,
    #[red_cap(enum_index = 1)]
    White,
    /// Will have a second field with a value in DB
    ///
    /// RedCap Field: `race_multiracial_other`
    /// Database Field: `race_multiracial_other`
    #[red_cap(enum_index = 9)]
    Multiracial,
    /// Will have a second field with a value in DB
    ///
    /// RedCap Field: `race_other`
    /// Database Field: `race_other`
    #[red_cap(enum_index = 6)]
    IdentifyOther,
    #[red_cap(enum_index = 8)]
    Declined,
}
impl MultiSelectType for Race {
    fn multi_select_key() -> &'static str {
        "race"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, RedCapEnum, ToSchema, EnumIter)]
pub enum HealthInsurance {
    #[red_cap(enum_index = 1)]
    Medicaid,
    #[red_cap(enum_index = 2)]
    Medicare,
    #[red_cap(enum_index = 3)]
    Private,
    #[red_cap(enum_index = 4)]
    VA,
    #[red_cap(enum_index = 5)]
    None,
}
impl MultiSelectType for HealthInsurance {
    fn multi_select_key() -> &'static str {
        "insurance"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, RedCapEnum, ToSchema, EnumIter)]
pub enum MobilityDevice {
    #[red_cap(enum_index = 1)]
    None,
    #[red_cap(enum_index = 2)]
    Cane,
    #[red_cap(enum_index = 3)]
    Walker,
    #[red_cap(enum_index = 4)]
    Rollator,
    #[red_cap(enum_index = 5)]
    ManualWheelchair,
    #[red_cap(enum_index = 6)]
    PowerWheelchair,
    #[red_cap(enum_index = 7)]
    PowerScooter,
}
impl MultiSelectType for MobilityDevice {
    fn multi_select_key() -> &'static str {
        "mobility_devices"
    }
}
