use actix_web::{ResponseError, HttpResponse};
use derive_more::Display;
use rand::{
	distributions::{Distribution, Standard},
	thread_rng, Rng,
};
use actix_web::http::StatusCode;
use serde_derive::{Serialize, Deserialize};
//use std::fmt::Display;
use serde::export::Formatter;
use serde::export::fmt::Error;

#[derive(Debug, Display)]
pub enum CustomError {
	#[display(fmt = "Custom Error 1")]
	CustomOne,
	#[display(fmt = "Custom Error 2")]
	CustomTwo,
	#[display(fmt = "Custom Error 3")]
	CustomThree,
	#[display(fmt = "Custom Error 4")]
	CustomFour,
}

// TODO 还可以用结构体作为Error，即三个结构体，分别是SystemError，GeneralError，LogicError代表不同级别
// TODO 否则像上面枚举的方式一个大的系统错误太多了很难枚举过来，只能用归类，然后输出不同文案
// TODO 貌似还是用枚举更方便，用带参数的枚举，类似Option
// TODO 这里要手动实现Display
#[derive(Debug)]
pub enum CommonError {
	// 分别是code，msg，（还可以有第三方code，msg等），第三个是用来记录异常栈（比如来自哪个方法，哪行等）
	// 毕竟Rust和Java不一样，不是靠抛出异常来实现的，而是返回一个Err，Err的内容自定义，而这里CommonError就是自定义的异常内容
	// 所以最好把这个加上
	System(i32, String, Option<String>),
	Logic(i32, String, Option<String>),
	General(i32, String, Option<String>),
}

// 没发现什么地方用到了它，但是又必须实现
impl std::fmt::Display for CommonError {
	// TODO 生命周期也能用推断，即 '_ ？
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		match self {
			CommonError::System(code, message, trace) => {
				write!(f, "({}, {})", code, message)
			}
			CommonError::Logic(code, message, trace) => {
				write!(f, "({}, {})", code, message)
			}
			CommonError::General(code, message, trace) => {
				write!(f, "({}, {})", code, message)
			}
		}
	}
}

#[derive(Serialize, Deserialize)]
struct ApiResult {
	pub code: i32,
	pub message: String,
	// TODO 这里要再加个抛异常的时间的属性方便查找
}

impl Distribution<CustomError> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CustomError {
		match rng.gen_range(1, 5) {
			1 => CustomError::CustomOne,
			2 => CustomError::CustomTwo,
			3 => CustomError::CustomThree,
			_ => CustomError::CustomFour
		}
	}
}

impl ResponseError for CommonError {
	fn error_response(&self) -> HttpResponse {
		// 这种情况就直接是HttpCode是200即可，和客户端约定的是根据JSON里的code字段来判断是否执行成功（是要分开，相当于底层和应用层分开）
		// TODO HTTP Code是200表示请求经过了应用层处理（即非框架级别内部的处理），然后处理结果要看code值；
		match self {
			CommonError::System(code, message, trace) => {
				//let a = *code;
				//let b = message.clone();
				//let c = stack.unwrap();
				// TODO 记录最高等级的日志
				eprintln!("{}, {}, {:?}", code, message, trace);
				HttpResponse::Ok().body(serde_json::to_string(&ApiResult {code: *code, message: message.clone()}).unwrap())
			}
			CommonError::Logic(code, message, trace) => {
				// TODO 记录其次等级的日志
				eprintln!("{}, {}, {:?}", code, message, trace);
				HttpResponse::Ok().body(serde_json::to_string(&ApiResult {code: *code, message: message.clone()}).unwrap())
			}
			CommonError::General(code, message, trace) => {
				// TODO 记录再其次的日志
				eprintln!("{}, {}", code, message);
				HttpResponse::Ok().body(serde_json::to_string(&ApiResult {code: *code, message: message.clone()}).unwrap())
			}
		}
	}
}

impl ResponseError for CustomError {
	fn error_response(&self) -> HttpResponse {
		match self {
			CustomError::CustomOne => {
				println!("one");
				HttpResponse::Forbidden().body(format!("{}", CustomError::CustomOne))
			}
			CustomError::CustomTwo => {
				println!("two");
				HttpResponse::Unauthorized().body(format!("{}", CustomError::CustomTwo))
			}
			CustomError::CustomThree => {
				println!("three");
				HttpResponse::InternalServerError().body(format!("{}", CustomError::CustomThree))
			}
			CustomError::CustomFour => {
				println!("four");
				HttpResponse::BadRequest().body(format!("{}", CustomError::CustomFour))
			}
		}
	}
}

pub async fn do_something_random() -> Result<(), CustomError> {
	let mut rng = thread_rng();

	if rng.gen_bool(2.0 / 10.0) {
		Ok(())
	} else {
		Err(rand::random::<CustomError>())
	}
}
