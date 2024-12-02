// THIS FILE IS A WIP.

use std::{
    io::{Read, Write},
    os::fd::FromRawFd,
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use starknet::providers::{
    jsonrpc::{JsonRpcMethod, JsonRpcResponse, JsonRpcTransport},
    ProviderRequestData,
};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};
use url::Url;

const FD_REQUEST: i32 = 3;
const FD_RESPONSE: i32 = 4;

const POLL_INTERVAL: Duration = Duration::from_millis(25);

/// A special transport designed to be used in WASI environment.
///
/// Under the hood, it uses specific file descriptors designated as request/response queues. The
/// WASI host executes the actual requests.
pub struct WebTransport {
    sender: UnboundedSender<HostRequest>,
    receiver: Arc<Mutex<UnboundedReceiver<String>>>,
    url: Url,
    headers: Vec<(String, String)>,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest<T> {
    id: u64,
    jsonrpc: &'static str,
    method: JsonRpcMethod,
    params: T,
}

#[derive(Debug, Serialize)]
struct HostRequest {
    url: String,
    body: String,
}

impl WebTransport {
    pub fn new(url: impl Into<Url>) -> Self {
        // TODO: refactor this into a separate type for host-guest communication.
        // Response retriever
        let (response_sender, response_receiver) = tokio::sync::mpsc::unbounded_channel::<String>();
        tokio::spawn(async move {
            // TODO: allows strings larger than buffer
            let mut response_queue = unsafe { std::fs::File::from_raw_fd(FD_RESPONSE) };
            let mut buffer = [0u8; 100_000];

            loop {
                let mut bytes_read;
                loop {
                    bytes_read = response_queue.read(&mut buffer).unwrap();
                    if bytes_read != 0 {
                        break;
                    }
                    tokio::time::sleep(POLL_INTERVAL).await;
                }

                response_sender
                    .send(unsafe { String::from_utf8_unchecked(buffer[0..bytes_read].to_vec()) })
                    .unwrap();
            }
        });

        // TODO: refactor this into a separate type for host-guest communication.
        // Request dumper
        let (request_sender, mut request_receiver) =
            tokio::sync::mpsc::unbounded_channel::<HostRequest>();
        tokio::spawn(async move {
            let mut request_queue = unsafe { std::fs::File::from_raw_fd(FD_REQUEST) };

            loop {
                let http_request = request_receiver.recv().await.unwrap();
                request_queue
                    .write_all(serde_json::to_string(&http_request).unwrap().as_bytes())
                    .unwrap();
            }
        });

        Self {
            sender: request_sender,
            url: url.into(),
            receiver: Arc::new(Mutex::new(response_receiver)),
            headers: vec![],
        }
    }

    pub fn add_header(&mut self, name: String, value: String) {
        self.headers.push((name, value))
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl JsonRpcTransport for WebTransport {
    // TODO: implement proper error handling
    type Error = std::convert::Infallible;

    async fn send_request<P, R>(
        &self,
        method: JsonRpcMethod,
        params: P,
    ) -> Result<JsonRpcResponse<R>, Self::Error>
    where
        P: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let request_body = JsonRpcRequest {
            id: 1,
            jsonrpc: "2.0",
            method,
            params,
        };
        let request_body = serde_json::to_string(&request_body).unwrap();

        // TODO: trace log

        // TODO: use `headers`
        self.sender
            .send(HostRequest {
                url: self.url.to_string(),
                body: request_body,
            })
            .unwrap();

        let response_body = {
            let mut guard = self.receiver.lock().await;
            guard.recv().await.unwrap()
        };

        // TODO: trace log

        let parsed_response = serde_json::from_str(&response_body).unwrap();

        Ok(parsed_response)
    }

    async fn send_requests<R>(
        &self,
        _requests: R,
    ) -> Result<Vec<JsonRpcResponse<serde_json::Value>>, Self::Error>
    where
        R: AsRef<[ProviderRequestData]> + Send + Sync,
    {
        unimplemented!("currently Starkli does not make use of batch requests")
    }
}
