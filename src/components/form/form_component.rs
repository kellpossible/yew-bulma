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
    form_link: FormFieldLink<Key>,
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
        let field_keys = self.props.form_link.registered_fields();

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
    /// Validate all the form fields, and submit (Triggering
    /// `onsubmit` callback) when all fields have completed their
    /// validations.
    ValidateThenSubmit,
    Submit,
    /// An event that will be ignored (to use in callbacks)
    Ignore,
}

/// [Properties](yew::Component::Properties) for [Form].
#[derive(Clone, Properties, PartialEq, Debug)]
pub struct FormProps<Key>
where
    Key: FieldKey + 'static,
{
    /// The link between this form and its fields.
    pub form_link: FormFieldLink<Key>,
    /// Fields, buttons and other elements within the form.
    pub children: Children,
    /// Triggered when the form has been requested to submit, returns
    /// errors if the fields in the form currently contain any
    /// validation errors.
    #[prop_or_default]
    pub onsubmit: Callback<Result<(), ValidationErrors<Key>>>,
    /// Triggered when form receives a [FormMsg::ValidateThenSubmit],
    /// and it has begun validation.
    #[prop_or_default]
    pub onsubmit_validate_start: Callback<()>,
    /// Triggered when elements in this form have been validated.
    #[prop_or_default]
    pub onvalidateupdate: Callback<ValidationErrors<Key>>,
    /// Whether to trigger the onsubmit event/callback when the
    /// internal `<form>`'s submit action is invoked.
    #[prop_or(true)]
    pub form_onsubmit: bool,
}

impl<Key> Component for Form<Key>
where
    Key: FieldKey + 'static,
{
    type Message = FormMsg<Key>;
    type Properties = FormProps<Key>;

    fn create(props: FormProps<Key>, link: ComponentLink<Self>) -> Self {
        let field_link = props.form_link.clone();
        field_link.register_form(link.clone());

        Form {
            validation_errors: HashMap::new(),
            validating: false,
            props,
            form_link: field_link,
            link,
        }
    }

    fn update(&mut self, msg: FormMsg<Key>) -> ShouldRender {
        match msg {
            FormMsg::FieldValueUpdate(_) => true,
            FormMsg::ValidateThenSubmit => {
                self.props.onsubmit_validate_start.emit(());

                // Clear the errors to ensure that we re-validate all the fields.
                self.validation_errors.clear();
                self.validating = true;

                self.props
                    .form_link
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
            FormMsg::Ignore => false,
        }
    }

    fn view(&self) -> Html {
        let form_onsubmit = self.props.form_onsubmit;
        let onsubmit = self.link.callback(move |event: web_sys::FocusEvent| {
            // Prevent the default browser action of refreshing the page!
            event.prevent_default();

            if form_onsubmit {
                FormMsg::ValidateThenSubmit
            } else {
                FormMsg::Ignore
            }
        });

        html! {
            <form onsubmit=onsubmit>
                { self.props.children.clone() }
            </form>
        }
    }

    fn change(&mut self, props: FormProps<Key>) -> ShouldRender {
        if self.props != props {
            if self.form_link != props.form_link {
                let field_link = props.form_link.clone();
                if !field_link.form_is_registered() {
                    field_link.register_form(self.link.clone())
                }
                self.form_link = field_link;
            }

            self.props = props;
            true
        } else {
            false
        }
    }
}
