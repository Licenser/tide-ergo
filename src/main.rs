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
    Tide(tide::Error),
    DivisionByZero,
    DontLikeIt,
}

impl TryInto<http_types::StatusCode> for &Error {
    type Error = <http_types::StatusCode as std::convert::TryFrom<u16>>::Error;
    fn try_into(self) -> std::result::Result<http_types::StatusCode, Self::Error> {
        match self {
            Error::Yaml(_) | Error::DivisionByZero => 422.try_into(),
            Error::DontLikeIt => 500.try_into(),
            Error::Tide(t) => Ok(t.status()),
        }
    }
}

impl Into<Body> for Error {
    fn into(self) -> Body {
        match self {
            Error::Yaml(y) => format!("YAML error: {}", y).into(),
            Error::Tide(t) => format!("TIDE error: {}", t).into(),
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
impl From<tide::Error> for Error {
    fn from(e: tide::Error) -> Self {
        Error::Tide(e)
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

type Result<T> = std::result::Result<T, Error>;

fn handle42(mut counter: Counter) -> Result<Counter> {
    counter.count -= 1;
    counter.count = ergolib::nth42(counter.count)?;
    Ok(counter)
}

fn handle1337(mut counter: Counter) -> Result<Counter> {
    counter.count -= 1;
    counter.count = ergolib::nth1337(counter.count)?;
    Ok(counter)
}

async fn get42(mut req: Request<()>) -> Result<Response> {
    let body = req.body_string().await?;
    let mut counter: Counter = serde_yaml::from_str(&body)?;
    println!("[42] count is {}", counter.count);

    counter = handle42(counter)?;
    let mut res = Response::new(200);
    res.set_body(Body::from_json(&counter)?);
    Ok(res)
}

async fn get1337(mut req: Request<()>) -> Result<Response> {
    let body = req.body_string().await?;
    let mut counter: Counter = serde_yaml::from_str(&body)?;
    println!("[1337] count is {}", counter.count);

    counter = handle1337(counter)?;
    let mut res = Response::new(200);
    res.set_body(Body::from_json(&counter)?);
    Ok(res)
}
#[async_std::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut app = tide::new();
    app.at("/42").get(get42).at("/1337").get(get1337);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
