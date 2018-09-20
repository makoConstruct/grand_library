
// use std::slice::Iter as SliceIter;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate sled;

extern crate byteorder; //this is for converting u64s to Vec<u8>s in a platform-independent way


type Arce<T> = Arc<RwLock<T>>;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
struct VesselID(u64);

impl VesselID {
    fn write_id(&self, wtr:&mut Vec<u8>){
        use byteorder::{LittleEndian, WriteBytesExt};
        wtr.write_u64::<LittleEndian>(self.0).unwrap();
    }
    fn db_id(&self)-> Vec<u8> {
        let mut id = "vessel/".as_bytes().into();
        self.write_id(&mut id);
        id
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Vessel {
    children: Vec<VesselID>,
    body: String,
    name: String,
    id: VesselID,
    parent: VesselID,
}

fn json_to_vessel(from: &str)-> Result<Vessel, serde_json::Error> {
    serde_json::from_str(from)
}

fn vessel_to_json(v:&Vessel)-> String {
    serde_json::to_string(v).unwrap() //literally cannot fail =__=
}


struct Database {
    db: sled::Tree,
}


impl Database {
    fn new()-> Database {
        Database{ db: sled::Tree::start(sled::ConfigBuilder::new().path(std::path::Path::new("sled.db")).build()).unwrap() }
    }
    
    fn stow_vessel(&self, v:&Vessel){
        
        self.db.set(v.id.db_id(), vessel_to_json(v).into()).unwrap();
    }
    fn stow_string_string(&self, k:String, v:String){
        self.db.set(k.into(), v.into()).unwrap();
    }
    fn get_string_string(&self, k:String)-> Option<String> {
        self.db.get(k.as_bytes()).unwrap().map(|bl|
            unsafe{ String::from_utf8_unchecked(bl) } //never put any non-utf8 in the self.db!
        )
    }
    fn get_vessel(&self, k:VesselID)-> Option<Vessel> {
        self.db.get(k.db_id().as_slice()).unwrap().map(|bl|{
            let s = unsafe{ String::from_utf8_unchecked(bl) }; //never put any non-utf8 in the self.db!
            json_to_vessel(s.as_str()).unwrap()
        })
    }
}

fn main(){
    
    let db = Database::new();
    let cache: RwLock<HashMap<VesselID, Arce<Vessel>>> = RwLock::new(HashMap::new());
    
    let write_back_cache = ||{ //this will be done periodically, so that if the server crashes, not much data will be lost
        //write back to the db from the cache
        for (_id, v) in cache.read().unwrap().iter() {
            db.stow_vessel(&*v.read().unwrap());
        }
    };
    

    let get_vessel = |id:VesselID|-> Option<Arce<Vessel>> {
        {
            let rc = cache.read().unwrap();
            if let Some(ar) = rc.get(&id) {
                return Some(ar.clone());
            }
        }
        let mut wc = cache.write().unwrap();
        
        if let Some(v) = db.get_vessel(id) {
            let v:Arce<Vessel> = Arce::new(RwLock::new(v));
            wc.insert(id, v.clone());
            Some(v)
        }else{
            None
        }
    };
    
    
}