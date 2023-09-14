use anyhow::Result;

use hyper::http::StatusCode;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;

use crate::device_detector::DeviceDetector;

async fn serve_request(
    req: Request<Body>,
    device_detector: DeviceDetector,
) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/detect") => {
            // TODO prevent pulling entire body into memory in case of abuse
            let body = hyper::body::to_bytes(req.into_body()).await?;
            let body = String::from_utf8(body.to_vec())?;
            let detection = device_detector.parse(&body, None).await.unwrap();
            let response = serde_json::to_string(&detection.to_value())?;

            Ok(Response::new(Body::from(response)))
        }

        (&Method::GET, "/health") => Ok(Response::new("OK\n".into())),

        _route => {
            let err = "valid routes:\n  POST /detect with a body containing referer\n  GET  /health for heartbeat";
            eprintln!("{}", err);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(err))
        }
    }
    .map_err(|x| x.into())
}

pub async fn server(listen_address: SocketAddr, device_detector: DeviceDetector) {
    // TODO make ip configurable
    eprintln!("Listening on {}", listen_address);

    let make_svc = make_service_fn(|_conn| {
        let device_detector = device_detector.clone();

        let service = service_fn(move |req| {
            let device_detector = device_detector.clone();
            serve_request(req, device_detector)
        });

        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&listen_address).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    };
}
