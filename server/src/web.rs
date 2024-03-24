// Web service (user interface and routing)

use crate::{ManagerEvent, Task, Worker};
use crate::db::next_task_id;

use std::collections::HashMap;
use std::path::Path;
use std::sync::{mpsc::Sender, OnceLock, Arc, RwLock};

use cluster::filelib::{Attachment, AttachmentType};
use rocket::fs::TempFile;
use rocket::form::{Form, FromForm};
use rocket::response::Redirect;
use rocket_dyn_templates::{Template, context};


static MNG_TX: OnceLock<Sender<ManagerEvent>> = OnceLock::new();


#[get("/")]
fn index() -> Template {
    // Network stats, form to send task, form to get task info
    
    Template::render("index", context! { field: "value" })
}

#[get("/new_task")]
fn new_task_get() -> Template {
    Template::render("new_task", context! {})
}

#[derive(FromForm, Debug)]
struct WebTask<'a> {
    #[field(validate = len(1..))]
    shell: String,
    //attachment_type: AttachmentType,
    upload_file: Option<TempFile<'a>>,
    retain_attachment: bool
}

#[post("/new_task", data = "<form>")]
async fn new_task_post(form: Form<WebTask<'_>>) -> Redirect {
    // Send new task to manager (<shell script>, [file required for script])
    // Redirect to /[task_id]
    // NOTE: save shell in cookies: https://rocket.rs/v0.5-rc/guide/requests/#private-cookies
    
    dbg!(&form);

    let WebTask {
        shell,
        upload_file,
        retain_attachment
    } = form.into_inner();

    let task_id = next_task_id();    
    let mut attachment: Option<Attachment> = None;

    if let Some(mut file) = upload_file {      
        if let Some(name) = file.name() {
            let path = Path::new("./tasks").join(task_id.to_string());
            std::fs::create_dir_all(&path).unwrap();

            file.persist_to(path.join(name)).await.unwrap();

            attachment = Some(Attachment{
                size: file.len(),
                attachment_type: AttachmentType::Raw,
                retain_attachment,
                filename: file.name().unwrap().to_owned()
            });
        }
    }

    let new_task = Task {
        id: task_id,
        shell,
        attachment
    };
    
    println!("[web] Recieved task: {:?}", new_task);

    MNG_TX.get().unwrap().send(ManagerEvent::NewTask(new_task)).unwrap();
    
    Redirect::to(uri!(task_status(task_id)))
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

pub fn start_web_server(mng_tx: Sender<ManagerEvent>, workers: Arc<RwLock<HashMap<u8, Worker>>>) -> Result<(), rocket::Error> {
    MNG_TX.set(mng_tx).unwrap();
    main()
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    rocket::build()
        .mount("/", routes![index, new_task_get, new_task_post, task_status, cancel_task])
        .attach(Template::fairing())
        //.manage(workers)
        .launch()
        .await?;

    Ok(())
}