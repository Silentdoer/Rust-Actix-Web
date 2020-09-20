mod route_set;
mod student;
mod enumerate;
mod custom_error;
mod regex_constants;
mod middleware;
mod custom_guard;
mod database;

// TODO 在这个程序里这个又必须有，但是有的又可以省略下面两句代码，不知道为什么。。（莫非是redis_async版本太低所以必须用老版本的导入方式？）
#[macro_use]
extern crate redis_async;
// TODO 对于validator，下面四个都必须在lib.rs或main.rs里声明如下
#[macro_use]
extern crate validator_derive;
extern crate validator;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate diesel;

use std::{thread, time};

use actix_rt::System;
use actix_web::{middleware as awmw, web, guard, App, HttpRequest, HttpServer, HttpResponse, Responder, client::Client};
use std::sync::Mutex;
use actix_web::http::{header, Method, StatusCode};
use actix_session::CookieSession;
use actix_files as fs;
use actix_redis::RedisActor;
use actix_web::web::{ServiceConfig, route};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	// info trace debug，其实这里可以给出warn或error级别的等级，
	std::env::set_var("RUST_LOG", "actix_web=info, actix_server=info");
	env_logger::init();

	// 所有访问共享的数据，且会加锁访问，这里的Data其实就是Arc的re-export
	let counter = web::Data::new(Mutex::new(0usize));
	HttpServer::new(move || {
		// redis（TODO 这个必须写到里面，写到外面，比如counter的位置会报错，暂不知道为什么）
		// TODO 好像是说在里面创建的对象且设置为data，则redis_addr是ThreadLocal的对象？
		let redis_addr = RedisActor::start("localhost:6379");
		let manager = ConnectionManager::<PgConnection>::new("postgres://postgres:wyzpass@127.0.0.1/db_test");
		let pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool");

		App::new()
			// 可以将service之类的定义放到configure方法中（该方法类似XxConfig对象，包括了所有的配置），不过用处不是很大
			.configure(|cfg: &mut ServiceConfig| {
				//cfg.service().data().external_resource().route()
				cfg.service(route_set::validate_test)
					// 无权限
					.service(web::resource("/405").to(|req: HttpRequest| {
						HttpResponse::Unauthorized().body("抱歉，您没有权限访问哦～")
					}));
			})
			.app_data(counter.clone())
			// 开启压缩（默认的貌似是gzip？）
			.wrap(awmw::Compress::default())
			.wrap(awmw::Logger::default())
			// 这个貌似也能实现middleware的功能，传一个lambda表达式去
			//.wrap_fn(..)
			// 只需要结构体名即可（不是传名字即可，而是这个结构体很特别不需要{}也相当于new了对象，不过最好还是加上{}）
			// TODO 注意，middleware的处理一定是在handler处理前和处理后处理
			.wrap(middleware::SayHi{})
			// 是指为session分配32个字节？【返回的响应头里会有set-cookie: actix-session=xxxxxx一大串（貌似就是32字节）
			.wrap(CookieSession::signed(&[0; 32]).secure(false))
			// 貌似是指JSON数据最大不超过4096个字节？？（但是用8测试了下好像没有生效，还是说虽然填了8但是实际上它有个最小值?）
			.data(web::JsonConfig::default().limit(4096))
			// 还是用第三方的通用的好一点，不用actix的
			.data(Client::default())
			.data(redis_addr)
			.data(pool.clone())
			// 虽然可以直接用route，但是最好还是外部包一层service
			.route("/name/{name}/gender/{gender}", web::get().to(route_set::index))
			.service(web::resource("/ttt").route(web::get().to(route_set::foo)
				.guard(custom_guard::MyHeaderGuard{head_val: "text/plain".to_owned()})))
			// 不用route直接to表示任意Method都行，比如GET，HEAD，PUT；一般用于如logout这样的方法
			.service(web::resource("/kkkk").to(route_set::kkkk)
				// TODO guard::Header(...)表示请求必须是携带这个header，不过不好的是，值必须是application/json不能有;charset=uf8
				// TODO 所以其实不适合用guard做Header的校验，还是自己写个middleware来做，或者在对于方法里通过request去判断
				.guard(guard::Header("Content-Type", "application/json")))
			.service(web::resource("/uuuu").route(web::get().to(route_set::uuuu)))
			.service(route_set::book)
			.service(web::resource("/post1").route(web::post().to(route_set::post1)))
			.service(route_set::put1)
			.service(route_set::stud_post)
			.service(route_set::stud2_post)
			.service(route_set::book2)
			.service(route_set::test1)
			.service(route_set::test2)
			// 错误的例子，尽管编译期间不报错
			.service(route_set::test4)
			.service(route_set::test12)
			.service(route_set::test5)
			.service(route_set::test6)
			.service(route_set::favicon)
			.service(route_set::welcome)
			.service(route_set::redis_set)
			.service(route_set::redis_test)
			.service(route_set::redis_del)
			.service(route_set::redis_hset)
			.service(route_set::do_something)
			.service(route_set::custom_error)
			.service(route_set::custom_error2)
			.service(route_set::db_diesel)
			.service(route_set::db_diesel_post)
			.service(web::resource("/test_lambda").to(|req: HttpRequest| match *req.method() {
				Method::GET => HttpResponse::Ok(),
				Method::POST => HttpResponse::MethodNotAllowed(),
				_ => HttpResponse::NotFound(),
			}))
			// 展示一个静态文件列表的页面（这个页面应该是actix根据自己的规则写的一个ul列表的页面
			.service(fs::Files::new("/static", "static").show_files_listing())
			// 重定向，TODO 注意，actix-web能处理这种请求http://localhost:8088index，它等同于8088/index（很多框架都这么处理）
			// 同样的8088//index等同于8088/index（甚至///index）
			.service(web::resource("/").route(web::get().to(|req: HttpRequest| {
				// 重定向到/index请求（这里/可以不要）
				HttpResponse::Found().header(header::LOCATION, "/index").finish()
			})))
			// 这个可以直接将/定位为从static目录获取index.html文件，上面的redirect还必须先写一个/index的方法
			//.service(fs::Files::new("/", "./static/").index_file("index.html"))
			.service(
				// 类似Java里的UserController上面的RequestMapping("/user")
				web::scope("/user")
					// TODO rust规范里，如果某个变量不用到，但是又不能drop那么快，则应该以下划线开头（如果是_则在赋值后立刻就会drop可能导致问题）
					.service(web::resource("/test1").route(web::get().to(|_req: HttpRequest| {
						HttpResponse::Ok().body("aaa")
					})))
					.service(web::resource("/test2").route(web::get().to(|req: HttpRequest| {
						HttpResponse::Ok().body("bbb")
					})))
			)
			.default_service(
				// 这里的""貌似是指没有匹配到的意思？而不是说http://localhost:8088（没有最后的/）的意思
				// TODO 经过测试""确实是指没有找到匹配的路径(else)，http://localhost:8088是会匹配"/"的
				// 但是上面已经是default_service()，所以这里其实可以不用给出resource？（可以参考下面注释掉的default_service）
				web::resource("")
					// 当没有匹配到但是是get请求，则返回404页面
					.route(web::get().to(route_set::p404))
					// 没有匹配到但是是非get请求
					// TODO 所以说一个resource请求可以有多个对应的route（或者说resource+route才是具体的匹配，但是route里面兼具提供处理方案
					.route(
						// route内部还能有route？
						// TODO guard其实就是类似route，只不过它是更加细化的匹配，除了匹配Method外还能匹配Content-Type之类的
						// 比如 guard::All(guard::Get()).and(guard::Header("content-type", "plain/text"))
						web::route()
							.guard(guard::Not(guard::Get()))
							.to(HttpResponse::MethodNotAllowed),
					),
			)
		//.default_service(web::get().to(route_set::other))
	}).bind("127.0.0.1:8088")?.run().await
}
