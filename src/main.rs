#![feature(async_closure)]

use reqwest::{Client, Response};
use serde_json::Value;
use warp::filters::body;
use warp::filters::path::{full, FullPath};
use warp::http::Uri;
use warp::Filter;

#[tokio::main]
async fn main() {
  let target_host = "https://api.tzstats.com";

  let get_route = warp::get().map(move || warp::redirect::temporary(Uri::from_static(target_host)));

  let post_route = warp::post()
    .and(full())
    .and(body::json())
    .and_then(async move |full_route: FullPath, mut body: Value| {
      hijack(&mut body);
      Client::new()
        .post(&format!("{}{}", target_host, full_route.as_str()))
        .header("Host", "api.tzstats.com")
        .json(&body)
        .send()
        .await
        .map_err(|_| warp::reject::not_found())
    })
    .and_then(async move |resp: Response| {
      resp
        .json::<Value>()
        .await
        .map_err(|_| warp::reject::not_found())
    })
    .map(|j| warp::reply::json(&j));

  let route = get_route.or(post_route);

  warp::serve(route).run(([127, 0, 0, 1], 8099)).await;
}

fn hijack(v: &mut Value) {
  match v {
    Value::Array(ref mut values) => {
      for value in values {
        hijack(value);
      }
    }

    Value::Object(ref mut map) => {
      if let Some(v) = map.get("fee").cloned() {
        map.insert("fees".to_owned(), v);
      }

      for (_, value) in map.iter_mut() {
        hijack(value);
      }
    }

    _ => (),
  }
}
