use futures::future;
use hyper::rt::{Future, Stream};
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;

fn auth(login: &str, password: &str) -> String {
    let mut loginpass = String::from(login);
    loginpass.push_str(":");
    loginpass.push_str(password);
    let hash = base64::encode(loginpass.as_str());
    let mut str = String::from("Basic ");
    str.push_str(hash.as_str());
    str
}

pub fn fetch_api_future(api_key: &str, endpoint: &str) -> impl Future<Item = String, Error = ()> {
    let mut request = Request::builder();

    let https = HttpsConnector::new(4).unwrap();
    let client = Client::builder().build::<_, hyper::Body>(https);

    request
        .uri(format!("https://www.toggl.com/api/v8/{}", endpoint))
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