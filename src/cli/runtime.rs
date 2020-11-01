// LNP Node: node running lightning network protocol and generalized lightning
// channels.
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use core::convert::TryInto;
use std::thread::sleep;
use std::time::Duration;

use lnpbp::lnp::ZmqType;
use lnpbp_services::esb;

use crate::rpc::request::OptionDetails;
use crate::rpc::{Request, ServiceBus};
use crate::{Config, Error, LogStyle, ServiceId};

pub struct Runtime {
    identity: ServiceId,
    esb: esb::Controller<ServiceBus, Request, Handler>,
}

impl Runtime {
    pub fn with(config: Config) -> Result<Self, Error> {
        debug!("Setting up RPC client...");
        let identity = ServiceId::client();
        let bus_config = esb::BusConfig::with_locator(
            config
                .ctl_endpoint
                .try_into()
                .expect("Only ZMQ RPC is currently supported"),
            Some(ServiceId::router()),
        );
        let esb = esb::Controller::with(
            map! {
                ServiceBus::Ctl => bus_config
            },
            Handler {
                identity: identity.clone(),
            },
            ZmqType::RouterConnect,
        )?;

        // We have to sleep in order for ZMQ to bootstrap
        sleep(Duration::from_secs_f32(0.1));

        Ok(Self { identity, esb })
    }

    pub fn identity(&self) -> ServiceId {
        self.identity.clone()
    }

    pub fn request(
        &mut self,
        daemon: ServiceId,
        req: Request,
    ) -> Result<(), Error> {
        debug!("Executing {}", req);
        self.esb.send_to(ServiceBus::Ctl, daemon, req)?;
        Ok(())
    }

    pub fn report_response(&mut self) -> Result<(), Error> {
        for (_, _, rep) in self.esb.recv_poll()? {
            match rep {
                Request::Failure(fail) => {
                    eprintln!(
                        "{}: {}",
                        "Request failure".err(),
                        fail.err_details()
                    );
                    Err(Error::from(fail))?
                }
                resp => println!("{:#}", resp),
            }
        }
        Ok(())
    }

    pub fn report_progress(&mut self) -> Result<usize, Error> {
        let mut counter = 0;
        let mut finished = false;
        while !finished {
            finished = true;
            for (_, _, rep) in self.esb.recv_poll()? {
                counter += 1;
                match rep {
                    Request::Failure(fail) => {
                        eprintln!(
                            "{}: {}",
                            "Request failure".err(),
                            fail.err_details()
                        );
                        Err(Error::from(fail))?
                    }
                    Request::Progress(info) => {
                        println!("{}", info.progress());
                        finished = false;
                    }
                    Request::Success(OptionDetails(Some(info))) => {
                        println!("{}{}", "Success: ".ended(), info.ender());
                    }
                    Request::Success(OptionDetails(None)) => {
                        println!("{}", "Success".ended());
                    }
                    other => {
                        eprintln!(
                            "{}: {}",
                            "Unexpected report".err(),
                            other.err_details()
                        );
                        Err(Error::Other(s!("Unexpected server response")))?
                    }
                }
            }
        }
        Ok(counter)
    }
}

pub struct Handler {
    identity: ServiceId,
}

impl esb::Handler<ServiceBus> for Handler {
    type Request = Request;
    type Address = ServiceId;
    type Error = Error;

    fn identity(&self) -> ServiceId {
        self.identity.clone()
    }

    fn handle(
        &mut self,
        _senders: &mut esb::SenderList<ServiceBus, ServiceId>,
        _bus: ServiceBus,
        _addr: ServiceId,
        _request: Request,
    ) -> Result<(), Error> {
        // Cli does not receive replies for now
        Ok(())
    }

    fn handle_err(&mut self, err: esb::Error) -> Result<(), esb::Error> {
        // We simply propagate the error since it's already being reported
        Err(err)?
    }
}
