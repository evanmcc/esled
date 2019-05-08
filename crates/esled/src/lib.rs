#[macro_use]
extern crate erlang_nif_sys;
extern crate sled;

use erlang_nif_sys::*;
use sled::Db;


static mut SLEDDB_TYPE: *const ErlNifResourceType = 0 as *const ErlNifResourceType;
static mut DTOR_COUNTER: Option<AtomicIsize> = None;

nif_init!("mynifmod",
          [
              ("times2", 1, slice_args!(times2)),
              ("test_enif_make_pid", 0, test_enif_make_pid),
              ("rustmap", 0, rustmap),
              ("rustmap_dtor_count", 0, rustmap_dtor_count),
              ("to_str", 1, slice_args!(to_str)),
              ("hash", 1, slice_args!(hash)),
              ("make_map", 0, slice_args!(make_map)),
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

fn times2(env: *mut ErlNifEnv, args: &[ERL_NIF_TERM]) -> ERL_NIF_TERM {
    unsafe {
        let mut result: i32 = mem::uninitialized();
        if 1==args.len() && 0!=enif_get_int(env, args[0], &mut result) {
            enif_make_int(env, 2*result)
        }
        else {
            enif_make_badarg(env)
        }
    }
}

fn test_enif_make_pid(env: *mut ErlNifEnv, _: c_int, _: *const ERL_NIF_TERM) -> ERL_NIF_TERM {
    let mut pid: ErlNifPid = unsafe { mem::uninitialized() };
    unsafe { enif_self(env, &mut pid) };
    unsafe { enif_make_pid(env, &pid) }
}


