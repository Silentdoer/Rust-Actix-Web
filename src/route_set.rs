use actix_web::{web, get, put, post, Result, HttpResponse, HttpRequest, Responder, error, Error, HttpMessage};
use actix_files as fs;

use std::sync::Mutex;
use crate::student::{Student, Stud};
use bytes::{BytesMut, Bytes};
use futures::StreamExt;
use json::JsonValue;
use actix_web::http::StatusCode;
use actix_session::{Session};
use crate::enumerate::GenderEnum;

// 和java里的Mapping方法不同点是，Rust里的get之类的，以及url都在在其他地方写的，所以单看route方法会很不直观
// 所以这里建议在注释上写好get，url等方便查看（TODO 查下rust是否可以将enum转换为基础类型的值，actix是否支持自定义转换，
// TODO 比如外界传的gender是-1这里表示是女，0表示未知，1表示是男，2表示是双性人，然后这里自动转换为自己写的性别enum）
// get /name/{name}/gender/{gender}
pub async fn index(info: web::Path<(String, i32)>) -> Result<String> {
	Ok(format!("Welcome {}! gender: {}", info.0, info.1))
}

pub async fn foo() -> &'static str {
	"hello, sf二手房jl\n"
}

// get /
pub async fn other(req: HttpRequest) -> HttpResponse {
	HttpResponse::Ok().body(format!("Other Req {}", req.uri()))
}

// /kkkk
pub async fn kkkk(state: web::Data<Mutex<usize>>, req: HttpRequest) -> HttpResponse {
	println!("{:?}", req);
	*(state.lock().unwrap()) += 1;
	HttpResponse::Ok().body(format!("Num of requests: {}", state.lock().unwrap()))
}

// /uuuu
pub async fn uuuu(state: web::Data<Mutex<usize>>, req: HttpRequest) -> HttpResponse {
	println!("sssr{:?}", req);
	*(state.lock().unwrap()) += 1;
	HttpResponse::Ok().body(format!("E$$DNum of requests: {}", state.lock().unwrap()))
}

/// TODO 还能写成类似这种格式：/foo/{name}.html和/foo/{name}.{ext}
#[get("/book/id/{id}")]
pub async fn book(info: web::Path<i32>) -> impl Responder {
	// Path实现了Deref能够自动解引用，所以这里可以不写info.into_inner()直接用info
	format!("hello {} is id", info)
}

#[get("/book/name/{name}")]
pub async fn book2(req: HttpRequest) -> impl Responder {
	format!("￥￥hello {} is id", req.match_info().get("name").unwrap())
}

#[get("/test1/{name}/{age}")]
pub async fn test1(info: web::Path<Stud>) -> impl Responder {
	// 这种情况下name就必须是pub了，如果只用于serde序列化可以不是pub
	format!("￥￥hello {} is id {}", info.name, info.age)
}

#[get("/test5/{name}/{age}")]
pub async fn test5(info: web::Path<(String, i32)>) -> impl Responder {
	// 这种情况下name就必须是pub了，如果只用于serde序列化可以不是pub
	format!("TTT {} is id {}", info.0, info.1)
}

#[get("/test6/{name}/{age}")]
pub async fn test6(info: web::Path<(String, GenderEnum)>) -> impl Responder {
	// 这种情况下name就必须是pub了，如果只用于serde序列化可以不是pub
	format!("TTT {} is id {:?}", info.0, info.1)
}

// /test2?name=kkk&age=88
#[get("/test2")]
pub async fn test2(info: web::Query<Stud>) -> impl Responder {
	// 这种情况下name就必须是pub了，如果只用于serde序列化可以不是pub
	format!("￥￥hello {} is sfsd {}", info.name, info.age)
}

// TODO 貌似不能这么写，会报错。。（官网上的例子貌似也是必须自己写成一个struct，如上面的Stud） /test2?name=kkk&age=88（这里的key名字可以随意。。）
#[get("/test4")]
pub async fn test4(info: web::Query<(String, String)>) -> impl Responder {
	// 这种情况下name就必须是pub了，如果只用于serde序列化可以不是pub
	let tmp = info.into_inner();
	println!("{}, ", &tmp.0);
	format!("RRhello {:?} is sfsd", &tmp)
}

// /test2?name=kkk&age=88；这种方式不行，有点可惜，因为String没有实现serde的Deserialize，而且也不清楚是否可以有两个Query
// TODO 不过可以用req.match_info().query("name").unwrap()
/*#[get("/test3")]
pub async fn test3(name: web::Query<String>, age: web::Query<String>) -> impl Responder {
	// 这种情况下name就必须是pub了，如果只用于serde序列化可以不是pub
	format!("XX hello {} is sfsd {}", name, age)
}*/

/// TODO 当请求类型是application/x-www-form-urlencoded，可以用web::Form<Student>来获取form数据
/// 注意对于post或者put方法，如果payload是JSON格式，需要填写Content-Type为application/json才行
pub async fn post1(item: web::Json<Student>) -> HttpResponse {
	println!("model: {:?}", &item);
	// Json虽然也实现了Deref，但是这里需要手动转换否则会报错（主要是json方法的问题？）
	HttpResponse::Ok().json(item.into_inner())
}

#[put("/student/id/{id}")]
pub async fn put1(info: web::Path<i32>, item: web::Json<Student>, req: HttpRequest) -> HttpResponse {
	println!("put {}, model: {:?}, {}#", info, &item, req.query_string());
	// Json虽然也实现了Deref，但是这里需要手动转换否则会报错（主要是json方法的问题？）
	// 这里也可以写item.0
	//req.take_payload()
	HttpResponse::Ok().json(item.into_inner())
}

/// 直接获取请求体字节的包装类Payload
#[post("/stud")]
pub async fn stud_post(mut payload: web::Payload) -> Result<HttpResponse, Error> {
	let mut body = BytesMut::with_capacity(16);
	// 注意，并不是请求体所有数据到位了才会执行到这里，它可以是只到了请求头和请求行的数据就可以匹配到这里，然后这里
	// 再进一步等待所有的数据获取完整
	// Some(chunk)是获取到了数据，然后chunk的数据还存在正确与错误的说法，所以下面还有chunk?
	while let Some(chunk) = payload.next().await {
		let chunk = chunk?;
		// 假设16是最大容量客户端会收到overflow的返回值，400异常
		if (body.len() + chunk.len()) > 32 {
			return Err(error::ErrorBadRequest("overflow"));
		}
		body.extend_from_slice(&chunk);
	}
	let obj = serde_json::from_slice::<Student>(&body)?;
	// 将obj转换为JSON字符串
	Ok(HttpResponse::Ok().json(obj))
}

#[post("/stud2")]
pub async fn stud2_post(body: Bytes) -> Result<HttpResponse, Error> {
	let result = json::parse(std::str::from_utf8(&body).unwrap());
	let injson: JsonValue = match result {
		Ok(v) => v,
		Err(e) => json::object! { "err" => e.to_string() },
	};
	Ok(HttpResponse::Ok()
		.content_type("application/json;charset=UTF8")
		.body(injson.dump()))
}

// .ico后缀可以不匹配？（貌似不行，所以这里应该要写成/favicon.ico而不能只写/favicon
// 不过这里支持正则表达式，所以用"/favicon.*"也行，不过貌似一般会在前面加个r表示这个是正则的pattern，即r"/favicon.*"
// TODO 好吧，这里的貌似不是正则表达式，就是actix自己的一种匹配方式，*表示*所在位置可以有0到n个字符
#[get("/favicon*")]
pub async fn favicon() -> Result<fs::NamedFile> {
	Ok(fs::NamedFile::open("static/favicon.ico")?)
}

#[get("/index")]
pub async fn welcome(session: Session, req: HttpRequest) -> Result<HttpResponse> {
	println!("{:?}", req);
	let mut counter = 100;
	// .get有点像.0，莫非是特殊的结构体？
	if let Some(count) = session.get::<i32>("counter")? {
		println!("SESSION value: {}", count);
		counter = count + 1;
	}

	// 貌似没有生效（TODO session功能貌似有问题，不过这个对自己不是很重要，反正自己不用session）
	session.set("counter", counter)?;

	Ok(HttpResponse::build(StatusCode::OK)
		.content_type("text/html; charset=utf-8")
		// include_str!宏居然还能这么用，将html文件转换为一个字符串（如果是比如图片之类的就不能这么搞）
		.body(include_str!("../static/index.html")))
}

pub async fn p404() -> Result<fs::NamedFile> {
	Ok(fs::NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}
