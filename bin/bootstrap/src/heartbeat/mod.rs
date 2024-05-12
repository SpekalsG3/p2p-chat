use std::time::Duration;
use tokio::task::JoinHandle;
use crate::heartbeat::task::heartbeat_task;
use crate::types::AppStateRc;

mod task;

pub async fn check_servers_heartbeat(
    app_state: AppStateRc,
) {
    let mut join = None::<JoinHandle<()>>;
    loop {
        // 10 min
        tokio::time::sleep(Duration::from_secs(60 * 10)).await;

        if let Some(ref join) = join {
            if !join.is_finished() {
                println!("[WARNING] check_servers_heartbeat: still running, sleeping...");
                continue;
            }
        }

        println!("[INFO] check_servers_heartbeat: starting new job");
        join = Some(tokio::spawn(heartbeat_task(app_state.clone())));
    }
}
