use super::{Form, FormMsg};
use form_validation::ValidationErrors;
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    rc::Rc,
};
use yew::ComponentLink;

type FormLink<Key> = ComponentLink<Form<Key>>;
type FieldLinkMap<Key> = HashMap<Key, Rc<dyn FieldLink<Key>>>;

pub trait FieldKey: Clone + PartialEq + Display + Hash + Eq + Debug {}

impl FieldKey for &str {}

pub trait FieldLink<Key: Clone>: Debug {
    fn field_key(&self) -> &Key;
    fn send_message(&self, msg: FieldMsg);
}

pub trait FormField<Key> {
    fn validation_errors(&self) -> &ValidationErrors<Key>;
    fn field_key(&self) -> &Key;
}

#[derive(Copy, Clone)]
pub enum FieldMsg {
    /// Validate the field, sends a [FormMsg::FieldValidationUpdate]
    /// to the [FormFieldLink] upon completion.
    Validate,
    ClearValidationErrors,
}

#[derive(Clone, Debug)]
pub struct FormFieldLink<Key = &'static str>
where
    Key: FieldKey + 'static,
{
    form_link: Rc<RefCell<Option<FormLink<Key>>>>,
    field_links: Rc<RefCell<FieldLinkMap<Key>>>,
}

impl<Key> PartialEq for FormFieldLink<Key>
where
    Key: FieldKey + 'static,
{
    fn eq(&self, other: &FormFieldLink<Key>) -> bool {
        Rc::ptr_eq(&self.form_link, &other.form_link)
            && Rc::ptr_eq(&self.field_links, &other.field_links)
    }
}

impl<Key> FormFieldLink<Key>
where
    Key: FieldKey + 'static,
{
    pub fn new() -> Self {
        Self {
            form_link: Rc::new(RefCell::new(None)),
            field_links: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn registered_fields(&self) -> Vec<Key> {
        self.field_links.borrow().keys().cloned().collect()
    }

    pub fn register_form(&self, link: ComponentLink<Form<Key>>) {
        *self.form_link.borrow_mut() = Some(link);
    }

    pub fn form_is_registered(&self) -> bool {
        self.form_link.borrow().is_some()
    }

    pub fn field_is_registered(&self, key: &Key) -> bool {
        self.field_links.borrow().contains_key(key)
    }

    pub fn register_field(&self, link: Rc<dyn FieldLink<Key>>) {
        self.field_links
            .borrow_mut()
            .insert(link.field_key().clone(), link);
    }

    pub fn send_field_message(&self, key: &Key, msg: FieldMsg) {
        self.field_links
            .borrow()
            .get(key)
            .unwrap_or_else(|| {
                panic!(
                    "expected there to be a FieldLink matching the FieldKey {0:?}",
                    key
                )
            })
            .send_message(msg);
    }

    pub fn send_all_fields_message(&self, msg: FieldMsg) {
        for field in self.field_links.borrow().values() {
            field.send_message(msg);
        }
    }

    pub fn send_form_message(&self, msg: FormMsg<Key>) {
        self.form_link
            .borrow()
            .as_ref()
            .expect("expected ComponentLink<Form> to be registered")
            .send_message(msg);
    }
}

impl<Key> Default for FormFieldLink<Key>
where
    Key: FieldKey + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
