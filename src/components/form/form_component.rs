use super::{FormFieldLink, FieldKey, FieldMsg};

use form_validation::ValidationErrors;
use yew::{html, Callback, Children, Component, ComponentLink, Html, Properties, ShouldRender};
use std::collections::HashMap;

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
    Cancel,
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
                { self.props.children.clone() }
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