use super::{FieldKey, FieldMsg, FormFieldLink};

use form_validation::ValidationErrors;
use std::collections::HashMap;
use yew::{html, Callback, Children, Component, ComponentLink, Html, Properties, ShouldRender};

#[derive(Debug)]
pub struct Form<Key>
where
    Key: FieldKey + 'static,
{
    validation_errors: HashMap<Key, ValidationErrors<Key>>,
    /// Will be true while waiting all fields to perform their validations
    validating: bool,
    props: FormProps<Key>,
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
}

/// [Properties](yew::Component::Properties) for [Form].
#[derive(Clone, Properties, PartialEq, Debug)]
pub struct FormProps<Key>
where
    Key: FieldKey + 'static,
{
    /// The link between this form and its fields.
    pub field_link: FormFieldLink<Key>,
    /// Fields, buttons and other elements within the form.
    pub children: Children,
    /// Triggered when the form has been requested to submit, returns
    /// errors if the fields in the form currently contain any
    /// validation errors.
    #[prop_or_default]
    pub onsubmit: Callback<Result<(), ValidationErrors<Key>>>,
    #[prop_or_default]
    pub onvalidateupdate: Callback<ValidationErrors<Key>>,
}

impl<Key> Component for Form<Key>
where
    Key: FieldKey + 'static,
{
    type Message = FormMsg<Key>;
    type Properties = FormProps<Key>;

    fn create(props: FormProps<Key>, link: ComponentLink<Self>) -> Self {
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
                let validation_errors = self.validation_errors();
                let result = if validation_errors.is_empty() {
                    Ok(())
                } else {
                    Err(validation_errors)
                };
                self.props.onsubmit.emit(result);
                true
            }
            FormMsg::FieldValidationUpdate(key, errors) => {
                self.validation_errors.insert(key, errors);

                self.props.onvalidateupdate.emit(self.validation_errors());

                if self.validating && self.all_validated() {
                    self.validating = false;
                    self.link.send_message(FormMsg::Submit)
                }
                true
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <>
                { self.props.children.clone() }
            </>
        }
    }

    fn change(&mut self, props: FormProps<Key>) -> ShouldRender {
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
