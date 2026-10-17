use async_trait::async_trait;
use crate::errors::DomainError;
use crate::models::broker_spec::BrokerSymbolSpec;
use crate::models::Symbol;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait BrokerSpecPort: Send + Sync {
    /// Simpan spesifikasi simbol broker ke storage
    async fn save_broker_spec(&self, spec: &BrokerSymbolSpec) -> Result<(), DomainError>;

    /// Ambil spesifikasi simbol broker berdasarkan simbol
    async fn get_broker_spec(&self, symbol: &Symbol) -> Result<Option<BrokerSymbolSpec>, DomainError>;

    /// Daftar seluruh spesifikasi simbol broker yang terdaftar
    async fn list_broker_specs(&self) -> Result<Vec<BrokerSymbolSpec>, DomainError>;
}
