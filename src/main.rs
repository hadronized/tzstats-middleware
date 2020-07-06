#![feature(async_closure)]

use reqwest::{Client, Response};
use serde_json::Value;
use std::ops::Deref as _;
use std::sync::Arc;
use structopt::StructOpt;
use warp::filters::body;
use warp::filters::path::{full, FullPath};
use warp::Filter;

#[derive(Debug, StructOpt)]
#[structopt(
  name = "tzstats-middleware",
  author = "Dimitri Sabadie <dimitri.sabadie@ledger.com>",
  about = "A tzstats mangler applying on-the-fly rules to JSON HTTP requests."
)]
struct CLIOpts {
  /// Target API host to forward requests to.
  #[structopt(short, long, default_value = "api.tzstats.com")]
  target_host: String,
}

#[tokio::main]
async fn main() {
  let CLIOpts { target_host } = CLIOpts::from_args();

  // warp the target_host parameter in an Arc so that we can share it for all connections
  let target_host = Arc::new(target_host);
  let target_host_filter = warp::any().map(move || target_host.clone());

  let get_route_with_body = warp::get()
    .and(full())
    .and(body::json())
    .and(target_host_filter.clone())
    .and_then(
      async move |full_route: FullPath, body: Value, target_host: Arc<String>| {
        Client::new()
          .get(&format!("https://{}{}", *target_host, full_route.as_str()))
          .header("Host", target_host.deref())
          .json(&body)
          .send()
          .await
          .map_err(|_| warp::reject::not_found())
      },
    )
    .and_then(async move |resp: Response| {
      resp
        .json::<Value>()
        .await
        .map_err(|_| warp::reject::not_found())
    })
    .map(|mut j| {
      hijack(&mut j);
      warp::reply::json(&j)
    });

  let get_route = warp::get()
    .and(full())
    .and(target_host_filter.clone())
    .and_then(
      async move |full_route: FullPath, target_host: Arc<String>| {
        Client::new()
          .get(&format!("https://{}{}", target_host, full_route.as_str()))
          .header("Host", target_host.deref())
          .send()
          .await
          .map_err(|_| warp::reject::not_found())
      },
    )
    .and_then(async move |resp: Response| {
      resp
        .json::<Value>()
        .await
        .map_err(|_| warp::reject::not_found())
    })
    .map(|mut j| {
      hijack(&mut j);
      warp::reply::json(&j)
    });

  let post_route = warp::post()
    .and(full())
    .and(body::json())
    .and(target_host_filter.clone())
    .and_then(
      async move |full_route: FullPath, mut body: Value, target_host: Arc<String>| {
        hijack(&mut body);
        Client::new()
          .post(&format!("https://{}{}", target_host, full_route.as_str()))
          .header("Host", target_host.deref())
          .json(&body)
          .send()
          .await
          .map_err(|_| warp::reject::not_found())
      },
    )
    .and_then(async move |resp: Response| {
      resp
        .json::<Value>()
        .await
        .map_err(|_| warp::reject::not_found())
    })
    .map(|mut j| {
      hijack(&mut j);
      warp::reply::json(&j)
    });

  let route = get_route_with_body.or(get_route.or(post_route));

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
