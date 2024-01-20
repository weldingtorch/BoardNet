use std::{sync::{Arc, RwLock}, collections::HashMap};

use rocket::response::Redirect;


#[get("/")]
fn index() -> &'static str {
    // Network stats, form to send task, form to get task info
    "Hello, world!"
}

#[post("/")]
fn add_task() -> Redirect {
    // NOTE: Upload file: https://rocket.rs/v0.5-rc/guide/requests/#temporary-files
    // NOTE: forms: https://rocket.rs/v0.5-rc/guide/requests/#forms
    // Send new task (<shell script> [any file required for script])
    // Redirect to /[task_id]
    // NOTE: save shell in cookies: https://rocket.rs/v0.5-rc/guide/requests/#private-cookies

    let id = 123;
    
    Redirect::to(uri!(task_status(id)))
}

#[get("/<task_id>")]
fn task_status(task_id: u32) -> String {
    // Task status: waiting in queue/started/finished/canceled
    // Task input <download script> [download file]
    // Task out first 100 lines <download [task].out>
    // Task err first 100 lines <download [task].err>
    // Cancel? (only for "waiting in queue" I guess)

    format!("Status of task id: {}", task_id)
}

#[get("/cancel?<task_id>")]
fn cancel_task(task_id: u32) -> Redirect {
    // try to cancel
    // redirect to "/task"

    Redirect::to(uri!(task_status(task_id)))
}

#[rocket::main]
pub async fn start_web_server(workers: Arc<RwLock<HashMap<u8, crate::Worker>>>) -> Result<(), rocket::Error> {
    rocket::build()
        .mount("/", routes![index, add_task, task_status, cancel_task])
        .manage(workers)
        .launch()
        .await?;

    Ok(())
}