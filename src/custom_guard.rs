use actix_web::guard::Guard;
use actix_http::RequestHead;

pub struct MyHeaderGuard {
	pub head_val: String
}

// 如果不包含application/json则失败
impl Guard for MyHeaderGuard {
	fn check(&self, request: &RequestHead) -> bool {
		// 这里的extensions不是.html的意思，而是貌似是一个Attribute用来存储一些request scope的数据。。
		// version是指http的协议版本，peer_addr是客户端的地址
		println!("extensions:{:?}##version:{:?}##peer_addr:{:?}"
				 , request.extensions
				 , request.version
				 , request.peer_addr);
		// TODO 注意，Key是不区分大小写的，所以客户端里的是content-type也是可以的
		if request.headers.contains_key("Content-Type") {
			let head_val = request.headers.get("Content-Type").unwrap().to_str().unwrap().to_lowercase();
			if head_val.contains(&self.head_val) {
				true
			} else {
				false
			}
		} else {
			false
		}
	}
}
