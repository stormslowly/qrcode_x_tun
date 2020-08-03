// use tun_tap_mac;


extern crate mac_utun;

use mac_utun::get_utun;

fn main() -> std::io::Result<()> {
  let mut buf = vec![0u8; 5000];

  let (df, tun_name) = get_utun()?;

  println!("{}", tun_name);


  loop {
    let n = df.recv(&mut buf[..])?;

    println!("{:?} bytes got", n);

    println!("{:?}", &buf[0..n]);
  }

}
