use async_trait::async_trait;
use fast_socks5::client::Socks5Datagram;
use fast_socks5::util::target_addr::TargetAddr;
use log::info;
use stun::addr::*;
use stun::agent::*;
use stun::client::*;
use stun::message::*;
use stun::xoraddr::*;
use stun::Error;

use clap::{arg, command, Arg, ArgAction};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::lookup_host;
use tokio::net::TcpStream;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let matches = command!() // requires `cargo` feature
        .arg(
            arg!([SERVER] "STUN server host and port. If PORT is omitted, it defaults to 3478")
                .value_name("HOST:PORT")
                .default_value("stun.cloudflare.com"),
        )
        .arg(
            Arg::new("ipv4_only")
                .long("ipv4-only")
                .short('4')
                .required(false)
                .help("Do not use IPv6 addresses when connecting to STUN server")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("host:port")
                .long("socks5")
                .short('s')
                .required(false)
                .help("Use SOCKS5 udp proxy"),
        )
        .get_matches();

    let (handler_tx, mut handler_rx) = tokio::sync::mpsc::unbounded_channel();

    let mut server = matches.get_one::<String>("SERVER").unwrap().clone();
    if server.as_str().starts_with('[') {
        // IPv6 literal
        if !server.contains("]:") {
            server.push_str(":3478");
        }
    } else if !server.contains(':') {
        server.push_str(":3478");
    }

    let ipv4_only = matches.get_flag("ipv4_only");

    let addr = lookup_host(server)
        .await
        .expect("failed to parse/resolve server address and port number")
        .find(|addr| !ipv4_only || addr.is_ipv4())
        .expect("unable to resolve remote host name");

    info!("Server address is: {}", addr);

    let mut client = if let Some(proxy) = matches.get_one::<String>("socks5") {
        println!("Using SOCKS5 proxy: {}", proxy);
        let backing_socket = TcpStream::connect(proxy).await?;
        let u = fast_socks5::client::Socks5Datagram::bind(backing_socket, "0.0.0.0:0")
            .await
            .map_err(|e| Error::Other(e.to_string()))?;

        let conn = Arc::new(Socks5Udp(u, TargetAddr::Ip(addr)));
        ClientBuilder::new().with_conn(conn).build()?
    } else {
        let conn = UdpSocket::bind(if addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        })
        .await?;

        conn.connect(addr).await?;
        ClientBuilder::new().with_conn(Arc::new(conn)).build()?
    };

    let mut msg = Message::new();
    msg.build(&[Box::<TransactionId>::default(), Box::new(BINDING_REQUEST)])?;

    client.send(&msg, Some(Arc::new(handler_tx))).await?;

    if let Some(event) = handler_rx.recv().await {
        let msg = event.event_body?;
        let mut xor_addr = XorMappedAddress::default();
        if xor_addr.get_from(&msg).is_ok() {
            println!("{}", xor_addr.ip);
        } else {
            info!("XOR-MAPPED-ADDRESS attribute does not exist, trying MAPPED-ADDRESS instead");

            let mut addr = MappedAddress::default();
            addr.get_from(&msg)?;
            println!("{}", addr.ip);
        }
    }

    client.close().await?;

    Ok(())
}

struct Socks5Udp(Socks5Datagram<TcpStream>, TargetAddr);
use webrtc_util::Result as WResult;
#[async_trait]
impl webrtc_util::Conn for Socks5Udp {
    async fn connect(&self, _addr: SocketAddr) -> WResult<()> {
        unimplemented!()
    }
    async fn recv(&self, buf: &mut [u8]) -> WResult<usize> {
        let r = self.0.recv_from(buf).await;
        match r {
            Ok((n, _)) => WResult::Ok(n),
            Err(e) => Err(webrtc_util::Error::Other(e.to_string())),
        }
    }
    async fn recv_from(&self, _buf: &mut [u8]) -> WResult<(usize, SocketAddr)> {
        unimplemented!()
    }
    async fn send(&self, buf: &[u8]) -> WResult<usize> {
        let r = match self.1 {
            TargetAddr::Ip(addr) => self.0.send_to(buf, addr).await,
            TargetAddr::Domain(ref domain, port) => {
                self.0.send_to(buf, (domain.as_ref(), port)).await
            }
        };
        match r {
            Ok(n) => WResult::Ok(n),
            Err(e) => Err(webrtc_util::Error::Other(e.to_string())),
        }
    }
    async fn send_to(&self, buf: &[u8], target: SocketAddr) -> WResult<usize> {
        let r = self.0.send_to(buf, target).await;
        match r {
            Ok(n) => WResult::Ok(n),
            Err(e) => Err(webrtc_util::Error::Other(e.to_string())),
        }
    }
    async fn close(&self) -> WResult<()> {
        Ok(())
    }
    fn local_addr(&self) -> WResult<SocketAddr> {
        unimplemented!()
    }
    fn remote_addr(&self) -> Option<SocketAddr> {
        unimplemented!()
    }
}
