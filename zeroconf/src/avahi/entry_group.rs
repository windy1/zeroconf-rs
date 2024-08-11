//! Rust friendly `AvahiEntryGroup` wrappers/helpers

use std::sync::Arc;

use super::{client::ManagedAvahiClient, string_list::ManagedAvahiStringList};
use crate::avahi::avahi_util;
use crate::ffi::UnwrapMutOrNull;
use crate::Result;
use avahi_sys::{
    avahi_client_errno, avahi_entry_group_add_service_strlst,
    avahi_entry_group_add_service_subtype, avahi_entry_group_commit, avahi_entry_group_free,
    avahi_entry_group_is_empty, avahi_entry_group_new, avahi_entry_group_reset, AvahiClient,
    AvahiEntryGroup, AvahiEntryGroupCallback, AvahiIfIndex, AvahiProtocol, AvahiPublishFlags,
};
use libc::{c_char, c_void};

/// Wraps the `AvahiEntryGroup` type from the raw Avahi bindings.
///
/// This struct allocates a new `*mut AvahiEntryGroup` when `ManagedAvahiEntryGroup::new()` is
/// invoked and calls the Avahi function responsible for freeing the group on `trait Drop`.
#[derive(Debug)]
pub struct ManagedAvahiEntryGroup {
    inner: *mut AvahiEntryGroup,
    _client: Arc<ManagedAvahiClient>,
}

impl ManagedAvahiEntryGroup {
    /// Initializes the underlying `*mut AvahiEntryGroup` and verifies it was created; returning
    /// `Err(String)` if unsuccessful.
    ///
    /// # Safety
    /// This function is unsafe because of the raw pointer dereference.
    pub unsafe fn new(
        ManagedAvahiEntryGroupParams {
            client,
            callback,
            userdata,
        }: ManagedAvahiEntryGroupParams,
    ) -> Result<Self> {
        let inner = avahi_entry_group_new(client.inner, callback, userdata);

        if inner.is_null() {
            let err = avahi_util::get_error(avahi_client_errno(client.inner));
            Err(format!("could not initialize AvahiEntryGroup: {}", err).into())
        } else {
            Ok(Self {
                inner,
                _client: client,
            })
        }
    }

    /// Delegate function for [`avahi_entry_group_is_empty()`].
    ///
    /// [`avahi_entry_group_is_empty()`]: https://avahi.org/doxygen/html/publish_8h.html#af5a78ee1fda6678970536889d459d85c
    ///
    /// # Safety
    /// This function is unsafe because of the call to `avahi_entry_group_is_empty()`.
    pub unsafe fn is_empty(&self) -> bool {
        avahi_entry_group_is_empty(self.inner) != 0
    }

    /// Delegate function for [`avahi_entry_group_add_service()`].
    ///
    /// Also propagates any error returned into a `Result`.
    ///
    /// [`avahi_entry_group_add_service()`]: https://avahi.org/doxygen/html/publish_8h.html#acb05a7d3d23a3b825ca77cb1c7d00ce4
    ///
    /// # Safety
    /// This function is unsafe because of the call to `avahi_entry_group_add_service_strlst()`.
    pub unsafe fn add_service(
        &mut self,
        AddServiceParams {
            interface,
            protocol,
            flags,
            name,
            kind,
            domain,
            host,
            port,
            txt,
        }: AddServiceParams,
    ) -> Result<()> {
        avahi_util::sys_exec(
            || {
                avahi_entry_group_add_service_strlst(
                    self.inner,
                    interface,
                    protocol,
                    flags,
                    name,
                    kind,
                    domain,
                    host,
                    port,
                    txt.map(|t| t.inner()).unwrap_mut_or_null(),
                )
            },
            "could not register service",
        )
    }

    /// Delegate function for [`avahi_entry_group_add_service_subtype()`].
    ///
    /// Also propagates any error returned into a `Result`.
    ///
    /// [`avahi_entry_group_add_service_subtype()`]: https://avahi.org/doxygen/html/publish_8h.html#a93841be69a152d3134b408c25bb4d5d5
    ///
    /// # Safety
    /// This function is unsafe because of the call to `avahi_entry_group_add_service_subtype()`.
    pub unsafe fn add_service_subtype(
        &mut self,
        AddServiceSubtypeParams {
            interface,
            protocol,
            flags,
            name,
            kind,
            domain,
            subtype,
        }: AddServiceSubtypeParams,
    ) -> Result<()> {
        avahi_util::sys_exec(
            || {
                avahi_entry_group_add_service_subtype(
                    self.inner, interface, protocol, flags, name, kind, domain, subtype,
                )
            },
            "could not register service subtype",
        )
    }

    /// Delegate function for [`avahi_entry_group_commit()`].
    ///
    /// Also propagates any error returned into a `Result`.
    ///
    /// [`avahi_entry_group_commit()`]: https://avahi.org/doxygen/html/publish_8h.html#a2375338d23af4281399404758840a2de
    ///
    /// # Safety
    /// This function is unsafe because of the call to `avahi_entry_group_commit()`.
    pub unsafe fn commit(&mut self) -> Result<()> {
        avahi_util::sys_exec(
            || avahi_entry_group_commit(self.inner),
            "could not commit service",
        )
    }

    /// Delegate function for [`avahi_entry_group_reset()`].
    ///
    /// [`avahi_entry_group_reset()`]: https://avahi.org/doxygen/html/publish_8h.html#a1293bbccf878dbeb9916660022bc71b2
    ///
    /// # Safety
    /// This function is unsafe because of the call to `avahi_entry_group_reset()`.
    pub unsafe fn reset(&mut self) {
        avahi_entry_group_reset(self.inner);
    }

    /// Delegate function for [`avahi_entry_group_get_client()`].
    ///
    /// # Safety
    /// This function is unsafe because it returns a raw pointer to
    /// the underlying `AvahiClient`.
    pub unsafe fn get_client(&self) -> *mut AvahiClient {
        avahi_sys::avahi_entry_group_get_client(self.inner)
    }
}

impl Drop for ManagedAvahiEntryGroup {
    fn drop(&mut self) {
        unsafe { avahi_entry_group_free(self.inner) };
    }
}

/// Holds parameters for initializing a new `ManagedAvahiEntryGroup` with
/// `ManagedAvahiEntryGroup::new()`.
///
/// See [`avahi_entry_group_new()`] for more information about these parameters.
///
/// [avahi_entry_group_new()]: https://avahi.org/doxygen/html/publish_8h.html#abb17598f2b6ec3c3f69defdd488d568c
#[derive(Builder, BuilderDelegate)]
pub struct ManagedAvahiEntryGroupParams {
    client: Arc<ManagedAvahiClient>,
    callback: AvahiEntryGroupCallback,
    userdata: *mut c_void,
}

/// Holds parameters for `ManagedAvahiEntryGroup::add_service()`.
///
/// See [`avahi_entry_group_add_service()`] for more information about these parameters.
///
/// [`avahi_entry_group_add_service()`]: https://avahi.org/doxygen/html/publish_8h.html#acb05a7d3d23a3b825ca77cb1c7d00ce4
#[derive(Builder, BuilderDelegate)]
pub struct AddServiceParams<'a> {
    interface: AvahiIfIndex,
    protocol: AvahiProtocol,
    flags: AvahiPublishFlags,
    name: *const c_char,
    kind: *const c_char,
    domain: *const c_char,
    host: *const c_char,
    port: u16,
    txt: Option<&'a ManagedAvahiStringList>,
}

/// Holds parameters for `ManagedAvahiEntryGroup::add_service_subtype()`.
///
/// See [`avahi_entry_group_add_service_subtype()`] for more information about these parameters.
///
/// [`avahi_entry_group_add_service_subtype()`]: https://www.avahi.org/doxygen/html/publish_8h.html#a93841be69a152d3134b408c25bb4d5d5
#[derive(Builder, BuilderDelegate)]
pub struct AddServiceSubtypeParams {
    interface: AvahiIfIndex,
    protocol: AvahiProtocol,
    flags: AvahiPublishFlags,
    name: *const c_char,
    kind: *const c_char,
    domain: *const c_char,
    subtype: *const c_char,
}
