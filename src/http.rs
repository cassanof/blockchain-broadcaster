use std::{
    net::SocketAddr,
    pin::Pin,
    str::FromStr,
    sync::Arc,
    task::{Context, Poll},
};

use futures::Future;
use hyper::{service::Service, Body, Request, Response, Server};
use tokio::sync::Mutex;

use crate::{
    messages::{Message, NewMessage},
    uor_opt, uor_res,
};

use redis::Commands;

/// Represents a wrapper struct for the HTTP server that runs with the work queue.
/// The server supports only one session at a time. For concurrency reasons.
pub struct HTTP {
    // the host and port for a http server
    host: String,
    port: String,
}

impl HTTP {
    pub fn new(host: String, port: String) -> Self {
        HTTP { host, port }
    }

    pub async fn start(
        self,
        redis: redis::Connection,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = SocketAddr::from_str(&format!("{}:{}", self.host, self.port))?;

        let server = Server::bind(&addr).serve(MakeSvc {
            session: Arc::new(Session::create(redis)),
        });

        println!("Listening on http://{}", addr);

        server.await?;
        Ok(())
    }
}

/// Represents a service for the hyper http server
struct Svc {
    // using a mutex to make sure not two sessions are running a container at the same time.
    // this might change if we want to design a more concurrent system.
    session: Arc<Session>,
}

impl Service<Request<Body>> for Svc {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        fn mk_error(s: String, code: u16) -> Result<Response<Body>, hyper::Error> {
            Ok(Response::builder()
                .status(code)
                .body(Body::from(s))
                .unwrap())
        }
        fn mk_response(s: String) -> Result<Response<Body>, hyper::Error> {
            Ok(Response::builder().body(Body::from(s)).unwrap())
        }

        let cloned_session = self.session.clone();
        Box::pin(async move {
            // routes
            // - GET:
            //   - /<id> -> get all messages since id
            // - POST:
            //   - / -> post a message


            // get the method
            let method = req.method().to_string();

            println!("Requested: {} -> {}", method, req.uri());
            match method.as_str() {
                "POST" => {
                    // post a message
                    let body = uor_res!(hyper::body::to_bytes(req.body_mut()).await, || mk_error(
                        "Failed to read body".to_string(),
                        400
                    ));
                    let message = uor_res!(String::from_utf8(body.to_vec()), || mk_error(
                        "Failed to parse body".to_string(),
                        400
                    ));

                    // return error if message has newlines
                    if message.contains('\n') {
                        return mk_error("Error: Message contains newlines".to_string(), 400);
                    }

                    let message = match NewMessage::from_str(&message) {
                        Ok(m) => m.to_string(),
                        Err(e) => return mk_error(format!("Error: {}", e), 400),
                    };

                    {
                        let mut redis = cloned_session.db.lock().await;

                        uor_res!(
                            redis::pipe()
                                .atomic()
                                .rpush("messages", message)
                                .ignore()
                                .query::<()>(&mut *redis),
                            || mk_error("Failed to push message to redis".to_string(), 500)
                        );
                    }

                    // sleep to rate limit
                    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

                    mk_response(String::new())
                }
                "GET" => {
                    // get id from path
                    let path = req.uri().path().to_string();
                    let id = uor_opt!(path.split('/').last(), || mk_error(
                        "Failed to get id from path".to_string(),
                        400
                    ));

                    if id.is_empty() {
                        return mk_error("Error: Didn't provide an id".to_string(), 500);
                    }
                    let id = uor_res!(id.parse::<isize>(), || mk_error(
                        "Error: Failed to parse id".to_string(),
                        400
                    ));

                    if id < 0 {
                        return mk_error("Error: Id must be positive".to_string(), 400);
                    }

                    // get all messages since id
                    let res: Vec<String> = {
                        let mut redis = cloned_session.db.lock().await;
                        uor_res!(redis.lrange("messages", id, id + 200), || mk_error(
                            "Failed to get messages from redis".to_string(),
                            500
                        ))
                    };

                    let mut buf = String::new();
                    for (i, msg) in res.iter().enumerate() {
                        let line = format!("{}:{}\n", (i as isize) + id, msg);
                        buf.push_str(&line);
                    }

                    mk_response(buf)
                }
                _ => mk_error(format!("ERROR: Invalid method {}", method), 500),
            }
        })
    }
}

/// Represents a maker for a service for the hyper http server

struct MakeSvc {
    session: Arc<Session>,
}

impl<T> Service<T> for MakeSvc {
    type Response = Svc;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let session = self.session.clone();
        let fut = async move { Ok(Svc { session }) };
        Box::pin(fut)
    }
}

/// Represents the session being manipulated by the http server
struct Session {
    pub db: Mutex<redis::Connection>,
}

impl Session {
    pub fn create(con: redis::Connection) -> Self {
        Session {
            db: Mutex::new(con),
        }
    }
}
