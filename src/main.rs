use std::os::unix::net::{UnixStream, UnixListener};
use std::net::{TcpStream, TcpListener};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short="p")]
    tcp_port: u16,
    #[structopt(short="u")]
    unix_addr: String,
    #[structopt(short="r")]
    reverse: bool,
}

// proxy an incoming tcp stream to a unix stream
fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    if opt.reverse {
        return reverse(opt);
    }

    println!("listening tcp_port {} <---> {} unix", opt.tcp_port, &opt.unix_addr);
    let listener = TcpListener::bind(("127.0.0.1", opt.tcp_port))?;

    for tcp_stream in listener.incoming() {
        println!("incoming connection");
        match tcp_stream {
            Ok(mut tcp_stream) => {
                let mut send_unix_stream = UnixStream::connect(&opt.unix_addr).unwrap();
                let mut recv_unix_stream = send_unix_stream.try_clone()?;
                let mut send_tcp_stream = tcp_stream.try_clone()?;
                std::thread::spawn(move || {
                    std::io::copy(&mut tcp_stream, &mut send_unix_stream).unwrap();
                });
                std::thread::spawn(move || {
                    std::io::copy(&mut recv_unix_stream, &mut send_tcp_stream).unwrap();
                });
            }
            _ => (),
        }
    }

    Ok(())
}

// TODO direction trait, collapse this with main() fn
// current way has a bunch of copy-paste
fn reverse(opt: Opt) -> std::io::Result<()> {
    println!("listening unix {} <---> {} tcp_port", &opt.unix_addr, opt.tcp_port);
    let listener = UnixListener::bind(&opt.unix_addr)?;
    for unix_stream in listener.incoming() {
        println!("incoming connection");
        match unix_stream {
            Ok(mut unix_stream) => {
                let mut send_tcp_stream = TcpStream::connect(("127.0.0.1", opt.tcp_port)).unwrap();
                let mut recv_tcp_stream = send_tcp_stream.try_clone()?;
                let mut send_unix_stream = unix_stream.try_clone()?;
                std::thread::spawn(move || {
                    std::io::copy(&mut unix_stream, &mut send_tcp_stream).unwrap();
                });
                std::thread::spawn(move || {
                    std::io::copy(&mut recv_tcp_stream, &mut send_unix_stream).unwrap();
                });
            }
            _ => (),
        }
    }

    Ok(())
}
