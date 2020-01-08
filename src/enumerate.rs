use serde::{Deserialize, Deserializer,};
use serde::de::{self, Visitor, Error};
use std::fmt;

#[derive(Debug)]
pub enum GenderEnum {
	Male = 1,
	Female = -1,
	Unknown = 0,
	// 双性人
	Double = 2,
}

impl<'de> Visitor<'de> for GenderEnum {
	type Value = GenderEnum;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("无法将数值转换为GenderEnum")
	}

	/// 这里还可以实现其它基础类型，如i64,u32之类的，但是没必要，上面的expecting是必须实现的
	fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
		where
			E: de::Error,
	{
		Ok(if value == 1 {GenderEnum::Male} else if value == -1 {GenderEnum::Female} else if value == 2 {GenderEnum::Double} else {GenderEnum::Unknown})
	}
}

impl<'de> Deserialize<'de> for GenderEnum {

	fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
		D: Deserializer<'de> {
		// 这里有点蛋疼，虽然要求必须是enum里具体的某个值，但是实际上它最终反序列化是看上面的Visitor的，即这里换成GenderEnum::Female也一样
		deserializer.deserialize_i32(GenderEnum::Female)
	}
}

/*
impl From<i32> for GenderEnum {
	fn from(origin: i32) -> Self {
		if origin == 1 {
			GenderEnum::Male
		} else {
			GenderEnum::Female
		}
	}
}*/
