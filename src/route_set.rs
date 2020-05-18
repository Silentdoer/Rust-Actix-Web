// 这里web::{self, BytesMut}的意思是web, web::BytesMut的意思
use actix_web::{client::Client, web::{self, BytesMut}, get, put, post, Result, HttpResponse, HttpRequest, Responder, error, Error, HttpMessage};
use actix_files as fs;

use std::sync::Mutex;
use crate::student::{Student, Stud, NeedValidData};
use bytes::{Bytes};
use futures::StreamExt;
use json::JsonValue;
use actix_web::http::StatusCode;
use actix_session::Session;
use crate::enumerate::GenderEnum;
use actix::Addr;
use actix::prelude::*;
use actix_redis::{RedisActor, Command};
use redis_async::resp::RespValue;
use redis_async::*;
use crate::custom_error::{do_something_random, CustomError, CommonError};
use actix_web::error::ErrorBadRequest;
use validator::Validate;

use crate::database::schema::*;
use crate::database::{User, UserDo};
use diesel::r2d2::ConnectionManager;
use diesel::pg::PgConnection;
use diesel::prelude::*;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

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
	// match_info().get(...)里可以用正则表达式？
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
	// 这里又可以不需要&info.0，有点搞不懂。。
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

// 以后可以写个宏来定义结构体，否则每个handler上面的QueryStr都得写不同的名字。。
#[derive(Deserialize)]
struct QueryStr {
	name: Option<String>
}

// 函数参数上的类型如果写在函数里是无法被函数参数发现的，所以这个有点蛋疼，多个RPath还必须不重名【但是实际上这个RPath或
// RHeader或RQuery都是只在某个请求里用到，其他里是不会用到的，，然后也无法在全局里用{}加上一个作用域【或许可以用宏实现】
#[get("/test12")]
pub async fn test12(info: web::Query<QueryStr>) -> impl Responder {
	// 这种情况下name就必须是pub了，如果只用于serde序列化可以不是pub
	format!("33￥￥444hello {} is sfsd", if let Some(d) = info.into_inner().name {d} else {"未提供".to_string()})
}

// TODO 貌似不能这么写，会报错。。（官网上的例子貌似也是必须自己写成一个struct，如上面的Stud） /test2?name=kkk&age=88（这里的key名字可以随意。。）
// 但是这个应该是可以被优化的（但是不应该被优化，因为QueryString是可以不按顺序来写，如name=8&age=9和age=9&name=8是等价的
// 如果可以是Path则会出问题
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
	let mut body = bytes::BytesMut::with_capacity(16);
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

#[post("/redisSet")]
pub async fn redis_set(info: web::Json<Stud>, redis: web::Data<Addr<RedisActor>>)
					   -> Result<HttpResponse, Error> {
	println!("hello redis set");
	let info = info.into_inner();
	// SET就是redis-cli里的set命令（还有hset等；后面的参数顺序其实就是和redis-cli里的顺序一致
	// 现在只是创建了这个命令，但还没执行（应该
	let name = redis.send(Command(resp_array!["SET", "namespace1:key:name", info.name]));
	let age = redis.send(Command(resp_array!["SET", "namespace1:key:age", info.age]));
	// 执行命令（或者其实上面已经提交了命令，这里只是join？
	let res: Vec<Result<RespValue, Error>>
		= futures::future::join_all(vec![name, age].into_iter())
		.await
		.into_iter()
		.map(|item| {
			// map_err是针对Result的方法，它的作用是当Result是Err的时候用Error::from将异常转换为Error（actix里的）
			// and_then也是Result里的
			item.map_err(Error::from)
				.and_then(|res| res.map_err(Error::from))
		}).collect();

	if !res.iter().all(|res| match res {
		Ok(RespValue::SimpleString(x)) if x == "OK" => true,
		_ => false,
	}) {
		// 这里finish()是指Response数据完成，不需要body直接finish
		// 这里body()和finish()都有完成build的作用，但是像header(..)则不会，还是返回的ResponseBuilder
		Ok(HttpResponse::InternalServerError().finish())
	} else {
		Ok(HttpResponse::Ok().body("successfully cached values\n"))
	}
}

#[get("/redisTest/{val}")]
pub async fn redis_test(info: web::Path<String>, redis: web::Data<Addr<RedisActor>>) -> Result<HttpResponse, Error> {
	println!("redis test in");
	// set命令只能一条一条执行（除非写事物操作）
	let res = redis.send(Command(resp_array![
									"SET",
									"kkk",
									info.into_inner()
								])).await?;

	match res {
		// set成功则返回字符串OK
		Ok(RespValue::SimpleString(x)) if x == "OK" => {
			Ok(HttpResponse::Ok().body("successfully set\n"))
		}
		_ => {
			Ok(HttpResponse::InternalServerError().finish())
		}
	}
}

#[get("/redis_del/{key}")]
pub async fn redis_del(key: web::Path<String>, redis: web::Data<Addr<RedisActor>>) -> Result<HttpResponse, Error> {
	println!("redis del in");
	// TODO 不是所有的“数组”都支持最后一个元素可以有,，比如这里的resp_array![..]就不行（至少当前版本不行）
	let res = redis.send(Command(resp_array![
									"DEL",
									key.into_inner()
								])).await?;
	match res {
		// del key1 key2 ...；成功删除多少个key就会返回成功个数（integer）
		Ok(RespValue::Integer(x)) if x == 1 => {
			Ok(HttpResponse::Ok().body("successfully del\n"))
		}
		_ => {
			Ok(HttpResponse::InternalServerError().finish())
		}
	}
}

#[get("/redis_hset/{key}/{field}/{value}")]
pub async fn redis_hset(info: web::Path<(String, String, String)>
						, redis: web::Data<Addr<RedisActor>>) -> Result<HttpResponse, Error> {
	println!("redis del in");
	// TODO 这里必须是&info.0，而上面的不用貌似是因为resp_array!对参数的处理不同导致的。。
	let res = redis.send(Command(resp_array![
									"HSET",
									&info.0,
									&info.1,
									&info.2
								])).await?;
	match res {
		// hset和del一样返回(integer)1【注意其实就是返回1，前面的是指类型是integer）
		Ok(RespValue::Integer(x)) if x == 1 => {
			Ok(HttpResponse::Ok().body("successfully hset\n"))
		}
		_ => {
			Ok(HttpResponse::InternalServerError().finish())
		}
	}
}

#[get("/do_something")]
pub async fn do_something() -> Result<HttpResponse, Error> {
	// await其实就类似future.get()
	do_something_random().await?;
	Ok(HttpResponse::Ok().body("Nothing happened."))
}

#[get("/custom_error/{id}")]
pub async fn custom_error(r#type: web::Path<i32>) -> Result<HttpResponse, Error> {
	/*if rng.gen_bool(2.0 / 10.0) {
		Ok(())
	} else {
		Err(rand::random::<CustomError>())
	}*/
	match r#type.into_inner() {
		1 => Err(CustomError::CustomOne)?,
		2 => Err(CustomError::CustomTwo)?,
		3 => Err(CustomError::CustomThree)?,
		_ => Err(CustomError::CustomFour)?
	}
}

// TODO 抛了异常就会被记录（Err），并且会记录Err的所有内容，这一个不知道怎么去关闭它，想自己来实现日志打印
// TODO 所以应该是这里弄：middleware::Logger::default()，自己实现一个，这样就不会出现重复打印问题
// TODO 根据返回是Err然后再去记录（改了info级别后debug不打印了，但是会打印出口简单日志（不包括请求体数据））
#[get("/custom_error2/{id}")]
pub async fn custom_error2(r#type: web::Path<i32>) -> Result<HttpResponse, Error> {
	/*if rng.gen_bool(2.0 / 10.0) {
		Ok(())
	} else {
		Err(rand::random::<CustomError>())
	}*/
	match r#type.into_inner() {
		1 => Err(CommonError::System(5000, "系统异常，这个异常一般来自其他地方抛出重新放到这里".to_owned(), Some("来自route_set custom_error2".to_owned())))?,
		2 => Err(CommonError::Logic(4000, "系统繁忙，请稍后重试".to_owned(), Some("来自route_set custom_error2".to_owned())))?,
		_ => Err(CommonError::General(3000, "密码错误".to_owned(), None))?
	}
}

#[post("/validate_test")]
pub async fn validate_test(data: web::Json<NeedValidData>, client: web::Data<Client>) -> Result<String, Error> {
	let data = data.into_inner();
	let result: Result<_, _> = data.validate();
	if result.is_err() {
		return Err(CommonError::Logic(4000, "入口参数验证失败".to_owned(), None))?;
	}
	// 算了，HttpClient工具还是用通用的吧，用actix的不够通用
	//let res: Result<_, _> = client.get("https://baidu.com").await?;
	/*let mut res = client.get("https://baidu.com").await.map_err(Error::from)?;
	let mut body = BytesMut::new();
	while let Some(chunk) = res.next().await {
		body.extend_from_slice(&chunk?);
	}

	let string = serde_json::to_string(&body).unwrap();
	println!("百度数据：{}", string);
	Ok(string)*/
	Ok("暂时没用".to_owned())
}

#[get("/db_diesel/{id}")]
pub async fn db_diesel(info: web::Path<i32>, pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
	// TODO 还能在方法里面use（之前只知道可以在方法了里面创建方法，现在还可以创建结构类型）
	use crate::database::schema::tb_user::dsl::*;
	struct Aa {
		pro: i32
	}
	// 生成uuid算法的等级为v4
	let uuid = format!("{}", uuid::Uuid::new_v4());
	println!("uuid: {}", uuid);
	let conn = &pool.get().unwrap();
	// id对应table!{tb_user里的id
	let mut result = tb_user.filter(id.eq(&info.into_inner())).load::<crate::database::User>(conn);
	match result {
		// TODO 注意，这一层是指SQL语句没有问题（包括存在表，用户权限够等），但不代表select出来有数据，所以还要加一层match
		Ok(mut items) => {
			Ok(HttpResponse::Ok().body(match &items.pop() {
				Some(item) => serde_json::to_string(item).unwrap(),
				None => "没有数据".to_owned()
			}))
		},
		Err(e) => {
			Err(ErrorBadRequest(format!{"{:?}", e}))
		}
	}
}

// URL里不能有中文，需要进行URLEncoded（Chrome表面上可以写中文，但是发送数据时是进行了转换的）
#[get("/db_diesel_post/{name}/{age}")]
pub async fn db_diesel_post(info: web::Path<(String, i32)>, pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
	let (a, b) = info.into_inner();
	use crate::database::schema::tb_user::dsl::*;
	let conn = &pool.get().unwrap();
	let result = diesel::insert_into(tb_user).values(&UserDo {id: None, name: a, age: b }).execute(conn);
	// Some或Err是在if let左边
	if let Err(err) = result {
		Err(ErrorBadRequest(format!("{:?}", err)))
	} else {
		Ok(HttpResponse::Ok().body("插入数据成功"))
	}
}
