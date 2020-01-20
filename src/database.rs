use serde::Serialize;

// schema，可以单独写一个文件
pub mod schema {
	table! {
		tb_user (id) {
			id -> Integer,
			name -> Text,
			age -> Integer,
		}
	}
}

// model
#[derive(Serialize, Queryable)]
pub struct User {
	pub id: i32,
	pub name: String,
	pub age: i32,
}

// TODO use语句可以写到中间
use schema::tb_user;

// TODO 在Rust里一个表实际上可以对应两个Do，一个是全量插入的，一个是全量取出的，他们的区别在于
// TODO 全量取出的只有可能是null的需要写成Option，而全量插入的则对于哪些能够自动生成的数据如id和create_time等也是Option
// TODO 当然合并为一个也不是不可以，这个时候以全量插入的为准，到时候取出的时候会稍微麻烦一点，而且也不那么直观，
// TODO 比如我取出User，显然id不能也不会是null，但是model定义却是Option（为了兼容全量插入）
#[derive(Insertable)]
#[table_name = "tb_user"]
pub struct UserDo {
	pub id: Option<i32>,
	pub name: String,
	pub age: i32,
}
