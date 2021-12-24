use std::collections::HashMap;
use std::convert::Infallible;
use std::fs;
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{crate_description, crate_name, crate_version, App, Arg};
use hyper::{body::HttpBody, Body, Request, Response, Server};
use routerify::prelude::*;
use routerify::{Router, RouterService};

struct Config {
    output_dir: PathBuf,
}

fn parse_query_params(query: Option<&str>) -> HashMap<String, String> {
    let mut params = HashMap::default();

    if let Some(query) = query {
        eprintln!("Parsing query {:?}", query);
        for part in query.split("&") {
            let parts: Vec<&str> = part.split("=").collect();
            if parts.len() == 2 {
                params.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
    }

    params
}

async fn post_handler(mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let config = req.data::<Config>().unwrap();
    let params = parse_query_params(req.uri().query());
    if let Some(name) = params.get("name") {
        eprintln!("Writing file {:?}", name);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .read(false)
            .create_new(true)
            .open(std::path::Path::new(&config.output_dir).join(name))
            .unwrap();
        let body_mut = req.body_mut();
        while let Some(body) = body_mut.data().await {
            file.write(&body.unwrap()).unwrap();
        }
    }
    Ok(Response::new(Body::from("OK")))
}

#[tokio::main]
async fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("listen")
                .short("l")
                .value_name("LISTEN_ADDRESS")
                .help("The address to listen on")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .value_name("OUTPUT_DIRECTORY")
                .help("Directory to output to")
                .takes_value(true),
        )
        .get_matches();

    let parent_dir = if let Some(output_dir) = matches.value_of("output") {
        std::path::Path::new(output_dir).to_path_buf()
    } else {
        std::env::current_dir().unwrap()
    };

    if !parent_dir.exists() {
        std::fs::create_dir_all(&parent_dir).expect("Unable to create output directory");
    }

    let router = Router::builder()
        .data(Config {
            output_dir: parent_dir,
        })
        .post("/", post_handler)
        .build()
        .expect("Unable to build router");

    let service = RouterService::new(router).unwrap();

    let addr = SocketAddr::from_str(matches.value_of("listen").unwrap()).unwrap();

    let server = Server::bind(&addr).serve(service);

    eprintln!("Listening on {}", addr);
    if let Err(err) = server.await {
        eprintln!("Error: {}", err);
    }
}
