use core::{fmt, time};
use std::{env, error::Error, time::UNIX_EPOCH};

use crate::bluetooth::Device;
use serde::Deserialize;
use std::time::SystemTime;

pub struct server {
    bluetooth: Device,
}
#[derive(Debug)]
pub struct Servererror {
    message: String,
}
impl fmt::Display for Servererror {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Error for Servererror {
    fn description(&self) -> &str {
        &self.message
    }
}
#[repr(C)]
struct response {
    pub command: u8,
}
#[derive(Deserialize)]
struct Departuren {
    timestamp: u64,
}
#[derive(Deserialize)]
struct StopTI {
    departure: Departuren,
}

#[derive(Deserialize)]
struct Departure {
    stopTimeInstance: StopTI,
}
#[derive(Deserialize)]
struct Apiresponse {
    departures: Vec<Departure>,
}
const FRAMEESC: u8 = 0xFD;
const FRAMESTART: u8 = 0xFF;
const FRAMEEND: u8 = 0xFE;
const ESCCODE: u8 = 0x5a;
impl server {
    pub fn new(bluetooth: Device) -> Self {
        server { bluetooth }
    }
    //ensure that address is already established
    pub async fn start(&mut self) -> Result<(), Servererror> {
        match self.bluetooth.start_comm().await {
            Ok(_) => println!("Connection established"),
            Err(e) => return Err(Servererror { message: e.message }),
        }

        Ok(())
    }
    fn read_packet(&self, buffer: &[u8]) -> Result<response, Servererror> {
        let mut r = response { command: 0 };
        for i in 0..buffer.len() {
            if buffer[i] != FRAMESTART {
                continue;
            }
            if buffer[i + 1] != FRAMESTART {
                continue;
            }
            if buffer[i + 3] != FRAMEEND {
                continue;
            }
            r.command = buffer[i + 2];
        }
        if r.command == 0 {
            return Err(Servererror {
                message: "Invalid Packet".to_string(),
            });
        }
        Ok(r)
    }
    async fn gather_data() {
        let e = env::args().collect::<Vec<String>>();
        //get STOP from env
        let i = e.binary_search(&"STOP".to_string()).unwrap();
        let a = &e[i];
        //get unix time stamp
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let res=reqwest::get(format!("https://anytrip.com.au/api/v3/region/au2/departures/au2%3A{a}?limit=1&offset=0&ts={ts}"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let apiresponse: Apiresponse = serde_json::from_str(&res).unwrap();
    }
    pub async fn run(&mut self) -> Result<(), Servererror> {
        loop {
            let buffer = match self.bluetooth.receive_message().await {
                Ok(b) => b,
                Err(e) => return Err(Servererror { message: e.message }),
            };
            if buffer.len() == 0 {
                continue;
            }
            let bufu8 = buffer.as_slice();
            let response = self.read_packet(bufu8);
            match response {
                Ok(r) => {
                    if r.command != 0x01 {
                        println!("Invalid Command");
                        continue;
                    }
                }
                Err(e) => {
                    println!("Error Reading Packet {:?}", e);
                    continue;
                }
            }
        }
    }
}
