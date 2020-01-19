use std::pin::Pin;
use std::task::{Context, Poll};
use actix_service::{Service, Transform};
// TODO 这里的dev不应该理解成发布环境的dev，而是指用于开发阶段的开发工具包（但是感觉还是应该用其他包名好一点。。）
use actix_web::{http, dev::ServiceRequest, dev::ServiceResponse, Error, HttpResponse, HttpRequest};
use futures::future::{ok, Ready, Either};
use futures::Future;
use std::borrow::Borrow;
use actix_web::middleware::errhandlers::ErrorHandlerResponse::Response;
use std::any::Any;
use actix_http::error::{ErrorBadRequest, ErrorUnauthorized};

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct SayHi;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for SayHi
	where
		S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
		S::Future: 'static,
		B: 'static,
{
	type Request = ServiceRequest;
	type Response = ServiceResponse<B>;
	type Error = Error;
	// 和上面的Transform是两码事
	type Transform = SayHiMiddleware<S>;
	type InitError = ();
	type Future = Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		ok(SayHiMiddleware { service })
	}
}

pub struct SayHiMiddleware<S> {
	service: S,
}

impl<S, B> Service for SayHiMiddleware<S>
	where
		S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
		S::Future: 'static,
		B: 'static,
{
	type Request = ServiceRequest;
	type Response = ServiceResponse<B>;
	type Error = Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.service.poll_ready(cx)
	}

	fn call(&mut self, req: ServiceRequest) -> Self::Future {
		// TODO 这里其实可以做权限认证
		// TODO 处理请求前的处理（类似FilterChain中的Filter）
		println!("Hi from start. You requested: {}", req.path());
		// 这个鬼地方弄了N久都不知道怎么直接返回不直接执行后面的逻辑，去它妹的；
		// TODO 看了下官网的例子，貌似是通过定义一个返回没有权限的handler，然后发现没有权限则转发到这里去处理。。
		if !req.headers().contains_key("Authorization") {
			// async {}类似Future::new(..)一样，不过Future没有new方法所以用这种方式创建
			// ，加move是要当需要形成闭包时才用；如果是lambda则是类似|| async {}
			return Box::pin(async move {
				// 这种方式有点粗暴，直接http code返回了401，其实自己想返回200，但是ApiResult里再细分；
				// 毕竟这个是Api接口而不一定是网页打开（可以了，原来少了个into_body()..
				//Err(Error::from(ErrorUnauthorized("抱歉，您没有权限访问哦啦啦～～")))
				Ok(req.into_response(
					HttpResponse::Ok().body("抱歉您没有权限啦啦哦哦～").into_body()
				))
				//Either::Right()
			});
		}

		// 是不是可以通过type_id来实现判断是否是需要某个逻辑从而执行特定逻辑？（但是type_id怎么获取是个很大的问题。。）
		// self.service.type_id()
		let fut = self.service.call(req);

		Box::pin(async move {
			let res = fut.await?;

			// TODO 处理请求后的处理
			println!("Hi from response");
			Ok(res)
		})
	}
}
