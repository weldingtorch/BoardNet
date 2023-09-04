pub use std::io::{BufReader, BufWriter, Error};
use std::io::prelude::*;
pub use std::net::{TcpStream, TcpListener, ToSocketAddrs, Shutdown};


pub fn start_listener(addr: impl ToSocketAddrs) -> Result<TcpListener, Error> {
    Ok(TcpListener::bind(addr)?)
}

pub fn connect_to(addr: impl ToSocketAddrs) -> Result<TcpStream, Error> {
    Ok(TcpStream::connect(addr)?)
}

fn write_to_buf(from: &mut BufReader<impl Read>, to: &mut BufWriter<impl Write>, length: u64) -> Result<(), Error> {
    let mut buf = [0u8]; // one-byte buffer may be slow TODO: check how to copy buf to buf
    for _ in 0..length {
        from.read_exact(&mut buf)?;
        to.write_all(&buf)?;
    }
    to.flush()?;
    Ok(())
}

// In this case buffering doesn't help
pub fn send_u64(stream: &mut BufWriter<&TcpStream>, data: u64) -> Result<(), Error> {
    stream.write_all(&data.to_be_bytes())?;
    stream.flush()?;
    Ok(())
}

// In this case buffering doesn't help
pub fn recieve_u64(stream: &mut BufReader<&TcpStream>) -> Result<u64, Error> {
    let mut buf = [0u8; 8];
    stream.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

pub fn send_data(stream: &mut BufWriter<&TcpStream>, source: &mut BufReader<impl Read>, length: u64) -> Result<(), Error> {
    send_u64(stream, length)?;
    println!("Sending data. Size: {}B", length);
    write_to_buf(source, stream, length)?;
    Ok(())
}

pub fn recieve_data(stream: &mut BufReader<&TcpStream>, destination: &mut BufWriter<impl Write>) -> Result<(), Error> {
    let length = recieve_u64(stream)?;
    println!("Recieving data. Size: {}B", length);
    write_to_buf(stream, destination, length)?;
    Ok(())
}
