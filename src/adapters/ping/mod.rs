//! A simple adapter for testing ping / pong

use foxbox_taxonomy::api::{Error, InternalError, User};
use foxbox_taxonomy::channel::*;
use foxbox_taxonomy::manager::*;
use foxbox_taxonomy::services::*;
use foxbox_taxonomy::values::{format, Value};

use std::sync::Arc;

static ADAPTER_NAME: &'static str = "Ping/Pong adapter (built-in)";
static ADAPTER_VENDOR: &'static str = "team@link.mozilla.org";
static ADAPTER_VERSION: [u32; 4] = [0, 0, 0, 1];

pub struct PingPong {
  getter_ping_id: Id<Channel>,
}

impl PingPong {
  pub fn id() -> Id<AdapterId> {
    Id::new("pingpong@link.mozilla.org")
  }

  pub fn service_pingpong_id() -> Id<ServiceId> {
    Id::new("service:pingpong@link.mozilla.org")
  }

  pub fn getter_ping_id() -> Id<Channel> {
    Id::new("getter:pingpong@link.mozilla.org")
  }
}

impl Adapter for PingPong {
  fn id(&self) -> Id<AdapterId> {
      Self::id()
  }

  fn name(&self) -> &str {
      ADAPTER_NAME
  }

  fn vendor(&self) -> &str {
      ADAPTER_VENDOR
  }

  fn version(&self) -> &[u32; 4] {
      &ADAPTER_VERSION
  }

  fn fetch_values(&self,
                  mut set: Vec<Id<Channel>>,
                  _: User)
                  -> ResultMap<Id<Channel>, Option<Value>, Error> {
      set.drain(..)
          .map(|id| {
                if id == self.getter_ping_id {
                    (id, Ok(Some(Value::new("pong".to_owned()))))
                } else {
                    (id.clone(), Err(Error::Internal(InternalError::NoSuchChannel(id))))
                }
            })
          .collect()
  }
}

impl PingPong {
  pub fn init(adapt: &Arc<AdapterManager>) -> Result<(), Error> {
    let service_pingpong_id = PingPong::service_pingpong_id();
    let getter_ping_id = PingPong::getter_ping_id();
    let adapter_id = PingPong::id();
    let pingpong = Arc::new(PingPong {getter_ping_id: getter_ping_id.clone() });
    try!(adapt.add_adapter(pingpong));
    let mut service = Service::empty(&service_pingpong_id, &adapter_id);
    service.properties.insert("model".to_owned(), "Mozilla ping/pong v1".to_owned());
    try!(adapt.add_service(service));
    try!(adapt.add_channel(Channel {
      id: getter_ping_id,
      service: service_pingpong_id.clone(),
      adapter: adapter_id.clone(),
      feature: Id::new("pingpong/ping"),
      supports_fetch: Some(Signature::returns(Maybe::Required(format::STRING.clone()))),
      ..Channel::default()
    }));
    Ok(())
  }
}