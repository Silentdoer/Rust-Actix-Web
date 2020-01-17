use regex::Regex;
use lazy_static::lazy_static;

lazy_static!{
	pub static ref PHONE_REG: Regex = Regex::new(r"^[1-9]\d{10}$").unwrap();
}
