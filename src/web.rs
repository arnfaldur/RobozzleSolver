use std::io::{stdout, Write};

use curl::easy::{Easy, Form};
use curl::*;

pub fn start_web_client() {

    let mut handle = Easy::new();
    let url = "http://www.robozzle.com/beta/index.html?puzzle=1874";
    handle.url(url).unwrap();
    handle.write_function(|data| {
        println!("Writing a thing: ");
        stdout().write_all(data).unwrap();
        println!("Done writing a thing.");
        Ok(data.len())
    }).unwrap();
    handle.perform().unwrap();
    let mut form = Form::new();
    form.part("asdf").add();
    handle.httppost(form);


//    let mut data = Vec::new();
//    let mut handle = Easy::new();
//    handle.url("https://www.rust-lang.org/").unwrap();
//    let mut transfer = handle.transfer();
//    transfer.write_function(|new_data| {
//        data.extend_from_slice(new_data);
//        Ok(new_data.len())
//    }).unwrap();
//    transfer.perform().unwrap();
//    println!("{:?}", data);

}