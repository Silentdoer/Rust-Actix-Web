use serde_derive::{Serialize, Deserialize};

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
