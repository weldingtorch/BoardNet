// Task database interface

use std::sync::atomic::{AtomicU32, Ordering};

use mysql::prelude::*;
use mysql::*;

static NEXT_TASK_ID: AtomicU32 = AtomicU32::new(0);


fn load_task_id() {
    // load from save.dat
}

pub fn next_task_id() -> u32 {
    NEXT_TASK_ID.fetch_add(1, Ordering::AcqRel)
}

pub fn db_test() {
    // Opts::from_url("mysql://master:devpass@localhost").unwrap();
    let opts = OptsBuilder::new()
        .user(Some("master"))
        .pass(Some("devpass"));
    let conn = Conn::new(opts).unwrap();
    
}