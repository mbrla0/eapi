#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate hyper_native_tls;
mod objects;

pub mod e;
pub use e::Sources;

use std::io::Read;
pub trait Post{
	fn id(&self)		-> u64;
	fn data(&self)		-> Result<Box<Read>, Error>;
	fn data_ext(&self)	-> Option<&str>;
}

pub trait Pool<P: Post>: Iterator<Item = Result<P, Error>>{
	fn title(&self)			-> String;
	fn description(&self)	-> String;
}

pub trait Source{
	type Post:	Post;
	type Query:	Iterator<Item = Result<Self::Post, Error>>;
	type Pool:	Pool<Self::Post>;

	fn query(&self, query: &str)
		-> Option<Self::Query>;
	fn pool(&self, id: u64)
		-> Result<Self::Pool, Error>;
	fn post(&self, id: u64)
		-> Result<Self::Post, Error>;
}

fn user_agent() -> String{
	format!("{}/{} ({})",
		env!("CARGO_PKG_NAME"),
		env!("CARGO_PKG_VERSION"),
		env!("CARGO_PKG_AUTHORS"))
}

#[derive(Debug)]
pub enum Error{
	TlsError,
	HyperError(hyper::Error),
	ParseError(serde_json::Error)
}


