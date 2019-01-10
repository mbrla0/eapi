#![feature(conservative_impl_trait)]

extern crate hyper;
extern crate hyper_native_tls;

extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

mod eapi;

use eapi::Rating;
enum Flags{
    /// Ignore posts with rating greater than the given one
    FilterGt(Rating),
    /// Ignore posts with rating greater or equal to the given one
    FilterGe(Rating),
    /// Ignore posts with rating lesser than the given one
    FilterLt(Rating),
    /// Ignore posts with rating lesser or equal to the given one
    FilterLe(Rating),
}

// enum Filter{
//     Gt(Rating), // >
//     Ge(Rating), // >=
//     Lt(Rating), // <
//     Le(Rating)  // <=
// }
//
// struct Operation{
//     mode: Mode,
//     filter: Option<Filter>,
//
// }

fn main() {
    // Greeting
    println!("EAPI Downloader {} - Download your content from e621!", env!("CARGO_PKG_VERSION"));
    use eapi::{Source, Sources};
    let source = Sources::E621;
    let cinderfrost = source.pool(587).unwrap();

    let mut index: usize = 1;
    for post in cinderfrost.posts(){
        // Ignore Explicit
        use eapi::Rating;
        if post.rating() < Rating::Explicit{
            use std::fs::File;
            use std::path::PathBuf;

            let file = post.file();
            let path = PathBuf::from(format!("{}.{}", index, file.extention));
            if !path.exists(){
                let mut target = File::create(path).unwrap();

                // Download the file onto target
                use std::io::copy;
                copy(&mut file.try_open().unwrap(), &mut target).unwrap();
            }

            // // Save the comments as well, if possible
            // let path = PathBuf::from(format!("{}_Comments.txt", index));
            // if !path.exists(){
            //     let mut target = File::create(path).unwrap();
			//
            //     for comment in post.comments().unwrap(){
            //         use std::io::Write;
            //         writeln!(target, "[Posted on {} by {}]", comment.timestamp(), comment.creator_username());
            //         writeln!(target, "{}", comment.body());
            //         writeln!(target, "");
            //     }
            // }
        }
        index += 1;
    }
}
