pub mod connector;
pub mod ctrader;
pub mod messages;
pub mod validator;

#[cfg(test)]
mod tests;

// Re-export public symbols for 100% backward compatibility
pub use connector::{BrokerConnector, CandleBufferMap};
pub use ctrader::CtraderOpenApiConnector;
pub use messages::{Mt5SocketMessage, MAX_CANDLE_CAPACITY};
pub use validator::DataIntegrityValidator;
