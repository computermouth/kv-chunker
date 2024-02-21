
use fastly::http::{Method, StatusCode};
use fastly::{Error, Request, Response};
use fastly::kv_store::{KVStore, KVStoreError};
use fastly::kv_store::handle::PendingObjectStoreLookupHandle;
use fastly::Body;

// pub const PASSWORD: &str = "yourpassword";
// mod pw;

const STORE_NAME: &str = "chunkstore";

fn put(name: &str, pc: usize, pcs: usize, data: Vec<u8>) -> Result<Response, Error> {

    let store = KVStore::open(STORE_NAME)?;
    if store.is_none() {
        return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        .with_body_text_plain("couldn't open store\n"));
    }
    let mut store = store.unwrap();

    let mut pcs_key_name = name.to_string();
    pcs_key_name.push_str("_pcs");
    store.insert(pcs_key_name.as_str(), pcs.to_string())?;

    let mut pc_key_name = name.to_string();
    pc_key_name.push_str("_");
    pc_key_name.push_str(&pc.to_string());
    store.insert(&pc_key_name, data)?;

    Ok(Response::from_status(StatusCode::OK))
}

fn get(name: &str) -> Result<Response, Error> {

    let store = KVStore::open(STORE_NAME)?;
    if store.is_none() {
        return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        .with_body_text_plain("couldn't open store\n"));
    }
    let store = store.unwrap();

    let mut pcs_key_name = name.to_string();
    pcs_key_name.push_str("_pcs");

    match store.lookup(&pcs_key_name) {
        Ok(o) => {
            if o.is_none() {
                return Ok(Response::from_status(StatusCode::NOT_FOUND)
                .with_body_text_plain("pcs not found\n"));
            }
            let size = o.unwrap().into_string().parse::<usize>();
            if size.is_err(){
                return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                .with_body_text_plain("couldn't parse size\n"));
            }
            let size = size.unwrap();

            let mut resbody = Body::new();

            let handles: Vec<PendingObjectStoreLookupHandle> = (0..size).into_iter().map(|i|{
                let mut pc_key = name.to_string();
                pc_key.push_str("_");
                pc_key.push_str(&i.to_string());
                store.lookup_async(&pc_key).unwrap().unwrap()
            }).collect();

            for (i, v) in handles.into_iter().enumerate() {
                let lookup = store.pending_lookup_wait(v);
                if lookup.is_err(){
                    return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                    .with_body_text_plain("couldn't parse size\n"));
                }
                let lookup = lookup.unwrap();
                if lookup.is_none() {
                    let s = format!("pc {} not found\n", i);
                    return Ok(Response::from_status(StatusCode::NOT_FOUND)
                    .with_body_text_plain(&s));
                }
                let lookup = lookup.unwrap();
                resbody.append(lookup);
            }

            let mut res = Response::from_status(StatusCode::OK);
            res.set_header("version", "1");
            res.set_body(resbody);
            res.set_content_type(fastly::mime::APPLICATION_OCTET_STREAM);

            Ok(res)
        },
        Err(_) => { 
            Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            .with_body_text_plain("couldn't open store\n"))
        }
    }
}

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {

    // Filter request methods...
    match req.get_method() {
        // &Method::PUT => {
        //     if !req.contains_header("pw"){
        //         return Ok(Response::from_status(StatusCode::BAD_REQUEST)
        //         .with_body_text_plain("no pw\n"))
        //     }
        //     if !req.get_header_str("pw").unwrap().eq(pw::PASSWORD){
        //         return Ok(Response::from_status(StatusCode::UNAUTHORIZED)
        //         .with_body_text_plain("bad pw\n"))
        //     }
        //     if !req.contains_header("sz"){
        //         return Ok(Response::from_status(StatusCode::BAD_REQUEST)
        //         .with_body_text_plain("no sz\n"))
        //     }
        //     if !req.contains_header("pc"){
        //         return Ok(Response::from_status(StatusCode::BAD_REQUEST)
        //         .with_body_text_plain("no pc\n"))
        //     }
        //     if !req.contains_header("pcs"){
        //         return Ok(Response::from_status(StatusCode::BAD_REQUEST)
        //         .with_body_text_plain("no pcs\n"))
        //     }
        //     if !req.contains_header("name"){
        //         return Ok(Response::from_status(StatusCode::BAD_REQUEST)
        //         .with_body_text_plain("no name\n"))
        //     }
        //     let name = req.get_header_str("name").unwrap().to_owned();
        //     let pc: usize = req.get_header_str("pc").unwrap().parse().unwrap();
        //     let pcs: usize = req.get_header_str("pcs").unwrap().parse().unwrap();
        //     let size: usize = req.get_header_str("sz").unwrap().parse().unwrap();
        //     let data = req.into_body_bytes();
        //     if size != data.len(){
        //         println!("sz: {} data.len: {}", size, data.len());
        //         return Ok(Response::from_status(StatusCode::BAD_REQUEST)
        //         .with_body_text_plain("bad sz\n"))
        //     }
        //     put(&name, pc, pcs, data)
        // }
        &Method::GET => {
            let name = &req.get_path()[1..];
            get(name)
        }
        _ => {
            Ok(Response::from_status(StatusCode::METHOD_NOT_ALLOWED)
                .with_body_text_plain("This method is not allowed\n"))
        },
    }

}
