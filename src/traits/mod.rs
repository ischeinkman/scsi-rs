
mod buffer;
pub use self::buffer::*;


use AumsError;

pub trait CommunicationChannel {
    fn out_transfer<B : Buffer>(&mut self, bytes : &B) -> Result<usize, AumsError>;
    fn in_transfer<B : Buffer>(&mut self, buffer : &mut B) -> Result<usize, AumsError>;
}