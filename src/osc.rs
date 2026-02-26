use crossbeam_channel::{bounded, select, unbounded};
use rosc::OscPacket;
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::Duration,
};
use thiserror::Error;
use tokio::task::JoinHandle;

use crate::ui::ConsoleState;

pub fn handle_osc(opt: Option<OscPacket>, state: &mut ConsoleState) {
    use crate::osc::is_osc_address;
    use rosc::OscType;
    let osc_addresser = &state.osc_address_manager;
    if is_osc_address(&opt, &osc_addresser.master_volume) {
        if let Some(osc) = &opt {
            match osc {
                OscPacket::Message(osc_message) => {
                    osc_message.args.iter().for_each(|osc_type| match osc_type {
                        OscType::Float(x) if (0.0..=1.5).contains(x) => {
                            state.master_volume = *x;
                        }
                        OscType::Double(x) if (0.0..=150.0).contains(x) => {
                            state.master_volume = (*x / 100.0) as f32;
                        }
                        OscType::Int(x) if (0..=150).contains(x) => {
                            state.master_volume = (*x / 100) as f32;
                        }
                        OscType::Long(x) if (0..=1500).contains(x) => {
                            state.master_volume = (*x / 1000) as f32;
                        }
                        _ => {
                            println!("Invalid OSCType or Invalid Value {osc:?} for Master Volume");
                        }
                    });
                }
                _ => {}
            }
        }
    }
    if is_osc_address(&opt, &osc_addresser.master_dmx) {
        if let Some(osc) = &opt {
            match osc {
                OscPacket::Message(osc_message) => {
                    osc_message.args.iter().for_each(|osc_type| match osc_type {
                        OscType::Float(x) if (0.0..=1.0).contains(x) => {
                            state.master_dimmer = *x;
                        }
                        OscType::Int(x) if (0..=100).contains(x) => {
                            state.master_dimmer = (*x / 100) as f32;
                        }
                        OscType::Double(x) if (0.0..=100.0).contains(x) => {
                            state.master_dimmer = (*x / 100.0) as f32;
                        }
                        OscType::Long(x) if (0..=1000).contains(x) => {
                            state.master_dimmer = (x / 1000) as f32;
                        }
                        _ => {
                            println!("Invalid OSCType or Invalid Value {osc:?} for Master DMX");
                        }
                    });
                }
                _ => {}
            }
        }
    }
    let exec_dimmer = state.executors.iter_mut().find(|exec| {
        let id = exec.id as i8 + 1;
        is_osc_address(
            &opt,
            format!(
                "{}{id}{}",
                state.osc_address_manager.executor_identifier,
                state.osc_address_manager.executor_dimmer
            ),
        )
    });
    if let Some(exec) = exec_dimmer {
        if let Some(osc) = &opt {
            match osc {
                OscPacket::Message(osc_message) => {
                    osc_message.args.iter().for_each(|osc_type| match osc_type {
                        OscType::Float(x) if (0.0..=1.0).contains(x) => {
                            if exec.cue_list.len() > 0 {
                                exec.fader_level = *x;
                            }
                        }
                        _ => {}
                    })
                }
                _ => {}
            }
        }
    }
    let exec_go = state.executors.iter_mut().find(|exec| {
        let id = exec.id as i8 + 1;
        is_osc_address(
            &opt,
            format!(
                "{}{id}{}",
                state.osc_address_manager.executor_identifier,
                state.osc_address_manager.executor_go
            ),
        )
    });
    if let Some(exec) = exec_go {
        exec.go();
    }
    let exec_go_back = state.executors.iter_mut().find(|exec| {
        let id = exec.id as i8 + 1;
        is_osc_address(
            &opt,
            format!(
                "{}{id}{}",
                state.osc_address_manager.executor_identifier,
                state.osc_address_manager.executor_go_back
            ),
        )
    });
    if let Some(exec) = exec_go_back {
        exec.go_back();
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid IP:Port formatting")]
    InvalidIpFormatting,
    #[error("Binding Error: {0}")]
    BindingError(String),
}
pub struct OSCManager {
    thread_stopper: crossbeam_channel::Sender<()>,
    thread_handle: JoinHandle<()>,
    osc_receiver: crossbeam_channel::Receiver<Option<OscPacket>>,
    osc_history: Vec<OscPacket>,
}
impl Drop for OSCManager {
    fn drop(&mut self) {
        if let Ok(_) = self.thread_stopper.send(()) {
        } else {
            println!("Stopping thread message failed to send");
        }
        self.thread_handle.abort_handle().abort();
        println!("Dropping OSC Manager");
    }
}
impl OSCManager {
    pub fn from(address: impl Into<String>) -> Result<Self, Error> {
        use scan_fmt::scan_fmt;
        let address = address.into();
        match scan_fmt!(&address, "{}.{}.{}.{}:{}", u8, u8, u8, u8, u16) {
            Ok((a, b, c, d, port)) => {
                match UdpSocket::bind(SocketAddr::new(
                    std::net::IpAddr::V4(Ipv4Addr::new(a, b, c, d)),
                    port,
                )) {
                    Ok(socket) => {
                        let _ = socket.set_read_timeout(Some(Duration::from_millis(50)));
                        let channels = unbounded();
                        let osc_channels = bounded(1);
                        let handle = tokio::spawn(async move {
                            let thread_stop = channels.1;
                            let osc_sender = osc_channels.0;
                            let mut buffer = [0u8; rosc::decoder::MTU];
                            'task: loop {
                                select! {
                                    recv(thread_stop) -> _msg => {
                                        break 'task;
                                    }
                                    send(osc_sender,match socket.recv_from(&mut buffer) {
                                        Ok((size, _)) => {
                                            if let Ok((_, packet)) =
                                                rosc::decoder::decode_udp(&buffer[..size])
                                            {
                                                Some(packet)
                                            } else {
                                                None
                                            }
                                        }
                                        Err(_) => {None}
                                    }) -> res => {
                                        match res {
                                            Ok(_) => {},
                                            Err(_) => {},
                                        }
                                    }
                                }
                            }
                            println!("OSC Thread stopped");
                        });
                        Ok(Self {
                            thread_stopper: channels.0,
                            thread_handle: handle,
                            osc_receiver: osc_channels.1,
                            osc_history: Default::default(),
                        })
                    }
                    Err(e) => Err(Error::BindingError(e.to_string())),
                }
            }
            _ => Err(Error::InvalidIpFormatting),
        }
    }
    pub fn get_osc(&mut self) -> Option<OscPacket> {
        if let Ok(opt_packet) = self.osc_receiver.try_recv() {
            if let Some(packet) = opt_packet {
                self.osc_history.push(packet.clone());
                if self.osc_history.len() > 20 {
                    self.osc_history.remove(0);
                }
                return Some(packet);
            }
        }
        None
    }
    pub fn get_osc_history(&self) -> &Vec<OscPacket> {
        &self.osc_history
    }
}

pub fn is_osc_address(opt: &Option<OscPacket>, addr: impl std::fmt::Display) -> bool {
    let address = addr.to_string();
    match opt {
        Some(p) => match p {
            OscPacket::Message(osc_message) => osc_message.addr == address,
            OscPacket::Bundle(osc_bundle) => false,
        },
        None => false,
    }
}

pub struct OSCNaming {
    /// The OSC Adress name to modify the master volume
    pub master_volume: String,
    /// The OSC Adress name to modify the master dmx
    pub master_dmx: String,
    /// Executor OSC Identifier
    pub executor_identifier: String,
    /// Executor Dimmer OSC
    pub executor_dimmer: String,
    /// Executor GO OSC
    pub executor_go: String,
    /// Executor GO BACK OSC
    pub executor_go_back: String,
}

impl Default for OSCNaming {
    fn default() -> Self {
        Self {
            master_volume: String::from("/MasterVolume"),
            master_dmx: String::from("/MasterDMX"),
            executor_identifier: String::from("/Executor"),
            executor_dimmer: String::from("/Dimmer"),
            executor_go: String::from("/Go"),
            executor_go_back: String::from("/GoBack"),
        }
    }
}
