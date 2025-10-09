// IO utilities (networking and file transmission)

use std::io::{prelude::*, BufReader, BufWriter, Error};
use std::net::{TcpStream, TcpListener, ToSocketAddrs, IpAddr, Ipv4Addr, SocketAddr};
use std::fs::File;
use std::time::Duration;

use fxhash::hash64;
use ifcfg::{IfCfg, AddressFamily};


pub fn start_listener(addr: impl ToSocketAddrs) -> Result<TcpListener, Error> {
    Ok(TcpListener::bind(addr)?)
}

pub fn connect_to(addr: impl ToSocketAddrs) -> Result<TcpStream, Error> {
    Ok(TcpStream::connect(addr)?)
}

fn get_host_net_info() -> Option<(SocketAddr, SocketAddr)>{
    let ifaces = IfCfg::get().expect("Failed to get network interfaces");
    for iface in ifaces {
        if &iface.name == "lo" { continue } // Skip loopback device

        for addr in iface.addresses {
            match addr.address_family {
                AddressFamily::IPv4 => {
                    if addr.hop.is_none() { continue } // Skip disconnected
                    let (a, m) = (addr.address, addr.mask);
                    if a.and(m).is_none() { continue } // Skip empty addresses
                    return Some((a?, m?)); 
                },
                AddressFamily::IPv6 => continue,   // TODO: consider addind ipv6 support
                _ => println!("{:?}", addr.address_family),
            }
        }
    }

    None
}

pub fn discover_server_ip() -> Option<Ipv4Addr> {
    let (host_addr, net_mask) = get_host_net_info()?;
    println!("{:?} {:?}", host_addr, net_mask);
    let (host_addr, net_mask) = match (host_addr.ip(), net_mask.ip()) {
        (IpAddr::V4(a), IpAddr::V4(m)) => (a, m),
        _ => None?
    };

    let net_addr = (host_addr & net_mask).to_bits();
    let last_addr = !net_mask.to_bits();
    let pool = (0..=last_addr).map(|x| (Ipv4Addr::from(net_addr | x), 1337));
    
    for addr in pool {
        println!("{:?}", addr);
        if let Ok(mut stream) = TcpStream::connect_timeout(&SocketAddr::from(addr), Duration::from_millis(125)) {
            let mut buf = [0u8; 6];
            if stream.read_exact(&mut buf).is_err() { continue };
            
            let ret = match &buf {
                b"server" => {                      // return server IP
                    stream.write_all(b"search").unwrap();
                    Some(addr.0)
                },
                b"client" => {                      // ask client for server IP
                    let mut addr_buf = [0u8; 4];
                    if stream.read_exact(&mut addr_buf).is_err() { continue };
                    if addr_buf == [0u8; 4] { continue };
                    Some(Ipv4Addr::from(addr_buf))  // server IP candidate. Connect to check
                },
                _ => None,
            };

            return ret;
        }
    }
    
    None
}

pub fn write_to_buf(from: &mut BufReader<impl Read>, to: &mut BufWriter<impl Write>, length: u64) -> Result<(), Error> {
    let mut buf = [0u8]; // one-byte buffer is slow TODO: keep reading & writing until length exceeded
    let mut bytes_left = length;
    let mut bytes_read;

    while bytes_left > 0 {
        bytes_read = from.read(&mut buf)?;
        to.write_all(&buf[..bytes_read])?;
        bytes_left -= bytes_read as u64;
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