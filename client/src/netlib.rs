pub use std::io::{BufReader, BufWriter, Error, Read, Write};
pub use std::net::{TcpStream, TcpListener, ToSocketAddrs, Shutdown};


pub fn start_listener(addr: impl ToSocketAddrs) -> Result<TcpListener, Error> {
    Ok(TcpListener::bind(addr)?)
}

pub fn connect_to(addr: impl ToSocketAddrs) -> Result<TcpStream, Error> {
    Ok(TcpStream::connect(addr)?)
}

pub fn recieve_u64(reader: &mut BufReader<&TcpStream>) -> Result<u64, Error> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

pub fn recieve_data(reader: &mut BufReader<&TcpStream>) -> Result<Box<[u8]>, Error> {
    let size = recieve_u64(reader)?;
    println!("recieving data. Size: {}", size);
    let mut buf = vec![0; size.try_into().unwrap()].into_boxed_slice();
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn send_u64(writer: &mut BufWriter<&TcpStream>, data: u64) -> Result<(), Error> {
    println!("Sending data: {}", data);
    writer.write_all(&data.to_be_bytes())?;
    writer.flush()?; // Maybe I don't need to flush. Will check that later.
    Ok(())
}

pub fn send_data(writer: &mut BufWriter<&TcpStream>, data: &[u8]) -> Result<(), Error> {
    send_u64(writer, data.len() as u64)?;
    writer.write_all(data)?;
    writer.flush()?; // This one too.
    Ok(())
}
