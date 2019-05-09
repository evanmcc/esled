//#[macro_use]
extern crate erlang_nif_sys;
extern crate sled;

use erlang_nif_sys::*;
use sled::Db;

use std::{mem, ptr, path::Path, str, slice};
//use std::cmp::min;
use std::sync::atomic::{AtomicIsize, Ordering};

static mut SLEDDB_TYPE: *const ErlNifResourceType = 0 as *const ErlNifResourceType;
static mut DTOR_COUNTER: Option<AtomicIsize> = None;

nif_init!("mynifmod",
          [
              ("open", 1, slice_args!(open)),
              ("put", 3, slice_args!(put)),
              ("get", 2, slice_args!(get)),
              ("sleddb_dtor_count", 0, sleddb_dtor_count)
          ],
          {load: esled_load});

unsafe fn esled_load(env: *mut ErlNifEnv,
                     _priv_data: *mut *mut c_void,
                     _load_info: ERL_NIF_TERM) -> c_int {
    let mut tried: ErlNifResourceFlags = mem::uninitialized();
    DTOR_COUNTER = Some(AtomicIsize::new(0));
    SLEDDB_TYPE = enif_open_resource_type(
        env,
        ptr::null(),
        b"sleddb\0".as_ptr(),
        Some(sleddb_destructor),
        ErlNifResourceFlags::ERL_NIF_RT_CREATE,
        &mut tried);
    SLEDDB_TYPE.is_null() as c_int
}

unsafe extern "C" fn sleddb_destructor(_env: *mut ErlNifEnv, handle: *mut c_void) {
    DTOR_COUNTER.as_mut().unwrap().fetch_add(1, Ordering::SeqCst);
    let db = ptr::read(handle as *mut Db);
    db.flush();
}

fn open(env: *mut ErlNifEnv, args: &[ERL_NIF_TERM]) -> ERL_NIF_TERM {
    let db = match args.len() {
        1 => {
            let path = bin_to_slice(env, args[0]);
            let path = str::from_utf8(path).unwrap();
            match Db::start_default(Path::new(path)) {
                Ok(db) => {
                    db
                }
                Err(_) => { // improve this
                    return unsafe { enif_make_badarg(env) }
                }
            }
        }
        _ => {
            return unsafe { enif_make_badarg(env) }
        }
    };
    unsafe {
        let mem = enif_alloc_resource(SLEDDB_TYPE, mem::size_of::<Db>());
        assert_eq!(mem as usize % mem::align_of::<Db>(), 0);
        ptr::write(mem as *mut Db, db);
        let term = enif_make_resource(env, mem);
        enif_release_resource(mem);
        term
    }
}

fn put(env: *mut ErlNifEnv, args: &[ERL_NIF_TERM]) -> ERL_NIF_TERM {
    let (db, key, value) = match args.len() {
        3 => {
            let d: &Db = mem::unintialized();
            let d = unsafe { enif_get_resource(env, args[0], SLEDDB_TYPE, &d) };
            let k = bin_to_slice(env, args[1]);
            let v = bin_to_slice(env, args[2]);
            (d, k, v)
        }
        _ => {
            return unsafe { enif_make_badarg(env) }
        }
    }
    match db.set(key, value) {
        Ok(_) =>
            return atom ok
}

fn get(env: *mut ErlNifEnv, args: &[ERL_NIF_TERM]) -> ERL_NIF_TERM {
}

unsafe fn sleddb_dtor_count(env: *mut ErlNifEnv, _: c_int, _: *const ERL_NIF_TERM) -> ERL_NIF_TERM {
    let cnt = DTOR_COUNTER.as_mut().unwrap().load(Ordering::SeqCst);
    enif_make_int(env, cnt as i32)
}

fn bin_to_slice<'a>(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> &'a [u8] {
    unsafe {
        let mut bin: ErlNifBinary = mem::uninitialized();
        enif_inspect_binary(env, term, &mut bin);
        slice::from_raw_parts(bin.data, bin.size)
    }
}
