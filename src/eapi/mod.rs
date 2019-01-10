#![allow(dead_code)]

/// In accordance to Notice #2 on the e621 API, a non-empty user must be
/// used for requests, otherwise, the request will result in error 403,
/// or "Forbidden".
use hyper::header::UserAgent;
fn get_user_agent() -> UserAgent{
    UserAgent("eapi-rs/1.0 (DarkRyu550)".to_owned())
}

/// Type used to identificate objects
/// NOTE: Not yet in use.
pub enum Identification{
    Id(u64),
    Md5(String)
}

/// Contains all JSON objects used by the API
pub mod objects;

use std::error::Error as StdError;
#[derive(Debug)]
pub enum Error{
    /// Could not find the object that was asked for
    NotFound,
    /// The expected number of posts is not the same as the actual number of posts in a pool
    PoolSizeMismatch(u64, u64),
    /// Error retrieving structure from the source
    RetrieveError(Box<StdError>)
}

use hyper::Client;
/// Creates a new Client
fn get_client() -> Result<Client, Error>{
    // Create a new client, using the default system way of connecting to
    // a possible HTTPS connection
    use hyper_native_tls::NativeTlsClient;
    use hyper::net::HttpsConnector;
    let tls = try!(NativeTlsClient::new().map_err(|what|
        Error::RetrieveError(Box::new(what))
    ));
    let connector = HttpsConnector::new(tls);
    Ok(Client::with_connector(connector))
}

/// Generates a URL for a source compatible with this API
pub trait Source: Sized{
    /// Number used for indexing the first element in a list, such as 0 or 1
    fn index_start(&self) -> u64;

    /// Retrieves a Post from the source
    fn post<'a>(&'a self, id: u64) -> Result<Post<'a, Self>, Error>;

    /// Retrieves the comment pool from a Post
    fn comment_pool<'a>(&'a self, id: u64) -> Result<Vec<Comment<'a, Self>>, Error>;

    /// Retrieves a Pool from the source
    fn pool<'a>(&'a self, id: u64) -> Result<Pool<'a, Self>, Error>;

    /// Retrieves User's data from the source
    fn user<'a>(&'a self, id: u64) -> Result<User<'a, Self>, Error>;
}

#[derive(Copy, Clone, Debug)]
pub enum Sources{
    /// Popular furry-related art repository
    E621,
    /// SFW-only version ef e621, uses the same API
    E926
}
impl Source for Sources{
    fn index_start(&self) -> u64{
        match self{
            &Sources::E621 => 1,
            &Sources::E926 => 1
        }
    }

    fn post<'a>(&'a self, id: u64) -> Result<Post<'a, Self>, Error>{
        let url = match self{
            &Sources::E621 => format!("http://e621.net/post/show.json?id={}", id),
            &Sources::E926 => format!("http://e926.net/post/show.json?id={}", id)
        };

        // Try to download the requested post
        let client = try!(get_client());
        let page = try!(
            client.get(&url)
            .header(get_user_agent())
            .send()
            .map_err(|what| Error::RetrieveError(Box::new(what)))
        );

        // If successful, try to deserialize it into a Page
        use serde_json::from_reader;
        let object = try!(from_reader(page).map_err(|what| Error::RetrieveError(Box::new(what))));
        Ok(Post{
            source: self,
            object: object
        })
    }

    fn comment_pool<'a>(&'a self, id: u64) -> Result<Vec<Comment<'a, Self>>, Error>{
        let url = |id, page| match self{
            &Sources::E621 => format!("http://e621.net/comment/index.json?post_id={}&page={}", id, page),
            &Sources::E926 => format!("http://e926.net/comment/index.json?post_id={}&page={}", id, page)
        };

        let get_pool = move |client: &Client, url: &str| -> Result<objects::CommentPool, Error>{
            // Try to open a connection to the required object
            let page = try!(
                client.get(url)
                .header(get_user_agent())
                .send()
                .map_err(|what| Error::RetrieveError(Box::new(what)))
            );

            // If successful, try to deserialize it
            use serde_json::from_reader;
            from_reader(page).map_err(|what| Error::RetrieveError(Box::new(what)))
        };

        /// Checks if a given page is the tail of its pool
        let tail = |pool: &objects::CommentPool|{
            pool.0.len() == 0
            /* Check e621-limits if enabled */
            || if cfg!(feature = "e621-limits") { pool.0.len() != 25 } else { false }
        };

        let client = try!(get_client());
        let mut object: objects::CommentPool = try!(get_pool(&client, &url(id, self.index_start())));
        if !tail(&object){
            // Try to fill using the remaining comment pages
            for i in self.index_start()..{
                let mut page: objects::CommentPool = try!(get_pool(&client, &url(id, i)));
                match tail(&page){
                    true  => { object.0.append(&mut page.0); break } ,
                    false => { object.0.append(&mut page.0) }
                }
            }
        }

        Ok(object.0.into_iter().map(|object| Comment{
            source: self,
            object: object
        }).collect())
    }

    fn pool<'a>(&'a self, id: u64) -> Result<Pool<'a, Self>, Error>{
        let url = |id, page| match self{
            &Sources::E621 => format!("http://e621.net/pool/show.json?id={}&page={}", id, page),
            &Sources::E926 => format!("http://e926.net/pool/show.json?id={}&page={}", id, page)
        };

        let get_pool = |client: &Client, url: &str| -> Result<objects::Pool, Error> {
            // Try to download the requested pool information using the given Client
            let page = try!(
                client.get(url)
                .header(get_user_agent())
                .send()
                .map_err(|what| Error::RetrieveError(Box::new(what)))
            );

            // If successful, try to deserialize it into a Pool
            use serde_json::from_reader;
            from_reader(page).map_err(|what| Error::RetrieveError(Box::new(what)))
        };

        let client = try!(get_client());
        let mut object = try!(get_pool(&client, &url(id, self.index_start())));
        for i in self.index_start() + 1..{
            if (object.posts.len() as u64) < object.post_count {
                println!("Count: {}, Posts: {}", object.post_count, object.posts.len());
                // Try to get the next pool page and append it
                let mut appendix = try!(get_pool(&client, &url(id, i)));
                if appendix.posts.len() != 0{
                    object.posts.append(&mut appendix.posts);
                }else{
                    return Err(Error::PoolSizeMismatch(object.post_count, object.posts.len() as u64))
                }
            }else{
                break
            }
        }

        Ok(Pool{
            source: self,
            object: object
        })
    }

    fn user<'a>(&'a self, id: u64) -> Result<User<'a, Self>, Error>{
        let url = match self{
            &Sources::E621 => format!("http://e621.net/user/index.json?id={}", id),
            &Sources::E926 => format!("http://e926.net/user/index.json?id={}", id)
        };

        // Connect to the user index
        let client = try!(get_client());
        let connection = try!(
            client.get(&url)
            .header(get_user_agent())
            .send()
            .map_err(|what| Error::RetrieveError(Box::new(what)))
        );

        use serde_json::from_reader;
        let mut users: Vec<objects::User> = try!(from_reader(connection).map_err(|what| Error::RetrieveError(Box::new(what))));
        let user = match users.len(){
            0 => return Err(Error::NotFound),
            _ => users.remove(0)
        };

        // Construct from raw object
        Ok(User{
            source: self,
            object: user
        })
    }
}

/// A value rating the "safety" of a post, the higher the safety, the lesser the value
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Rating{
    Safe,
    Questionable,
    Explicit
}

pub struct Image{
    /// Source URL
    pub source_url: String,
    /// File extention
    pub extention: String,

    /* Dimentions */
    pub width:  u64,
    pub height: u64
}
impl Image{
    /// Tries to open a read stream to the image
    pub fn try_open(&self) -> Result<impl Read, Error>{
        let client = try!(get_client());
        client.get(&self.source_url)
            .header(get_user_agent())
            .send()
            .map_err(|what| Error::RetrieveError(Box::new(what)))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Level{
    Blocked       = 10,
    Member        = 20,
    Privileged    = 30,
    Contributor   = 33,
    Janitor       = 35,
    Moderator     = 40,
    Administrator = 50,
}

use std::borrow::Cow;
#[derive(Debug)]
pub struct User<'a, S: Source + 'a>{
    source: &'a S,
    object: objects::User
}
impl<'a, S: Source> User<'a, S>{
    pub fn id(&self) -> u64{
        self.object.id
    }

    pub fn name(&self) -> &str{
        self.object.name.as_str()
    }

    /* TODO: Implement a higher-level datetime type */
    pub fn registered_at(&self) -> &str{
        self.object.created_at.as_str()
    }

    pub fn level(&self) -> Level{
        match self.object.level{
            10 => Level::Blocked,
            20 => Level::Member,
            30 => Level::Privileged,
            33 => Level::Contributor,
            35 => Level::Janitor,
            40 => Level::Moderator,
            50 => Level::Administrator,
            _ => panic!("User has invalid level {}", self.object.level)
        }
    }

    pub fn stats(&self) -> &objects::UserStats{
        &self.object.stats
    }

    pub fn avatar(&self) -> Result<Post<'a, S>, Error>{
        self.source.post(self.object.avatar_id)
    }
}
impl<'a, S: Source> User<'a, S>{
    pub fn from_raw(source: &'a S, object: objects::User) -> User<'a, S>{
        User{
            source: source,
            object: object
        }
    }

    pub fn to_raw(self) -> objects::User{
        self.object
    }
}

#[derive(Debug)]
pub struct Comment<'a, S: Source + 'a>{
    source: &'a S,
    object: objects::Comment
}
impl<'a, S: Source + 'a> Comment<'a, S>{
    pub fn id(&self) -> u64{
        self.object.id
    }

    pub fn body(&self) -> &str{
        &self.object.body
    }

    pub fn creator_username(&self) -> &str{
        &self.object.creator
    }

    pub fn timestamp(&self) -> &str{
        &self.object.created_at
    }

    /// The Post this comment belongs to
    pub fn parent(&self) -> Result<Post<'a, S>, Error>{
        self.source.post(self.object.post_id)
    }

    /// The User who posted this comment
    pub fn creator(&self) -> Result<User<'a, S>, Error>{
        self.source.user(self.object.creator_id)
    }
}

use std::io::Read;
#[derive(Debug)]
pub struct Post<'a, S: Source + 'a>{
    source: &'a S,
    object: objects::Post
}
impl<'a, S: Source> Post<'a, S>{
    /// Tries to open a read stream to this post's file
    pub fn file(&self) -> Image{
        Image{
            source_url: self.object.file_url.clone(),
            extention:  self.object.file_ext.clone(),
            width:  self.object.width,
            height: self.object.height,
        }
    }

    /// Tries to open a read stream to the preview of the post's file
    pub fn preview(&self) -> Image{
        Image{
            source_url: self.object.preview_url.clone(),
            extention:  "jpg".to_owned(),
            width:  self.object.preview_width,
            height: self.object.preview_height,
        }
    }

    /// Tries to open a read stream to the sample of the post's file
    pub fn sample(&self) -> Image{
        Image{
            source_url: self.object.sample_url.clone(),
            extention:  "jpg".to_owned(),
            width:  self.object.sample_width,
            height: self.object.sample_height,
        }
    }

    pub fn rating(&self) -> Rating{
        match self.object.rating.as_str(){
            "s" => Rating::Safe,
            "q" => Rating::Questionable,
            "e" => Rating::Explicit,
            r @ _ => panic!("Post has an unknown rating value \"{}\"", r)
        }
    }

    pub fn tags<'b>(&'b self) -> impl Iterator<Item=&'b str> + 'b{
        self.object.tags.split(" ")
    }

    pub fn parent(&self) -> Option<Post<'a, S>>{
        match self.object.parent_id{
            Some(parent_id) => self.source.post(parent_id).ok(),
            None => None
        }
    }

    pub fn children<'b>(&'b self) -> impl Iterator<Item = Post<'b, S>> + 'b{
        self.object.children.split(",").filter_map(move |id|{
            match u64::from_str_radix(id, 10){
                Ok(id) => self.source.post(id).ok(),
                Err(_) => None
            }
        })
    }

    pub fn comments<'b>(&'b self) -> Result<impl Iterator<Item = Comment<'b, S>> + 'b, Error>{
        // Build the comment pool from which the iterator will be built
        Ok(try!(self.source.comment_pool(self.object.id)).into_iter())
    }
}

#[derive(Debug)]
pub struct Pool<'a, S: Source + 'a>{
    source: &'a S,
    object: objects::Pool
}
impl<'a, S: Source> Pool<'a, S>{
    pub fn is_complete(&self) -> bool{
        self.object.post_count == self.object.posts.len() as u64
    }

    pub fn posts<'b>(&'b self) -> impl Iterator<Item = Post<'b, S>> + 'b{
        self.object.posts.iter().cloned().map(move |post_object| Post{
            source: self.source,
            object: post_object
        })
    }

    pub fn creator(&self) -> Result<User<'a, S>, Error>{
        self.source.user(self.object.user_id)
    }
}
