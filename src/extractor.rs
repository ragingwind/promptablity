use std::{io::Read, error::Error};

pub fn extract<R>(input: &mut R) -> Result<bool, Box<dyn Error>>
where
    R: Read,
{
    Ok(true)
}

#[cfg(test)]
mod test {
    pub use super::*;

    #[test]
    fn it_ok() {
        assert_eq!(true, true);
    }
}
