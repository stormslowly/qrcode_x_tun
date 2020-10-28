extern crate mac_utun;
extern crate etherparse;
extern crate packet;
extern crate opencv;
extern crate image;
extern crate bardecoder;


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
use image::{ImageBuffer, DynamicImage, FilterType};
use opencv::core::{CV_8UC3, Vec3};
use std::ops::Deref;
use bardecoder::decode::Decode;
use bardecoder::prepare::BlockedMean;

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

      let width = frame.size()?.width as u32;
      let height = frame.size()?.height as u32;


      println!("{w} {h}", w = width, h = height);

      let fm = ImageBuffer::from_fn(640, 360,
                                    |c, r| {
                                      let v3u: &Vec3<u8> = frame.at_2d((r * 2) as i32, (c * 2) as i32).unwrap();

                                      let vec = v3u.deref().to_vec();

                                      let r = vec.get(0).unwrap();
                                      let g = vec.get(1).unwrap();
                                      let b = vec.get(2).unwrap();

                                      image::Rgb([*g, *g, *g])
                                    },
      );

      let dynamicImage = DynamicImage::ImageRgb8(fm);

      let r = decoder.decode(&dynamicImage);

      println!("{:?}", r);

      dynamicImage.save("./test.png");
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
  let img = image::open("./test.jpg").unwrap();

  // Use default decoder
  let mut dd = bardecoder::default_builder();

  // Use some different arguments in one of the default components
  dd.prepare(Box::new(BlockedMean::new(9, 50)));

  // Build the actual decoder
  let decoder = dd.build();


  let results = decoder.decode(&img);
  println!("{:?}", results);


  for result in results {
    println!("{}", result.unwrap());
  }

  // run().unwrap();
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

