use crate::components::form::{FieldKey, FormMsg};

use form_validation::{AsyncValidatable, AsyncValidator, ValidationErrors};
use yew::{html, Callback, ChangeData, Component, ComponentLink, Html, Properties, ShouldRender};
use yewtil::future::LinkFuture;

use super::{FieldLink, FieldMsg, FieldProps, FormField, FormFieldLink, NeqAssignFieldProps};

use std::{
    fmt::{Debug, Display},
    future::Future,
    hash::Hash,
    pin::Pin,
    rc::Rc,
};

pub trait InputType {
    type Value: Clone + Display + PartialEq;

    fn value_from_html_value(html_value: &str) -> Self::Value;
    fn default_value() -> Self::Value;
    fn input_type() -> &'static str;
}

pub struct TextInputType;
pub type TextInput<Key> = InputField<Key, TextInputType>;

impl InputType for TextInputType {
    type Value = String;

    fn value_from_html_value(html_value: &str) -> Self::Value {
        html_value.to_string()
    }

    fn default_value() -> Self::Value {
        String::default()
    }

    fn input_type() -> &'static str {
        "text"
    }
}

pub struct PasswordInputType;
pub type PasswordInput<Key> = InputField<Key, PasswordInputType>;

impl InputType for PasswordInputType {
    type Value = String;

    fn value_from_html_value(html_value: &str) -> Self::Value {
        html_value.to_string()
    }

    fn default_value() -> Self::Value {
        String::default()
    }

    fn input_type() -> &'static str {
        "password"
    }
}

#[derive(Debug)]
pub struct InputField<Key, Type>
where
    Key: FieldKey + 'static,
    Type: InputType + 'static,
{
    value: Type::Value,
    validation_errors: ValidationErrors<Key>,
    display_validation_errors: ValidationErrors<Key>,
    props: InputFieldProps<Key, Type::Value>,
    form_link: FormFieldLink<Key>,
    link: ComponentLink<Self>,
}

impl<Key, Type> InputField<Key, Type>
where
    Key: FieldKey + 'static,
    Type: InputType + 'static,
{
    fn label(&self) -> Option<String> {
        if self.props.show_label {
            match &self.props.label {
                Some(label) => Some(label.clone()),
                None => Some(self.props.field_key.to_string()),
            }
        } else {
            None
        }
    }
}

pub enum InputFieldMsg<Key, Value> {
    Update(Value),
    /// Validate this field, sends a [FormMsg::FieldValidationUpdate]
    /// to the `form_link` upon completion.
    Validate,
    SetValidationErrors(ValidationErrors<Key>),
}

pub struct InputFieldLink<Key, Type>
where
    Key: FieldKey + 'static,
    Type: InputType + 'static,
{
    pub field_key: Key,
    pub link: ComponentLink<InputField<Key, Type>>,
}

impl<Key, Type> Debug for InputFieldLink<Key, Type>
where
    Key: FieldKey + 'static,
    Type: InputType + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SelectFieldLink<{0:?}>", self.field_key())
    }
}

impl<Type, Key> Into<InputFieldMsg<Type, Key>> for FieldMsg {
    fn into(self) -> InputFieldMsg<Type, Key> {
        match self {
            FieldMsg::Validate => InputFieldMsg::Validate,
        }
    }
}

impl<Key, Type> FieldLink<Key> for InputFieldLink<Key, Type>
where
    Key: FieldKey + 'static,
    Type: InputType + 'static,
{
    fn field_key(&self) -> &Key {
        &self.field_key
    }
    fn send_message(&self, msg: FieldMsg) {
        self.link.send_message(msg)
    }
}

/// [Properties](yew::Component::Properties) for [InputField].
#[derive(PartialEq, Clone, Properties, Debug)]
pub struct InputFieldProps<Key, Value>
where
    Key: FieldKey + 'static,
    Value: Clone + PartialEq,
{
    /// The key used to refer to this field.
    pub field_key: Key,
    /// The link to the form that this field belongs to.
    pub form_link: FormFieldLink<Key>,
    /// Whether to show the label. By default this is `true`. By
    /// default the label text comes fom the `field_key`'s `Display`
    /// implementation, however it can be overriden with the `label`
    /// property.
    #[prop_or(true)]
    pub show_label: bool,
    /// (Optional) Override the default label. Only displays if
    /// `show_label` is `true` (which it is by default).
    #[prop_or_default]
    pub label: Option<String>,
    /// (Optional) What validator to use for this field.
    #[prop_or_default]
    pub validator: AsyncValidator<Value, Key>,
    /// (Optional) A callback for when this field changes.
    #[prop_or_default]
    pub onchange: Callback<Value>,
    /// (Optional) A placeholder string.
    #[prop_or_default]
    pub placeholder: String,
    /// (Optional) Whether to validate when the field is updated.
    #[prop_or(true)]
    pub validate_on_update: bool,
    /// (Optional) Extra validation errors to display. These errors
    /// are not reported to the `Form`.
    #[prop_or_default]
    pub extra_errors: ValidationErrors<Key>,
}

impl<Key, Value> FieldProps<Key> for InputFieldProps<Key, Value>
where
    Key: FieldKey + 'static,
    Value: Clone + PartialEq,
{
    fn form_link(&self) -> &FormFieldLink<Key> {
        &self.form_link
    }
    fn field_key(&self) -> &Key {
        &self.field_key
    }
    fn extra_errors(&self) -> &ValidationErrors<Key> {
        &self.extra_errors
    }
}

impl<Key, Type> Component for InputField<Key, Type>
where
    Key: Clone + PartialEq + Display + FieldKey + Hash + Eq + 'static,
    Type: InputType + 'static,
{
    type Message = InputFieldMsg<Key, Type::Value>;
    type Properties = InputFieldProps<Key, Type::Value>;

    fn create(props: InputFieldProps<Key, Type::Value>, link: ComponentLink<Self>) -> Self {
        let form_link = props.form_link.clone();

        let field_link = InputFieldLink {
            field_key: props.field_key.clone(),
            link: link.clone(),
        };

        form_link.register_field(Rc::new(field_link));

        InputField {
            value: Type::default_value(),
            validation_errors: ValidationErrors::default(),
            display_validation_errors: props.extra_errors.clone(),
            props,
            form_link,
            link,
        }
    }

    fn update(&mut self, msg: InputFieldMsg<Key, Type::Value>) -> ShouldRender {
        match msg {
            InputFieldMsg::Update(value) => {
                let changed = value != self.value;

                if changed {
                    self.value = value.clone();
                    self.props.onchange.emit(value);
                    self.form_link
                        .send_form_message(FormMsg::FieldValueUpdate(self.props.field_key.clone()));

                    if self.props.validate_on_update {
                        self.update(InputFieldMsg::Validate);
                    }
                }

                true
            }
            InputFieldMsg::Validate => {
                let validate_future = self.validate_future_or_empty();
                self.link.send_future(async move {
                    let validation_errors = validate_future.await;

                    InputFieldMsg::SetValidationErrors(validation_errors)
                });
                false
            }
            InputFieldMsg::SetValidationErrors(errors) => {
                self.validation_errors = errors.clone();

                let mut display_errors = errors;
                display_errors.extend(self.props.extra_errors.clone());
                self.display_validation_errors = display_errors;

                self.form_link
                    .send_form_message(FormMsg::FieldValidationUpdate(
                        self.props.field_key.clone(),
                        self.validation_errors.clone(),
                    ));
                true
            }
        }
    }

    fn change(&mut self, props: InputFieldProps<Key, Type::Value>) -> ShouldRender {
        let link = self.link.clone();

        if self.props.extra_errors != props.extra_errors {
            let mut errors = self.validation_errors.clone();
            errors.extend(props.extra_errors.clone());
            self.display_validation_errors = errors;
        }

        self.props.neq_assign_field(props, move |new_props| {
            Rc::new(InputFieldLink {
                field_key: new_props.field_key().clone(),
                link: link.clone(),
            })
        })
    }

    fn view(&self) -> Html {
        let mut classes = vec!["input".to_string()];

        let validation_error =
            if let Some(errors) = self.display_validation_errors.get(&self.props.field_key) {
                classes.push("is-danger".to_string());
                let error_message = errors.to_string();
                html! {<p class="help is-danger">{ error_message }</p>}
            } else {
                html! {}
            };

        let input_onchange = self.link.callback(move |data: ChangeData| match data {
            ChangeData::Value(value) => InputFieldMsg::Update(Type::value_from_html_value(&value)),
            _ => panic!("invalid data type"),
        });

        let label = self.label();

        html! {
            <div class="field">
                {
                    match label {
                        Some(label) => {
                            html!{
                                <label class="label">{ label }</label>
                            }
                        },
                        None => {
                            html!{}
                        }
                    }
                }

                <div class="control">
                    <input
                        class=classes
                        value=self.value
                        type=Type::input_type()
                        placeholder=self.props.placeholder
                        onchange=input_onchange/>
                </div>
                { validation_error }
            </div>
        }
    }
}

impl<Key, Type> AsyncValidatable<Key> for InputField<Key, Type>
where
    Key: FieldKey,
    Type: InputType,
{
    fn validate_future(&self) -> Pin<Box<dyn Future<Output = Result<(), ValidationErrors<Key>>>>> {
        let value = self.value.clone();
        let field_key = self.props.field_key.clone();
        let validator = self.props.validator.clone();
        Box::pin(async move { validator.validate_value(&value, &field_key).await })
    }
}

impl<Key, Type> FormField<Key> for InputField<Key, Type>
where
    Key: FieldKey + 'static,
    Type: InputType + 'static,
{
    fn validation_errors(&self) -> &ValidationErrors<Key> {
        &self.validation_errors
    }

    fn field_key(&self) -> &Key {
        &self.props.field_key
    }
}
