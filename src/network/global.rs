use super::service::ServiceHandle;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref G_SERVICE_HANDLE: ServiceHandle = ServiceHandle::default();
}
