use async_std;
use core::convert::TryInto;
use ergolib;
use tide::{Body, Request, Response};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Counter {
    count: usize,
}

enum Error {
    Yaml(serde_yaml::Error),
    DivisionByZero,
    DontLikeIt,
}

impl TryInto<http_types::StatusCode> for &Error {
    type Error = <http_types::StatusCode as std::convert::TryFrom<u16>>::Error;
    fn try_into(self) -> std::result::Result<http_types::StatusCode, Self::Error> {
        match self {
            Error::Yaml(_) | Error::DivisionByZero => 422.try_into(),
            Error::DontLikeIt => 500.try_into(),
        }
    }
}

impl Into<Body> for Error {
    fn into(self) -> Body {
        match self {
            Error::Yaml(y) => format!("YAML error: {}", y).into(),
            Error::DivisionByZero => "count can't be zero".into(),
            Error::DontLikeIt => "The server doesn't like this number".into(),
        }
    }
}

impl From<Error> for Response {
    fn from(e: Error) -> Self {
        let mut r = Response::new(&e);
        r.set_body(e);
        r
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        Error::Yaml(e)
    }
}

impl From<ergolib::Error> for Error {
    fn from(e: ergolib::Error) -> Self {
        match e {
            ergolib::Error::DivisionByZero => Error::DivisionByZero,
            ergolib::Error::DontLikeIt => Error::DontLikeIt,
        }
    }
}

type Result = std::result::Result<Counter, Error>;

fn handle42(mut counter: Counter) -> Result {
    counter.count -= 1;
    counter.count = ergolib::nth42(counter.count)?;
    Ok(counter)
}

fn handle1337(mut counter: Counter) -> Result {
    counter.count -= 1;
    counter.count = ergolib::nth1337(counter.count)?;
    Ok(counter)
}

fn from_yaml(body: &str) -> std::result::Result<Counter, Error> {
    Ok(serde_yaml::from_str(body)?)
}

#[async_std::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut app = tide::new();
    app.at("/42")
        .get(|mut req: Request<()>| async move {
            let body = req.body_string().await?;
            // This isn't converted into a proper error despite being
            // implemented for `Error` since we can't return a custom error type
            let mut counter: Counter = serde_yaml::from_str(&body)?;
            println!("[42] count is {}", counter.count);

            match handle42(counter) {
                Ok(c) => counter = c,
                Err(e) => return Ok(e.into()),
            }
            let mut res = Response::new(200);
            res.set_body(Body::from_json(&counter)?);
            Ok(res)
        })
        .at("/1337")
        .get(|mut req: Request<()>| async move {
            let body = req.body_string().await?;
            // This is a "workaround"
            let mut counter: Counter = match from_yaml(&body) {
                Ok(c) => c,
                Err(e) => return Ok(e.into()),
            };
            println!("[1337] count is {}", counter.count);
            match handle1337(counter) {
                Ok(c) => counter = c,
                Err(e) => return Ok(e.into()),
            }
            let mut res = Response::new(200);
            res.set_body(Body::from_json(&counter)?);
            Ok(res)
        });
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
