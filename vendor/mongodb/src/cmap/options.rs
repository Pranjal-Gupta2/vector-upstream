#[cfg(test)]
use std::cmp::Ordering;
use std::{sync::Arc, time::Duration};

use derivative::Derivative;
#[cfg(test)]
use serde::de::{Deserializer, Error};
use serde::Deserialize;
use typed_builder::TypedBuilder;

use crate::{
    bson_util,
    client::{auth::Credential, options::ServerApi},
    compression::Compressor,
    event::cmap::{CmapEventHandler, ConnectionPoolOptions as EventOptions},
    options::{ClientOptions, DriverInfo, ServerAddress, TlsOptions},
};

/// Contains the options for creating a connection pool.
#[derive(Clone, Default, Deserialize, Derivative)]
#[derivative(Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConnectionPoolOptions {
    /// The application name specified by the user. This is sent to the server as part of the
    /// handshake that each connection makes when it's created.
    pub(crate) app_name: Option<String>,

    /// The connect timeout passed to each underlying TcpStream when attempting to connect to the
    /// server.
    #[serde(skip)]
    pub(crate) connect_timeout: Option<Duration>,

    /// The credential to use for authenticating connections in this pool.
    #[serde(skip)]
    pub(crate) credential: Option<Credential>,

    /// Extra information to append to the driver version in the metadata of the handshake with the
    /// server. This should be used by libraries wrapping the driver, e.g. ODMs.
    #[serde(skip)]
    pub(crate) driver_info: Option<DriverInfo>,

    /// Processes all events generated by the pool.
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    #[serde(skip)]
    pub(crate) cmap_event_handler: Option<Arc<dyn CmapEventHandler>>,

    /// The compressors that the Client is willing to use in the order they are specified
    /// in the configuration.  The Client sends this list of compressors to the server.
    /// The server responds with the intersection of its supported list of compressors.
    #[serde(skip)]
    pub(crate) compressors: Option<Vec<Compressor>>,

    /// Interval between background thread maintenance runs (e.g. ensure minPoolSize).
    #[cfg(test)]
    #[serde(rename = "backgroundThreadIntervalMS")]
    pub(crate) background_thread_interval: Option<BackgroundThreadInterval>,

    /// Connections that have been ready for usage in the pool for longer than `max_idle_time` will
    /// not be used.
    ///
    /// The default is that connections will not be closed due to being idle.
    #[serde(rename = "maxIdleTimeMS")]
    #[serde(default)]
    #[serde(deserialize_with = "bson_util::deserialize_duration_option_from_u64_millis")]
    pub(crate) max_idle_time: Option<Duration>,

    /// The maximum number of connections that the pool can have at a given time. This includes
    /// connections which are currently checked out of the pool.
    ///
    /// The default is 10.
    pub(crate) max_pool_size: Option<u32>,

    /// The minimum number of connections that the pool can have at a given time. This includes
    /// connections which are currently checked out of the pool. If fewer than `min_pool_size`
    /// connections are in the pool, connections will be added to the pool in the background.
    ///
    /// The default is that no minimum is enforced
    pub(crate) min_pool_size: Option<u32>,

    /// Whether to start the pool as "ready" or not.
    /// For tests only.
    #[cfg(test)]
    pub(crate) ready: Option<bool>,

    /// The declared API version
    ///
    /// The default value is to have no declared API version
    pub(crate) server_api: Option<ServerApi>,

    /// The options specifying how a TLS connection should be configured. If `tls_options` is
    /// `None`, then TLS will not be used for the connections.
    ///
    /// The default is not to use TLS for connections.
    #[serde(skip)]
    pub(crate) tls_options: Option<TlsOptions>,

    /// Whether or not the client is connecting to a MongoDB cluster through a load balancer.
    pub(crate) load_balanced: Option<bool>,
}

impl ConnectionPoolOptions {
    pub(crate) fn from_client_options(options: &ClientOptions) -> Self {
        Self {
            app_name: options.app_name.clone(),
            connect_timeout: options.connect_timeout,
            driver_info: options.driver_info.clone(),
            max_idle_time: options.max_idle_time,
            min_pool_size: options.min_pool_size,
            max_pool_size: options.max_pool_size,
            server_api: options.server_api.clone(),
            tls_options: options.tls_options(),
            credential: options.credential.clone(),
            cmap_event_handler: options.cmap_event_handler.clone(),
            compressors: options.compressors.clone(),
            #[cfg(test)]
            background_thread_interval: None,
            #[cfg(test)]
            ready: None,
            load_balanced: options.load_balanced,
        }
    }

    pub(crate) fn to_event_options(&self) -> EventOptions {
        EventOptions {
            max_idle_time: self.max_idle_time,
            min_pool_size: self.min_pool_size,
            max_pool_size: self.max_pool_size,
        }
    }
}

#[cfg(test)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum BackgroundThreadInterval {
    Never,
    Every(Duration),
}

#[cfg(test)]
impl<'de> Deserialize<'de> for BackgroundThreadInterval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = i64::deserialize(deserializer)?;
        Ok(match millis.cmp(&0) {
            Ordering::Less => BackgroundThreadInterval::Never,
            Ordering::Equal => return Err(D::Error::custom("zero is not allowed")),
            Ordering::Greater => {
                BackgroundThreadInterval::Every(Duration::from_millis(millis as u64))
            }
        })
    }
}

/// Options used for constructing a `Connection`.
#[derive(Derivative)]
#[derivative(Debug)]
#[derive(Clone)]
pub(crate) struct ConnectionOptions {
    pub(crate) connect_timeout: Option<Duration>,

    pub(crate) tls_options: Option<TlsOptions>,

    #[derivative(Debug = "ignore")]
    pub(crate) event_handler: Option<Arc<dyn CmapEventHandler>>,
}

impl From<ConnectionPoolOptions> for ConnectionOptions {
    fn from(pool_options: ConnectionPoolOptions) -> Self {
        Self {
            connect_timeout: pool_options.connect_timeout,
            tls_options: pool_options.tls_options,
            event_handler: pool_options.cmap_event_handler,
        }
    }
}

#[derive(Clone, Debug, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct StreamOptions {
    pub(crate) address: ServerAddress,

    #[builder(default)]
    pub(crate) connect_timeout: Option<Duration>,

    #[builder(default)]
    pub(crate) tls_options: Option<TlsOptions>,
}