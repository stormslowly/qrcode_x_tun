extern crate mac_utun;
extern crate etherparse;
extern crate packet;


use mac_utun::get_utun;
use etherparse::{SlicedPacket, PacketBuilder, Ipv4HeaderSlice};
use packet::{Builder, Packet, AsPacket};
use std::any::Any;
use std::process::Command;

fn display_bytes(bytes: &[u8]) -> () {
  println!("{:04} bytes", bytes.len());
  for byte in bytes {
    print!("{:02x} ", byte);
  }
  println!("");
}

fn clear_screen() {
  print!("\x1B[2J\x1B[1;1H");
}

fn show_as_qrcode(bytes: &[u8]) -> () {
  qr2term::print_qr(bytes).unwrap();
}

fn main() -> std::io::Result<()> {
  let mut buf = vec![0u8; 5000];

  let (df, tun_name) = get_utun()?;

  format!("ifconfig {tun} inet 10.0.1.1 10.0.1.2 up netmask 255.255.255.0", tun = tun_name);

  Command::new("ifconfig")
    .arg(&tun_name)
    .args(String::from("inet 10.0.1.1 10.0.1.2 up netmask 255.255.255.0").split(" "))
    .spawn()?;

  println!("{} is ready ", &tun_name);
  loop {
    let n = df.recv(&mut buf[..])?;

    clear_screen();
    println!("==>");
    display_bytes(&buf[0..n]);
    show_as_qrcode(&buf[0..n]);

    // 4 is skip the frame number 00 00 00 02
    match Ipv4HeaderSlice::from_slice(&buf[4..n]) {
      Ok(p) => {
        println!("source {source} -> to {to} [{protocol}] len {len}, ttl {ttl}",
                 source = p.source_addr(), to = p.destination_addr(),
                 protocol = p.protocol(),
                 len = p.payload_len(),
                 ttl = p.ttl()
        );


        let icmp = packet::icmp::Packet::new(&buf[23..n]).unwrap();
        let ip = packet::ip::v4::Packet::new(&buf[4..n]).unwrap();


        // println!("icmp {:#?}", icmp);

        // let mut packet = packet::ip::v4::Builder::default()
        //   .id(ip.id()).unwrap()
        //   .ttl(64).unwrap()
        //   .source("10.0.1.2".parse().unwrap()).unwrap()
        //   .destination("10.0.1.1".parse().unwrap()).unwrap()
        //   .icmp().unwrap()
        //   .echo().unwrap().reply().unwrap()
        //   .identifier(1).unwrap()
        //   .sequence(icmp.sequnce()).unwrap()
        //   .payload(icmp.payload()).unwrap()
        //   .build().unwrap();
        //
        //
        // let mut header = vec!(00u8, 00u8, 00u8, 02u8);
        //
        // header.append(&mut packet);


        buf[19] = 0x02;
        buf[23] = 0x01;

        buf[24] = 0x00;
        buf[26] = buf[26].wrapping_add(8);

        println!("<==");
        display_bytes(&buf[0..n]);

        df.send(&buf[0..n])?;
      }
      Err(e) => {
        println!("{:?} error", e);
      }
    }
  }
}

