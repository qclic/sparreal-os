use anyhow::Result;
use network_interface::Addr;
use network_interface::NetworkInterface;
use network_interface::NetworkInterfaceConfig;
use std::{
    fs::{remove_file, File},
    io::{self, stdin, stdout, Read, Write},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

use crate::project::Project;

pub struct UBoot {}

impl UBoot {
    pub fn run(project: &Project) -> Result<()> {
        let dtb_name = "phytium.dtb";
        let dtb_load_addr = "0x90600000";
        let kernel_bin = project
            .compile
            .as_ref()
            .unwrap()
            .bin
            .file_name()
            .unwrap().to_str().unwrap();

        let mut config = project.config.clone();
        let uboot = config.uboot.get_or_insert_default();

        if uboot.net.is_none() {
            println!("请选择网卡：");

            let interfaces = NetworkInterface::show().unwrap();
            for (i, interface) in interfaces.iter().enumerate() {
                let addr_list: Vec<Addr> = interface.addr.to_vec();
                for one in addr_list {
                    if let Addr::V4(v4_if_addr) = one {
                        println!("{}. [{}] - [{}]", i, interface.name, v4_if_addr.ip);
                    }
                }
            }
            let mut input = String::new();
            stdin().read_line(&mut input)?;

            let index = input.trim().parse::<usize>()?;
            println!("选择了 {}", interfaces[index].name);

            uboot.net = Some(interfaces[index].name.clone());
        }

        let mut ip = String::new();

        let interfaces = NetworkInterface::show().unwrap();
        for interface in interfaces.iter() {
            if &interface.name == uboot.net.as_ref().unwrap() {
                let addr_list: Vec<Addr> = interface.addr.to_vec();
                for one in addr_list {
                    if let Addr::V4(v4_if_addr) = one {
                        ip = v4_if_addr.ip.to_string();
                    }
                }
            }
        }

        if uboot.serial.is_none() {
            let ports = serialport::available_ports().expect("No ports found!");
            println!("请选择串口设备:");
            for (i, p) in ports.iter().enumerate() {
                println!("{i}. {}", p.port_name);
            }
            let mut input = String::new();
            stdin().read_line(&mut input)?;

            let port_index = input.trim().parse::<usize>()?;
            println!("选择了 {}", ports[port_index].port_name);

            uboot.serial = Some(ports[port_index].port_name.clone());
        }

        let mut port = serialport::new(uboot.serial.clone().unwrap(), 115_200)
            .timeout(Duration::from_millis(10))
            .open()
            .expect("Failed to open port");

        println!("串口：{}", port.name().unwrap_or_default());
        println!("TFTP: {}", ip);
        println!("内核：{}", kernel_bin);

        let config_str = toml::to_string(&config)?;

        {
            remove_file(&project.config_path)?;
            let mut file = File::create(&project.config_path)?;
            file.write_all(config_str.as_bytes())?;
        }

        let mut buf = [0u8; 1];
        let mut history = Vec::new();

        let input = Arc::new(Mutex::new(Vec::<u8>::new()));

        thread::spawn({
            let input = input.clone();
            move || loop {
                let mut buf = [0u8; 1];
                let _ = stdin().read_exact(&mut buf);
                input.lock().unwrap().push(buf[0]);
            }
        });

        let mut in_shell = false;

        let cmd = format!(
            "dhcp {dtb_load_addr} {ip}:{dtb_name};fdt addr {dtb_load_addr};bootp {ip}:{kernel_bin};booti $loadaddr - {dtb_load_addr}"
        );

        println!("启动命令：{}", cmd);

        println!("等待 U-Boot 启动...");

        loop {
            match port.read(&mut buf) {
                Ok(_t) => {
                    let ch = buf[0];
                    if ch == b'\n' && history.last() != Some(&b'\r') {
                        stdout().write(b"\r")?;
                        if in_shell {
                            history.clear();
                        }
                    }
                    history.push(ch);
                    if !in_shell {
                        let s = String::from_utf8(history.to_vec())?;
                        if s.contains("Hit any key to stop autoboot") {
                            in_shell = true;
                            port.write(b"a")?;
                            sleep(Duration::from_secs(1));

                            port.write_all(cmd.as_bytes())?;
                            port.write_all(b"\r\n")?;
                        }
                    }

                    stdout().write(&buf)?;
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                    stdout().flush()?;

                    let mut input = input.lock().unwrap();
                    if input.is_empty() {
                        continue;
                    }
                    port.write(&input)?;
                    input.clear();
                }
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }
}
