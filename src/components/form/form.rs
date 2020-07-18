use crate::components::form::field::FieldKey;
use form_validation::ValidationErrors;
use yew::html::Renderable;
use yew::{html, Callback, Children, Component, ComponentLink, Html, Properties, ShouldRender};

use super::field::{FieldLink, FieldMsg};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct Form<Key>
where
    Key: FieldKey + 'static,
{
    validation_errors: HashMap<Key, ValidationErrors<Key>>,
    /// Will be true while waiting all fields to perform their validations
    validating: bool,
    props: Props<Key>,
    field_link: FormFieldLink<Key>,
    link: ComponentLink<Self>,
}

impl<Key> Form<Key>
where
    Key: FieldKey + 'static,
{
    pub fn validation_errors(&self) -> ValidationErrors<Key> {
        let mut errors = ValidationErrors::default();
        for errors_for_key in self.validation_errors.values() {
            errors.extend(errors_for_key.clone())
        }
        errors
    }

    pub fn all_validated(&self) -> bool {
        let mut all_validated = true;
        let field_keys = self.props.field_link.registered_fields();

        for key in &field_keys {
            all_validated &= self.validation_errors.contains_key(key)
        }

        all_validated
    }
}

#[derive(Clone)]
pub enum FormMsg<Key> {
    FieldValueUpdate(Key),
    FieldValidationUpdate(Key, ValidationErrors<Key>),
    ValidateThenSubmit,
    Submit,
    Cancel,
}

#[derive(Clone, Properties, PartialEq, Debug)]
pub struct Props<Key>
where
    Key: FieldKey + 'static,
{
    pub field_link: FormFieldLink<Key>,
    pub children: Children,
    #[prop_or_default]
    pub oncancel: Callback<()>,
    #[prop_or_default]
    pub onsubmit: Callback<()>,
    #[prop_or_default]
    pub cancel_button_label: Option<String>,
    #[prop_or_default]
    pub submit_button_label: Option<String>,
}

impl<Key> Component for Form<Key>
where
    Key: FieldKey + 'static,
{
    type Message = FormMsg<Key>;
    type Properties = Props<Key>;

    fn create(props: Props<Key>, link: ComponentLink<Self>) -> Self {
        let field_link = props.field_link.clone();
        field_link.register_form(link.clone());

        Form {
            validation_errors: HashMap::new(),
            validating: false,
            props,
            field_link,
            link,
        }
    }

    fn update(&mut self, msg: FormMsg<Key>) -> ShouldRender {
        match msg {
            FormMsg::FieldValueUpdate(_) => true,
            FormMsg::ValidateThenSubmit => {
                // Clear the errors to ensure that we re-validate all the fields.
                self.validation_errors.clear();
                self.validating = true;

                self.props
                    .field_link
                    .send_all_fields_message(FieldMsg::Validate);

                false
            }
            FormMsg::Submit => {
                if self.validation_errors().is_empty() {
                    self.props.onsubmit.emit(());
                }
                true
            }
            FormMsg::Cancel => {
                self.props.oncancel.emit(());
                true
            }
            FormMsg::FieldValidationUpdate(key, errors) => {
                self.validation_errors.insert(key, errors);

                if self.validating && self.all_validated() {
                    self.validating = false;
                    self.link.send_message(FormMsg::Submit)
                }
                true
            }
        }
    }

    fn view(&self) -> Html {
        let onclick_submit = self.link.callback(|_| FormMsg::ValidateThenSubmit);
        let onclick_cancel = self.link.callback(|_| FormMsg::Cancel);

        let submit_button_label = self
            .props
            .submit_button_label
            .as_ref()
            .map_or("Submit".to_string(), |label| label.clone());
        let cancel_button_label = self
            .props
            .cancel_button_label
            .as_ref()
            .map_or("Cancel".to_string(), |label| label.clone());

        html! {
            <>
                { self.props.children.render() }
                <div class="field is-grouped">
                    <div class="control">
                        <button
                            class="button is-link"
                            onclick=onclick_submit
                            disabled=!self.validation_errors().is_empty()>
                            { submit_button_label }
                        </button>
                    </div>
                    <div class="control">
                        <button class="button is-link is-light" onclick=onclick_cancel>{ cancel_button_label }</button>
                    </div>
                </div>
            </>
        }
    }

    fn change(&mut self, props: Props<Key>) -> ShouldRender {
        if self.props != props {
            if self.field_link != props.field_link {
                let field_link = props.field_link.clone();
                if !field_link.form_is_registered() {
                    field_link.register_form(self.link.clone())
                }
                self.field_link = field_link;
            }

            self.props = props;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub struct FormFieldLink<Key = &'static str>
where
    Key: FieldKey + 'static,
{
    form_link: Rc<RefCell<Option<ComponentLink<Form<Key>>>>>,
    field_links: Rc<RefCell<HashMap<Key, Rc<dyn FieldLink<Key>>>>>,
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
        self.field_links
            .borrow()
            .keys()
            .map(|key| key.clone())
            .collect()
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
            .expect(&format!(
                "expected there to be a FieldLink matching the FieldKey {0:?}",
                key
            ))
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
