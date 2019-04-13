use std::os::unix::net::UnixStream;
use std::net::TcpListener;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short="p")]
    http_port: u16,
    #[structopt(short="u")]
    unix_server_addr: String,
}

// proxy an incoming http stream to a unix stream
fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    let listener = TcpListener::bind(("127.0.0.1", opt.http_port))?;

    for tcp_stream in listener.incoming() {
        println!("incoming connection");
        match tcp_stream {
            Ok(mut tcp_stream) => {
                let mut send_unix_stream = UnixStream::connect(&opt.unix_server_addr).unwrap();
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
