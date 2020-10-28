extern crate mac_utun;
extern crate etherparse;
extern crate packet;
extern crate opencv;
extern crate image;
extern crate bardecoder;
extern crate quirs;
extern crate futures;


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
use quirs::{Decoder, Image, Vec2D, Info};
use futures::future::Err;

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

            let fm = ImageBuffer::from_fn(640, 480,
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

            dynamicImage.save("./test.png");
            let r = decode_screen(&dynamicImage);

            println!("{:?}", r);

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

fn decode_screen(img: &DynamicImage) {
    let mut decoder = Decoder::new().expect("can't create Qui-RS decoder");

    let gray = img.to_luma();
    println!("{}", gray.width());

    let bytes = gray.to_vec();

    let img = Image::new(&bytes,
                         Vec2D {
                             x: gray.width() as usize,
                             y: gray.height() as usize,
                         }).expect("xxx");

    let results = decoder.decode_image(&img).expect("decode ok");

    println!("{:?}", results);
    for code in results {
        match code{
            Ok(code) => {
                println!("{:?}", code);
                let info = code.decode().unwrap_or(Info::frow_raw(quirc_data::Default()));
                println!("{}", info.as_str());
            },
            Err(e) => println!("{}", e),

        }

        // let code = code.expect("can't detect code");
        // let info = code.decode().expect("can't decode code");
        //
        // println!("{}", info.as_str().expect("code contents not ASCII"));
    }
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

