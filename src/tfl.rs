use std::str::FromStr;


use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::vec;

use ahash::{AHasher, HashMap, RandomState};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{from_reader, to_writer};

pub fn init(){
    //add bit to create cache folder if dosent exist
    let cache_path = Path::new("cache");
    if !cache_path.exists(){
        println!("creating:{:?}",cache_path);
        fs::create_dir(cache_path).unwrap();
    }
    

    //create hashmaps to hold data
    let mut out = TflData{
        stop_point_sequences: HashMap::with_capacity_and_hasher(2000,RandomState::new()),
        stop_points: HashMap::with_capacity_and_hasher(200_000, RandomState::new()),
        lines: HashMap::with_capacity_and_hasher(1000, RandomState::new()),
        modes: HashMap::with_capacity_and_hasher(20, RandomState::new()),
        zones: HashMap::with_capacity_and_hasher(20, RandomState::new())
    };

    //update_cached_req("Line/Route");
    let line_list: Vec<ApiLineRef> = from_reader(BufReader::new(cached_req("Line/Route"))).unwrap();
    for api_line_ref in line_list{
        #[cfg(debug_assertions)]
        {
            println!("id:{}",api_line_ref.id);
            if api_line_ref.id == "110" {
                std::process::exit(0);
            }
        }

        //check if the reported last time line was modifide is newer than last time 
        let path_to_last_grab_time = addr_to_path("time");
        let modifided:bool = if path_to_last_grab_time.exists() {
            let last_time: chrono::DateTime<Utc> = from_reader(File::open(path_to_last_grab_time).expect("file should be able to open if exists")).unwrap();
            if &api_line_ref.modified
            > 
            &last_time {
                true
            } else {
                false
            }
        } else{
            true
        };

        //get stop point sequences
        if modifided { update_cached_req(&format!("Line/{}/Route/Sequence/outbound",api_line_ref.id)).unwrap();}
        let api_stop_point_sequences_outbound:ApiStopPointSequences = from_reader(cached_req(&format!("Line/{}/Route/Sequence/outbound",api_line_ref.id))).unwrap();
        let api_api_stop_point_sequences_inbound:ApiStopPointSequences = if !api_stop_point_sequences_outbound.isOutboundOnly {
            if modifided { update_cached_req(&format!("Line/{}/Route/Sequence/inbound",api_line_ref.id)).unwrap();}
            from_reader(cached_req(&format!("Line/{}/Route/Sequence/outbound",api_line_ref.id))).unwrap()
        } else {
            ApiStopPointSequences{
                isOutboundOnly:true,
                stations: vec![],
                stopPointSequences: vec![]
            }
        };


        for api_stop_point in api_stop_point_sequences_outbound.stations {

        }
        



    }
    //println!("Line_list:{:?}",line_list);
    //println!("{:?}",cached_req("Line/Route"));

}

//MARK:API Structs
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)] struct ApiLineRef{
    id:String,
    modified:chrono::DateTime<Utc>,
    modeName:String,
    name:String

}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)] struct ApiStopPointSequences{
    isOutboundOnly: bool,
    stations: Vec<ApiStopPoint>,
    stopPointSequences: Vec<ApiStopPointSequence>
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)] struct ApiStopPointSequence{
    stopPoint: Vec<ApiStopPoint>,
    serviceType: String
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)] struct ApiStopPoint {
    id: String,
    topMostParentId:Option<String>,
    parentId:Option<String>,
    lat:f32,
    lon:f32,
    name:String,
    zone:Option<String>,
    stopLetter:Option<String>
}





//MARK:Output Structs
pub struct TflData<'a>{
    pub stop_point_sequences:HashMap<String, StopPointSequence<'a>>,
    pub stop_points:HashMap<String,StopPoint<'a>>,
    pub lines:HashMap<String,StopPoint<'a>>,
    pub modes:HashMap<String,Mode>,
    pub zones:HashMap<String,Zone>
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
    hereditary_stop_point_sequences:Vec<&'a StopPointSequence<'a>>,//duplaciitivitve deffinitly
}

pub struct Line<'a>{
    mode:&'a Mode,
    name: String,
    stop_point_sequences:Vec<&'a StopPointSequence<'a>>,//duplaciitivitve technically
}

pub struct  Mode{
    name: String
}

pub struct  Zone{
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

fn cached_req(addr: &str) -> File {
    let file_path = addr_to_path(addr);
    if file_path.exists() {
        File::open(file_path).expect("filepath is ok but file cannot be oppened")
    } else {
        //println!("getting:{}","https://api.tfl.gov.uk/".to_owned() + addr);
        update_cached_req(addr).unwrap();
        cached_req(addr)
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
    println!("addr:{}, status:{}, ({})", addr, res.status_text(), res.status());
    //println!("str:{}", res.into_string()?);
    //res.into_reader().read(buffer)?;
    // println!("len:{}",buffer.len());
    // file.write_all(buffer).unwrap();
    io::copy(&mut res.into_reader(), &mut file)?;
    Ok(())
}