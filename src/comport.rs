use std::io::{self, ErrorKind};
use std::error::Error;
use std::time::Duration;

// If this is giving you an error, install the `serialport` crate, if not done already.
use serialport::SerialPort;

pub fn open_com_port() -> Result<Box<dyn SerialPort>, Box<dyn Error>> {
    let port = serialport::new("/dev/ttyUSB0", 9600)
        .timeout(Duration::from_secs(1))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()?;
    Ok(port)
}

// pub fn read_from_com_port(port: &mut dyn SerialPort) -> std::io::Result<String> {
//     let mut serial_buf = vec![0; 32];
//     match port.read(serial_buf.as_mut_slice()) {
//         Ok(n) => Ok(String::from_utf8_lossy(&serial_buf[..n]).to_string()),
//         Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(String::new()),
//         Err(e) => Err(e),
//     }
// }

pub fn write_to_com_port(port: &mut dyn SerialPort, data: &str) -> std::io::Result<()> {
    port.write_all(data.as_bytes())?;
    port.flush()?;
    Ok(())
}


pub fn continuous_read(port: &mut dyn SerialPort) -> io::Result<String> {
    let mut buffer = [0u8; 128];
    let mut output_string = String::new();

    loop {
        match port.read(&mut buffer) {
            Ok(n) if n > 0 => {
                // Process or display the data
                println!("Received bytes: {:?}", &buffer[..n]);
                output_string = String::from_utf8_lossy(&buffer[..n]).to_string();
                
                // Example exit condition if data is "quit\n"
                if &buffer[..n] == b"quit\n" {
                    println!("'quit' received. Exiting read loop.");
                    break;
                }
            }
            Ok(_) => {
                // 0 bytes read could mean a timeout or no data. 
                // Possibly keep looping or break if desired.
            }
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                // If you want to handle timeouts differently, do so here
                eprintln!("Read timeout—no data received in time.");
            }
            Err(e) => {
                // On any other read error, log and break out of the loop
                eprintln!("Read error: {:?}", e);
                break;
            }
        }
    }

    // Return Ok once we're done or if an error was handled above
    Ok(output_string)
}