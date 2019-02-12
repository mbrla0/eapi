use ::{objects, user_agent, Error, Source};

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
	type Post	= Post;
	type Pool	= Pool;
	type Query	= Query;

	fn query(&self, query: &str) 
		-> Option<Query>{
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
		Some(match *self{
			Sources::E621 => {
				let url_e621 = move |p: u32| {
					format!(
						"https://e621.net/post/index.json?tags={}&page={}", 
						qstr,
						p)
				};

				Query{
					url: Box::new(url_e621),
					page: 0,
					buffer: Vec::new()
				}
			},
			Sources::E926 => {
				let url_e926 = move |p: u32| {
					format!(
						"https://e926.net/post/index.json?tags={}&page={}",
						qstr,
						p)
				};

				Query{
					url: Box::new(url_e926),
					page: 0,
					buffer: Vec::new()
				}
			}
		})
	}

	fn pool(&self, id: u64) 
		-> Option<Pool>{
		let qstr = format!("{}", id);
		Some(match *self{
			Sources::E621 => {
				let url_e621 = move |p: u32| {
					format!(
						"https://e621.net/pool/show.json?id={}&page={}", 
						qstr,
						p)
				};

				let mut a = Pool{
					url: Box::new(url_e621),
					page: 0,
					obj: None,
				};
				/* TODO: Change this to be more correct. */
				a.retrieve().unwrap();

				a
			},
			Sources::E926 => {
				let url_e926 = move |p: u32| {
					format!(
						"https://e926.net/pool/show.json?id={}&page={}",
						qstr,
						p)
				};

				let mut a = Pool{
					url: Box::new(url_e926),
					page: 0,
					obj: None
				};
				a.retrieve().unwrap();

				a
			}
		})
	}

	fn post(&self, id: u64)
		-> Option<Post>{
		
		unimplemented!()
	}
}

pub struct Query{
	url: 	Box<Fn(u32) -> String>,
	page: 	u32,
	buffer:	Vec<objects::Post>
}
impl Iterator for Query{
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
		
		if self.buffer.len() > 0{
			self.buffer.drain(0..1).next()
				.map(|post| Ok(Post(post)))
		} else { None }
	}
}

use ::Pool as IPool;
pub struct Pool{
	url:	Box<Fn(u32) -> String>,
	page:	u32,
	obj:	Option<objects::Pool>
}
impl Pool{
	fn retrieve(&mut self) -> Result<(), Error>{
		self.page += 1;
		
		match get_client()
			.and_then(|c|{
				let url = (self.url)(self.page);
				get_request(&c, &url)
					.send().map_err(|e| Error::HyperError(e))
			})
			.and_then(|r|
				serde_json::from_reader::<_, objects::Pool>(r)
					.map_err(|e| Error::ParseError(e))
			){
			Ok(obj) => self.obj = Some(obj),
			Err(what) => return Err(what)
		}

		Ok(())
	}
}
impl IPool<Post> for Pool{
	fn title(&self) -> String{
		unimplemented!()
	}
	fn author(&self) -> String{ "TODO".to_owned() }
}
impl Iterator for Pool{
	type Item = Result<Post, Error>;

	fn next(&mut self) -> Option<Result<Post, Error>>{
		if let None = self.obj { 
			if let Err(what) = self.retrieve(){
				return Some(Err(what))
			}
		}

		let len = self.obj.as_mut().unwrap().posts.len();
		if len == 0 {
			if let Err(what) = self.retrieve(){
				return Some(Err(what))
			}
		}
		let obj = self.obj.as_mut().unwrap();

		if obj.posts.len() > 0{
			obj.posts.drain(0..1).next().map(|post| Ok(Post(post)))
		} else { None }
	}
}

use std::io::Read;
use ::Post as IPost;
pub struct Post(objects::Post);
impl IPost for Post{
	fn data(&self) -> Result<Box<Read>, Error>{
		let client = get_client()?;
		
		get_request(&client, &self.0.file_url).send()
			.map(|read| Box::new(read) as Box<Read>)
			.map_err(|e| Error::HyperError(e))
	}

	fn data_ext(&self) -> Option<&str>{
		self.0.file_ext.as_ref().map(|a| a.as_str())
	}
}
