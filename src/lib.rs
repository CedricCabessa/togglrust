extern crate base64;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate futures;

use chrono::{DateTime, Utc};
use hyper::rt::{Future, Stream};
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use tokio::runtime::Runtime;

use futures::future;

pub mod humanize;

#[derive(Deserialize)]
struct Response {
    data: Option<Task>,
}
#[derive(Deserialize)]
pub struct Task {
    pub description: String,
    pub start: DateTime<Utc>,
}

fn auth(login: &str, password: &str) -> String {
    let mut loginpass = String::from(login);
    loginpass.push_str(":");
    loginpass.push_str(password);
    let hash = base64::encode(loginpass.as_str());
    let mut str = String::from("Basic ");
    str.push_str(hash.as_str());
    str
}

fn fetch_current_task_future(api_key: &str) -> impl Future<Item = String, Error = ()> {
    let mut request = Request::builder();

    let https = HttpsConnector::new(4).unwrap();
    let client = Client::builder().build::<_, hyper::Body>(https);

    request
        .uri("https://www.toggl.com/api/v8/time_entries/current")
        .header("Authorization", auth(api_key, "api_token"));
    client
        .request(request.body(Body::empty()).unwrap())
        .map_err(|_| ())
        .and_then(|res| {
            let f = if res.status().is_success() {
                future::ok(())
            } else {
                future::err(())
            };
            f.and_then(|_| res.into_body().concat2().map_err(|_| ()))
        })
        .map(|body| {
            ::std::str::from_utf8(&body)
                .expect("remote returns utf-8")
                .to_owned()
        })
}

pub fn get_current_task(api_key: &str) -> Result<Option<Task>, ()> {
    let mut rt = Runtime::new().unwrap();
    rt.block_on(fetch_current_task_future(&api_key))
        .and_then(|data| {
            let resp: Response = serde_json::from_str(data.as_str()).unwrap();
            Ok(resp.data)
        })
}
