// Bitcoin transaction processing & database indexing daemon
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


use std::io;
use std::error::Error;
use tokio::task::JoinError;


#[derive(Debug, Display)]
#[display_from(Debug)]
pub enum BootstrapError {
    IoError(io::Error),
    ArgParseError(String),
    SubscriptionError(zmq::Error),
    PublishingError(zmq::Error),
    MultithreadError(JoinError)
}

impl From<&str> for BootstrapError {
    fn from(err: &str) -> Self {
        BootstrapError::ArgParseError(err.to_string())
    }
}

impl From<String> for BootstrapError {
    fn from(err: String) -> Self {
        BootstrapError::ArgParseError(err)
    }
}

impl From<io::Error> for BootstrapError {
    fn from(err: io::Error) -> Self {
        BootstrapError::IoError(err)
    }
}

impl From<JoinError> for BootstrapError {
    fn from(err: JoinError) -> Self {
        BootstrapError::MultithreadError(err)
    }
}
