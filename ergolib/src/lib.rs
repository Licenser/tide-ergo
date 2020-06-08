pub enum Error {
    DontLikeIt,
    DivisionByZero,
}

pub fn nth42(n: usize) -> Result<usize, Error> {
    match n {
        13 => Err(Error::DontLikeIt),
        0 => Err(Error::DivisionByZero),
        _ => Ok(42 / n),
    }
}

pub fn nth1337(n: usize) -> Result<usize, Error> {
    match n {
        23 => Err(Error::DontLikeIt),
        0 => Err(Error::DivisionByZero),
        _ => Ok(1337 / n),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
