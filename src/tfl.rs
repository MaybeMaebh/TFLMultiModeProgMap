use std::collections::HashMap;

use time::OffsetDateTime;

use std::fs::{self, File};
use std::io::{self};
use std::path::{Path, PathBuf};

pub fn init(){
    //add bit to create cache folder if dosent exist
    let cache_path = Path::new("cache");
    if !cache_path.exists(){
        println!("creating:{:?}",cache_path);
        fs::create_dir(cache_path).unwrap();
    }
     
    println!("{:?}",cached_req_to_string("Line/Route"));
}

pub struct TflData<'a>{
    pub stop_point_sequences:HashMap<String, StopPointSequence<'a>>,
}

pub struct StopPointSequence<'a> {
    name:String,
    direction:Direction,
    originator:&'a StopPoint<'a>,
    destination:&'a StopPoint<'a>,
    line:&'a Line<'a>,
    reverse:Option<&'a StopPointSequence<'a>>,
    service_type:ServiceType
}

pub struct StopPoint<'a> {
    stop_point_sequences:Vec<&'a StopPointSequence<'a>>,//duplaciitivitve technically
    parent:Option<&'a StopPoint<'a>>,
    children:Option<&'a StopPoint<'a>>,
    hereditary_stop_point_sequences:Vec<&'a StopPointSequence<'a>>,//duplaciitivitve deffinitly
}

pub struct Line<'a>{
    mode:&'a Mode,
    name: String,
    stop_point_sequences:Vec<&'a StopPointSequence<'a>>,//duplaciitivitve technically
    tfl_modifide:OffsetDateTime
}

pub struct  Mode{
    name: String
}

pub enum Direction{
    Outbound,
    Inbound
}

pub enum ServiceType{
    Regular,
    Night
}

fn cached_req_to_string(addr: &str) -> File {
    let file_path = addr_to_path(addr);
    if file_path.exists() {
        File::open(file_path).expect("filepath is ok but file cannot be oppened")
    } else {
        println!("getting:{}","https://api.tfl.gov.uk/".to_owned() + addr);
        update_cached_req(addr).unwrap();
        cached_req_to_string(addr)
    }
}

fn addr_to_path(addr: &str) -> PathBuf{
    Path::new("cache").join(addr.replace("/", "し") + ".json")
}

fn update_cached_req(addr: &str) -> Result<(), ureq::Error>{
    let res = ureq::get(&("https://api.tfl.gov.uk/".to_owned() + addr)).call()?;
    let file_path = addr_to_path(addr);
    let mut file = File::create(file_path)?;
    //let buffer: &mut [u8] = &mut [];
    println!("satus:{}",res.status_text());
    //println!("str:{}", res.into_string()?);
    //res.into_reader().read(buffer)?;
    // println!("len:{}",buffer.len());
    // file.write_all(buffer).unwrap();
    io::copy(&mut res.into_reader(), &mut file)?;
    Ok(())
}