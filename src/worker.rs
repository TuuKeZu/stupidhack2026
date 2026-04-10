use crate::alcohol::AlcoholStatus;
use crate::packets::Response;
use crate::state::{SharedState, SocketType};
use tokio_cron_scheduler::Job;
use warp::filters::ws::Message;

pub async fn worker(state: SharedState) {
    let sched = tokio_cron_scheduler::JobScheduler::new()
        .await
        .expect("Failed to create scheduler");

    let state_clone = state;

    sched
        .add(
            Job::new_async("*/5 * * * * *", move |_uuid, _l| {
                let state = state_clone;
                Box::pin(async move {
                    state
                        .alcohol_update(|alcohol| match alcohol.status {
                            AlcoholStatus::Uninitialized => {
                                eprintln!(
                                    "[WORKER] AlcoholState is Uninitialized, sending status update"
                                );

                                let _ = alcohol.tick();

                                let response = Response::Status {
                                    current: alcohol.current,
                                    target: alcohol.target,
                                    update: alcohol.status.into(),
                                };

                                if let Ok(json) = serde_json::to_string(&response) {
                                    let json_client = json.clone();
                                    let json_pump = json;

                                    tokio::spawn(async move {
                                        let _ = state
                                            .send_message(
                                                SocketType::Client,
                                                Message::text(json_client),
                                            )
                                            .await;
                                    });
                                    tokio::spawn(async move {
                                        let _ = state
                                            .send_message(
                                                SocketType::Pump,
                                                Message::text(json_pump),
                                            )
                                            .await;
                                    });
                                }
                            }
                            AlcoholStatus::Calibrated => {
                                eprintln!("[WORKER] {:?}", alcohol);

                                let response = Response::Status {
                                    current: alcohol.current,
                                    target: alcohol.target,
                                    update: alcohol.status.into(),
                                };

                                match alcohol.tick() {
                                    None => (),
                                    Option::Some(amount) => {
                                        let response = Response::PumpUpdate { amount };
                                        tokio::spawn(async move {
                                            if let Ok(json) = serde_json::to_string(&response) {
                                                let _ = state
                                                    .send_message(
                                                        SocketType::Pump,
                                                        Message::text(json),
                                                    )
                                                    .await;
                                            }
                                        });
                                    }
                                }

                                if let Ok(json) = serde_json::to_string(&response) {
                                    let json_client = json.clone();
                                    let json_pump = json;

                                    tokio::spawn(async move {
                                        let _ = state
                                            .send_message(
                                                SocketType::Client,
                                                Message::text(json_client),
                                            )
                                            .await;
                                    });
                                    tokio::spawn(async move {
                                        let _ = state
                                            .send_message(
                                                SocketType::Pump,
                                                Message::text(json_pump),
                                            )
                                            .await;
                                    });
                                }
                            }
                        })
                        .await;
                })
            })
            .expect("Failed to add job"),
        )
        .await
        .expect("Failed to add job to scheduler");

    sched.start().await.expect("Failed to start scheduler");
}
