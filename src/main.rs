// use tun_tap_mac;


extern crate mac_utun;
extern crate etherparse;
extern crate packet;


use mac_utun::get_utun;
use etherparse::{SlicedPacket, PacketBuilder, Ipv4HeaderSlice};
use packet::{Builder, Packet};

fn main() -> std::io::Result<()> {
  let mut buf = vec![0u8; 5000];

  let (df, tun_name) = get_utun()?;

  println!("{}", tun_name);


  loop {
    let n = df.recv(&mut buf[..])?;

    println!("{:?} bytes got", n);


    for n in &buf[0..n] {
      print!("{:02x} ", n);
    };

    println!("");

    // println!("{:?}", &buf[0..n]);


    let ingreessIp = packet::ip::Packet::new(&buf[4..n]).unwrap();

    match Ipv4HeaderSlice::from_slice(&buf[4..n]) {
      Ok(p) => {
        println!("{:?}", p);

        println!("source {source} -> to {to} [{protocol}] len {len}, ttl {ttl}",
                 source = p.source_addr(), to = p.destination_addr(),
                 protocol = p.protocol(),
                 len = p.payload_len(),
                 ttl = p.ttl()
        );


        let parsed = packet::icmp::Packet::new(&buf[23..n]).unwrap();

        println!("{:?}", parsed);

        let toSend = packet::icmp::Builder::default()
          .echo().unwrap()
          .reply().unwrap()
          .payload(parsed.payload()).unwrap()
          .build().unwrap();


        let  ipPacket = packet::ip::Builder::default()
          .v4().unwrap()
          .source(ingreessIp.)
          .payload(toSend).unwrap()


        for n in toSend {
          print!("{:02x} ", n);
        };

        println!("")





        // PacketBuilder::ipv4()
      }
      Err(e) => {
        println!("{:?} error", e);
      }
    }


    let sendn = df.send(&buf[0..n])?;

    println!("{} nbyte send back", sendn);
  }
}
