use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum GenderEnum {
	Unknown = 0,
	Male = 1,
	Female = -1,
	Double = 2,
}