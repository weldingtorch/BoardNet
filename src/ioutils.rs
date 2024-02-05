// IO utilities (networking and file transmission)

use std::io::{BufReader, BufWriter, Error};
use std::io::prelude::*;
pub use std::net::{TcpStream, TcpListener, ToSocketAddrs, Ipv4Addr, Shutdown};
pub use std::fs::File;

pub use fxhash::hash64;


pub fn start_listener(addr: impl ToSocketAddrs) -> Result<TcpListener, Error> {
    Ok(TcpListener::bind(addr)?)
}

pub fn connect_to(addr: impl ToSocketAddrs) -> Result<TcpStream, Error> {
    Ok(TcpStream::connect(addr)?)
}

pub fn discover_master_ip(worker_addr: Ipv4Addr) {
    // TODO: brute force ip
    let octs = &worker_addr.octets()[0..3];
    let pool = (0..255).map(|x| (Ipv4Addr::new(octs[0], octs[1], octs[2], x), 1337));
    for addr in pool {
        let stream = TcpStream::connect(addr);
    }
    todo!();
}

pub fn write_to_buf(from: &mut BufReader<impl Read>, to: &mut BufWriter<impl Write>, length: u64) -> Result<(), Error> {
    let mut buf = [0u8]; // one-byte buffer is slow TODO: keep reading & writing until length exceeded
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

pub fn send_data(stream: &mut BufWriter<&TcpStream>, data: &[u8]) -> Result<(), Error> {
    let length = data.len().try_into().unwrap();  // Assume we run this on >=64-bit OS
    send_u64(stream, length)?;
    println!("Sending data. Size: {}B", length);
    stream.write_all(data)?;
    stream.flush()?;
    Ok(())
}

pub fn recieve_data(stream: &mut BufReader<&TcpStream>) -> Result<Vec<u8>, Error> {
    // TODO: refactor?
    let length = recieve_u64(stream)?.try_into().unwrap(); // Assume we run this on >=64-bit OS
    let mut buf: Vec<u8> = vec![0; length];
    println!("Recieving data. Size: {}B", length);
    stream.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn send_data_buffered(stream: &mut BufWriter<&TcpStream>, source: &mut BufReader<impl Read>, length: u64) -> Result<(), Error> {
    send_u64(stream, length)?;
    println!("Sending data. Size: {}B", length);
    write_to_buf(source, stream, length)?;
    Ok(())
}

pub fn recieve_data_buffered(stream: &mut BufReader<&TcpStream>, destination: &mut BufWriter<impl Write>) -> Result<(), Error> {
    let length = recieve_u64(stream)?;
    println!("Recieving data. Size: {}B", length);
    write_to_buf(stream, destination, length)?;
    Ok(())
}

pub fn get_bytes_of(path: &str) -> Result<(BufReader<File>, u64), Error>{
    let file = File::open(path)?;
    let length = file.metadata()?.len();
    Ok((BufReader::new(file), length))
}

fn get_unbuffered_bytes_of(path: &str) -> Result<Box<[u8]>, Error> {
    let mut reader = get_bytes_of(path)?.0;
    let mut data = vec![];
    reader.read_to_end(&mut data)?;
    Ok(data.into_boxed_slice())
}

pub fn get_hash_of(path: &str/*, cached_data: &mut CachedData*/) -> Result<u64, Error> {
    //if cached_data.client_hash != 0u64 {
    //    cached_data.client_hash
    //} else {
        let client_hash = hash64(&get_unbuffered_bytes_of(path)?);
        //cached_data.client_hash = client_hash;
        //client_hash 
    //}
    Ok(client_hash)
}