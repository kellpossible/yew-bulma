//! A collection of `yew` [Component](yew::Component)s to build user
//! interfaces with the `bulma` styles applied. These componeents
//! produce the correct HTML and attributes to be compatible with
//! `bulma`.

pub mod form;
pub mod icon;
pub mod select;

pub use form::*;
pub use icon::Icon;
pub use select::Select;
