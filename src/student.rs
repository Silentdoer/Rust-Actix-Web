use serde_derive::{Serialize, Deserialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct Student {
	// 属性可以不用pub
	name: String,
	age: usize,
}

#[derive(Deserialize)]
pub struct Stud {
	pub name: String,
	pub age: String,
}

// Validate功能有问题。。
#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct NeedValidData {
	#[validate(length(min = 1, max = 10))]
	id: String,
	// phone和email用不了
	#[validate(email)]
	email: String,
	#[validate(range(min = 18, max = 36))]
	age: i32,
	// 这个phone格式应该是美国那边的吧？（看下怎么自定义。。）
	//#[validate(phone)]
	#[validate(regex = "crate::regex_constants::PHONE_REG")]
	phone: String,
	// 这个如果是中文的url估计也会提示有问题，可能还需要自定义
	#[validate(url)]
	site: String,
	#[validate(contains = "wang")]
	name: String,
	#[validate(length(equal = 4))]
	pro1: String,
	//#[validate(custom = "crate::validate::custom_validate")]
}
