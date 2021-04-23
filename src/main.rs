use hyper::service::{make_service_fn, service_fn};
use hyper::Request;
use hyper::Response;
use hyper::Server;
use hyper::{Body, Result, StatusCode};
use std::fs::File;

fn main() {
    let mut rt = tokio::runtime::Builder::new_current_thread();
    let rt = rt.enable_io().worker_threads(1).build().unwrap();

    rt.block_on(async move {
        let addr = "0.0.0.0:8000".parse().unwrap();

        let make_service =
            make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(response_examples)) });

        let server = Server::bind(&addr).serve(make_service);

        println!("Listening on http://{}", addr);

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    });
}

async fn response_examples(mut req: Request<Body>) -> Result<Response<Body>> {
    let mut d = formdata::body_multipart(&mut req).await.unwrap();
    let f = File::create("./upl-spready.xlsx").unwrap();
    d.read_entry()
        .unwrap()
        .unwrap()
        .next_entry()
        .unwrap()
        .data
        .save()
        .write_to(f)
        .into_result()
        .unwrap();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}

#[derive(Debug)]
pub enum Error {
    NoFirstRow,
    // Calamine(XlsxError),
    SheetNotFound,
    HeaderNotFound {
        index: usize,
    },
    MissingMailColumn {
        mail1: Option<String>,
        mail2: Option<String>,
    },
    MissingBoundary,
}

mod formdata {
    use hyper::header::CONTENT_TYPE;
    use hyper::{Body, Request};

    use std::io::Cursor;

    use crate::Error;
    use multipart::server::Multipart;

    pub async fn body_multipart(
        request: &mut Request<Body>,
    ) -> Result<Multipart<Cursor<Vec<u8>>>, Error> {
        const BOUNDARY: &str = "boundary=";

        let boundary = request.headers().get(CONTENT_TYPE).and_then(|ct| {
            let ct = ct.to_str().ok()?;
            let idx = ct.find(BOUNDARY)?;
            dbg!(&ct[idx + BOUNDARY.len()..].to_string(), &ct, idx);
            Some(ct[idx + BOUNDARY.len()..].to_string())
        });
        dbg!(boundary.is_none());
        if boundary.is_none() {
            return Err(Error::MissingBoundary);
        }

        let body = hyper::body::to_bytes(request.body_mut()).await.unwrap();
        let boundary = boundary.unwrap();
        Ok(Multipart::with_body(Cursor::new(body.to_vec()), boundary))
    }
}
