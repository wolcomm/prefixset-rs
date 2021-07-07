use std::error::Error;

pub type TestResult = Result<(), Box<dyn Error>>;

pub fn assert_none<T>(opt: Option<T>) -> TestResult {
    if let Some(_) = opt {
        return Err("expected None".into())
    };
    Ok(())
}

pub fn assert_some<T>(opt: Option<T>) -> TestResult {
    if let None = opt {
        return Err("expected Some".into())
    };
    Ok(())
}
