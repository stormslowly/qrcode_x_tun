extern crate mac_utun;
extern crate etherparse;
extern crate packet;
extern crate opencv;
extern crate image;


use mac_utun::get_utun;
use etherparse::{SlicedPacket, PacketBuilder, Ipv4HeaderSlice};
use packet::{Builder, Packet, AsPacket};
use std::any::Any;
use std::process::Command;


use opencv::{
  core,
  highgui,
  prelude::*,
  videoio,
};
use image::ImageBuffer;
use opencv::core::{CV_8UC3, Vec3};

fn run() -> opencv::Result<()> {
  let window = "video capture";
  highgui::named_window(window, 1)?;
  let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;  // 0 is the default camera
  let opened = videoio::VideoCapture::is_opened(&cam)?;
  if !opened {
    panic!("Unable to open default camera!");
  }

  let decoder = bardecoder::default_decoder();

  loop {
    let mut frame = core::Mat::default()?;
    cam.read(&mut frame)?;
    if frame.size()?.width > 0 {
      highgui::imshow(window, &mut frame)?;
      print!("{:?}\n", frame);

      let width = frame.size()?.width as u32;//u32::try_from(frame.size()?.width)?;
      let height = frame.size()?.height as u32;


      println!("{w} {h}", w = width, h = height);


      let x = frame.at_2d::<Vec3<u8>>(0, 0).unwrap();

      println!("{:?}", x);


      let fm = ImageBuffer::from_fn(width, height,
                                    |c, r| {
                                      let v3u = frame.at_2d::<Vec3<u8>>(c as i32, r as i32).unwrap();

                                      image::Luma
                                    },
      );


      fm?.save("./test.png")?;
    }
    let key = highgui::wait_key(10)?;
    if key > 0 && key != 255 {
      break;
    }
  }
  Ok(())
}

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

fn main() {
  run().unwrap();
}

fn main_1() -> std::io::Result<()> {
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

