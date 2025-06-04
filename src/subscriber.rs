use std::error::Error;
use std::time::Duration;

use futures_util::StreamExt;
use tokio::{select, time::sleep};
use tokio_tungstenite::{connect_async, tungstenite::http::Uri};

use crate::payload::FlashblocksPayloadV1;

pub struct WebsocketSubscriber {
    uri: Uri,
}

impl WebsocketSubscriber {
    pub fn new(uri: Uri) -> Self {
        Self { uri }
    }

    pub async fn gather_flashblocks(
        &self,
        duration: Duration,
    ) -> Result<Vec<FlashblocksPayloadV1>, Box<dyn Error>> {
        println!("Gathering flashblocks for {} seconds", duration.as_secs());

        let mut flashblocks = Vec::new();
        let (ws_stream, _) = connect_async(&self.uri).await.unwrap();
        let (_, mut read) = ws_stream.split();

        let sleep = sleep(duration);
        tokio::pin!(sleep);

        loop {
            select! {
                () = &mut sleep => {
                    break;
                }

                Some(message) = read.next() => {
                    match message {
                        Ok(msg) => {
                            let text = msg.to_text()?;
                            let flashblock = serde_json::from_str::<FlashblocksPayloadV1>(&text).unwrap();
                            flashblocks.push(flashblock);
                        }
                        Err(e) => {
                            return Err(Box::new(e));
                        }
                    }
                }
            }
        }

        Ok(flashblocks)
    }
}
