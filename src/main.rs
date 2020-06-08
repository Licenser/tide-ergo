use async_std;
use ergolib;
use tide::{Body, Request, Response};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Counter {
    count: usize,
}

struct AppError(ergolib::Error);

impl std::error::Error for AppError {}

impl std::fmt::Debug for AppError {
    // we have to manually define Debug becuase the external library
    // didn't derive it
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ergolib::Error::DontLikeIt => write!(f, "Don't like it"),
            ergolib::Error::DivisionByZero => write!(f, "Division by zero"),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<ergolib::Error> for AppError {
    fn from(e: ergolib::Error) -> Self {
        Self(e)
    }
}

// this could just as easily be inlined in the middleware, or a normal
// method on AppError
impl Into<Response> for &AppError {
    fn into(self) -> Response {
        match self.0 {
            ergolib::Error::DontLikeIt => {
                let mut res = Response::new(500);
                res.set_body("The server doesn't like this number");
                res
            }

            ergolib::Error::DivisionByZero => {
                let mut res = Response::new(422);
                res.set_body("count can't be zero");
                res
            }
        }
    }
}

//only renamed for clarity
type AppResult = std::result::Result<Counter, AppError>;

//unchanged
fn handle42(mut counter: Counter) -> AppResult {
    counter.count -= 1;
    counter.count = ergolib::nth42(counter.count)?;
    Ok(counter)
}

//unchanged
fn handle1337(mut counter: Counter) -> AppResult {
    counter.count -= 1;
    counter.count = ergolib::nth1337(counter.count)?;
    Ok(counter)
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();

    app.middleware(tide::After(|res: tide::Response| async move {
        if let Some(e) = res.downcast_error::<AppError>() {
            Ok(e.into())
        } else if let Some(e) = res.downcast_error::<serde_yaml::Error>() {
            let mut res = Response::new(422);
            res.set_body(format!("YAML error: {}", e));
            Ok(res)
        } else {
            Ok(res)
        }
    }));

    app.at("/42").post(|mut req: Request<()>| async move {
        let body = req.body_string().await?;
        let mut counter: Counter = serde_yaml::from_str(&body)?;
        println!("[42] count is {}", counter.count);
        counter = handle42(counter)?;
        let mut res = Response::new(200);
        res.set_body(Body::from_json(&counter)?);
        Ok(res)
    });

    app.at("/1337").post(|mut req: Request<()>| async move {
        let body = req.body_string().await?;
        let mut counter: Counter = serde_yaml::from_str(&body)?;
        println!("[1337] count is {}", counter.count);
        counter = handle1337(counter)?;
        let mut res = Response::new(200);
        res.set_body(Body::from_json(&counter)?);
        Ok(res)
    });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
