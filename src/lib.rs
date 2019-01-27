#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate hyper_native_tls;
mod objects;

pub trait Source{
	/* TODO: Change to impl Trait when allowed. */
	fn query(&self, query: &str)
		-> Box<Iterator<Item = Result<Post, Error>>>;
	fn pool(&self, id: u64)
		-> Box<Iterator<Item = Result<Post, Error>>>;
	fn post(&self, id: u64)
		-> Option<Post>;
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

fn get_client() -> Result<hyper::Client, Error>{
	use hyper_native_tls::NativeTlsClient;
	let ntls = NativeTlsClient::new()
		.map_err(|_| Error::TlsError)?;
	
	use hyper::net::HttpsConnector;
	let conn = HttpsConnector::new(ntls);

	Ok(hyper::Client::with_connector(conn))
}

use hyper::client::{RequestBuilder, IntoUrl};
fn get_request<'a, U: IntoUrl>(
	client: &'a hyper::Client,
	url: U
	) -> RequestBuilder<'a>{
	
	use hyper::header::UserAgent;
	client.get(url)
		.header(UserAgent(user_agent()))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sources{ E621, E926 }
impl Source for Sources{
	fn query(&self, query: &str) 
		-> Box<Iterator<Item = Result<Post, Error>>>{
		/* Change the query string into a more URL-friendly representation
		 * by swapping non-ascii characters with their hexadecimal format.
		 * The hexadecimal format is encoded in UTF-8.
		 */
		let qstr: String = query.chars()
			.map(|c| {
			
			if c.is_ascii_alphanumeric(){
				let mut string = String::new();
				string.push(c);
				string
			} else {
				let mut buffer = Vec::new();
				buffer.resize(c.len_utf8(), 0_u8);
				let _ = c.encode_utf8(&mut buffer[..]);
				
				buffer.into_iter()
					.map(|b| format!("%{:X}", b))
					.collect()
			}

		}).collect();
		match *self{
			Sources::E621 => {
				let url_e621 = move |p: u32| {
					format!(
						"https://e621.net/post/index.json?tags={}&page={}", 
						qstr,
						p)
				};

				Box::new(Query{
					url: url_e621,
					page: 0,
					buffer: Vec::new()
				}) as Box<Iterator<Item = Result<Post, Error>>>
			},
			Sources::E926 => {
				let url_e926 = move |p: u32| {
					format!(
						"https://e926.net/post/index.json?tags={}&page={}",
						qstr,
						p)
				};

				Box::new(Query{
					url: url_e926,
					page: 0,
					buffer: Vec::new()
				}) as Box<Iterator<Item = Result<Post, Error>>>
			}
		}
	}

	fn pool(&self, id: u64) 
		-> Box<Iterator<Item = Result<Post, Error>>>{
		unimplemented!()
	}

	fn post(&self, id: u64)
		-> Option<Post>{
		
		unimplemented!()
	}
}

struct Query<F: Fn(u32) -> String>{
	url: 	F,
	page: 	u32,
	buffer:	Vec<objects::Post>
}
impl<F: Fn(u32) -> String> Iterator for Query<F>{
	type Item = Result<Post, Error>;
	
	fn next(&mut self) -> Option<Result<Post, Error>>{
		if self.buffer.len() == 0{
			self.page += 1;
			
			match get_client()
				.and_then(|c|{
					let url = (self.url)(self.page);
					get_request(&c, &url)
						.send().map_err(|e| Error::HyperError(e))
				})
				.and_then(|r|
					serde_json::from_reader(r)
						.map_err(|e| Error::ParseError(e))
				){
				Ok(buffer) => self.buffer = buffer,
				Err(what) => return Some(Err(what))
			}
		}

		self.buffer.pop()
			.map(|post| Ok(Post(post)))
	}
}

use std::io::Read;
pub struct Post(objects::Post);
impl Post{
	pub fn file(&self) -> Result<impl Read, Error>{
		let client = get_client()?;
		
		get_request(&client, &self.0.file_url).send()
			.map_err(|e| Error::HyperError(e))
	}

	pub fn file_ext(&self) -> Option<&str>{
		self.0.file_ext.as_ref().map(|a| a.as_str())
	}
}
