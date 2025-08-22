pub mod api;
pub mod discovery;
pub mod models;
pub mod error;

pub use api::ElgatoClient;
pub use discovery::discover_lights;
pub use models::{LightState, LightInfo, Settings, AccessoryInfo};
pub use error::{ElgatoError, Result};