// These require the `serde` dependency.
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct NewTx {
    tmp_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PostTx {
    tx_id: String,
    name: String,
    relative_path: String,
}

pub async fn tx(tmp_id: String) -> Result<PostTx, reqwest::Error> {
    let new_tx = NewTx { tmp_id: tmp_id };
    let tx_done: PostTx = reqwest::Client::new()
        .post("http://localhost:3000/tx/new")
        .json(&new_tx)
        .send()
        .await?
        .json()
        .await?;

    Ok(tx_done)
}
