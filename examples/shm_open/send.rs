pub mod shmbuf;

use std::{error::Error, io};

use shmbuf::{Shmbuf, BUF_SIZE};
use shmoo::{FromShm, Shm};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    let shmpath = &args[1];
    let string = &args[2];

    if string.len() > BUF_SIZE {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("string must be less than {} bytes long", BUF_SIZE),
        )));
    }

    let mut shm = Shm::options().read(true).write(true).open(shmpath)?;

    let shmbuf = Shmbuf::<BUF_SIZE>::from_shm_mut(&mut shm)?;

    shmbuf.write(string.as_bytes());

    // Tell peer that it can now access shared memory.
    shmbuf.sem1.post()?;

    // Wait until peer has modified shared memory.
    shmbuf.sem2.wait()?;

    let result = String::from_utf8(shmbuf.buf.to_vec()).unwrap();
    println!("{}", result);
    assert_eq!(result, result.to_ascii_uppercase());

    Ok(())
}
