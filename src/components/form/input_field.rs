use crate::components::form::{FieldKey, FormMsg};

use form_validation::{AsyncValidatable, AsyncValidator, ValidationErrors};
use yew::{
    html, Callback, ChangeData, Component, ComponentLink, Html, InputData, Properties, ShouldRender,
};
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
        match &self.props.label {
            Label::FieldKey => Some(self.props.field_key.to_string()),
            Label::Text(text) => Some(text.clone()),
            Label::None => None
        }
    }
}

pub enum InputFieldMsg<Key, Value> {
    /// Update the value in the field.
    Update(Value, UpdateSource),
    /// Validate this field, sends a [FormMsg::FieldValidationUpdate]
    /// to the `form_link` upon completion.
    Validate,
    SetValidationErrors(ValidationErrors<Key>),
    ClearValidationErrors,
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
        write!(f, "InputFieldLink<{0:?}>", self.field_key())
    }
}

impl<Type, Key> Into<InputFieldMsg<Type, Key>> for FieldMsg {
    fn into(self) -> InputFieldMsg<Type, Key> {
        match self {
            FieldMsg::Validate => InputFieldMsg::Validate,
            FieldMsg::ClearValidationErrors => InputFieldMsg::ClearValidationErrors,
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

/// Where an update came from.
pub enum UpdateSource {
    /// The update came from an onchange event.
    ChangeEvent,
    /// The update came from an oninput event.
    InputEvent,
}

/// See [InputFieldProps::update_on].
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum UpdateOn {
    /// Update and validate (depending also on
    /// [InputFieldProps::validate_on]) when `onchange` for the field
    /// fires. This happens when a change is committed. See [change
    /// event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/change_event)
    /// for more details.
    ChangeEvent,
    /// Update and validate (depending also on
    /// [InputFieldProps::validate_on]) with `oninput` for the field
    /// (and also with onchange). This happens when the text changes
    /// as the user types. See [input
    /// event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/input_event)
    /// for more details. Also update and validate when the
    /// `onchange` for the field fires. This happens when a change is
    /// committed. See [change
    /// event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/change_event)
    /// for more details.
    InputAndChangeEvent,
}

/// See [InputFieldProps::validate_on].
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum ValidateOn {
    /// Validate when an update fires (determined by
    /// [InputFieldProps::update_on]) and this update was triggered by
    /// an `onchange` event for the field. This happens when a change
    /// is committed. See [change
    /// event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/change_event)
    /// for more details.
    ChangeEvent,
    /// Validate when an update fires (determined by
    /// [InputFieldProps::update_on]), regardless of the event that
    /// triggered the update.
    AnyEvent,
    /// Don't update the validations for any events.
    None,
}

#[derive(PartialEq, Clone, Debug)]
pub enum LabelStyle {
    Above,
}

/// How to produce the label text.
#[derive(PartialEq, Clone, Debug)]
pub enum Label {
    /// Use `field_key`'s `Display` implementation for the label text.
    FieldKey,
    /// Use the supplied string for the label text.
    Text(String),
    /// Display no label.
    None
}

impl Default for Label {
    fn default() -> Self {
        Self::FieldKey
    }
}

impl Default for LabelStyle {
    fn default() -> Self {
        Self::Above
    }
}

/// How to produce the placeholder text.
#[derive(Debug, Clone, PartialEq)]
pub enum Placeholder {
    /// Use `field_key`'s `Display` implementation for the placeholder text.
    FieldKey,
    /// Use the supplied string for the placeholder text.
    Text(String),
    /// Display no placeholder text.
    None
}

impl Default for Placeholder {
    fn default() -> Self {
        Self::None
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
    /// (Optional) Set the label text. By default this is
    /// [Label::FieldKey].
    #[prop_or_default]
    pub label: Label,
    /// How to display the label. By default this is
    /// [LabelStyle::Above].
    #[prop_or_default]
    pub label_style: LabelStyle,
    /// (Optional) What validator to use for this field.
    #[prop_or_default]
    pub validator: AsyncValidator<Value, Key>,
    /// (Optional) Choose which event will cause the field to be
    /// updated, and validated (depending also on
    /// [InputFieldProps::validate_on]). This is
    /// [UpdateOn::ChangeEvent] by default. Using
    /// [UpdateOn::InputAndChangeEvent] will incurr a higher
    /// performance cost, but will react immediately to the user's
    /// input.
    #[prop_or(UpdateOn::ChangeEvent)]
    pub update_on: UpdateOn,
    /// (Optional) When responding to an update, choose which event
    /// types will trigger a validation. By default any event will
    /// trigger a validation on update. See [ValidateOn::AnyEvent].
    ///
    /// You may chose [ValidateOn::None] if you don't want any
    /// validations to occur.
    ///
    /// You may choose to change this option if you want to recieve
    /// updates via [InputFieldProps::onupdate] as the user types, but
    /// you don't want validations to occur you don't want until the
    /// field's `onchange` event callback fires.
    #[prop_or(ValidateOn::AnyEvent)]
    pub validate_on: ValidateOn,
    /// (Optional) A callback for when the contents of this field
    /// changes as a result of an update (determined by
    /// [InputFieldProps::update_on]).
    #[prop_or_default]
    pub onupdate: Callback<Value>,
    /// (Optional) Placeholder text. By default this is
    /// [Placeholder::None].
    #[prop_or_default]
    pub placeholder: Placeholder,
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
            InputFieldMsg::Update(value, source) => {
                let changed = value != self.value;

                if changed {
                    self.value = value.clone();
                    self.props.onupdate.emit(value);

                    self.form_link
                        .send_form_message(FormMsg::FieldValueUpdate(self.props.field_key.clone()));

                    match self.props.validate_on {
                        ValidateOn::ChangeEvent => {
                            if let UpdateSource::ChangeEvent = source {
                                self.update(InputFieldMsg::Validate);
                            }
                        }
                        ValidateOn::AnyEvent => {
                            self.update(InputFieldMsg::Validate);
                        }
                        ValidateOn::None => {}
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
            InputFieldMsg::ClearValidationErrors => {
                self.validation_errors = ValidationErrors::default();
                self.display_validation_errors = self.props.extra_errors.clone();

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

        let input_oninput = match self.props.update_on {
            UpdateOn::ChangeEvent => Callback::default(),
            UpdateOn::InputAndChangeEvent => self.link.callback(move |data: InputData| {
                InputFieldMsg::Update(
                    Type::value_from_html_value(&data.value),
                    UpdateSource::InputEvent,
                )
            }),
        };

        let input_onchange = self.link.callback(move |data: ChangeData| match data {
            ChangeData::Value(value) => InputFieldMsg::Update(
                Type::value_from_html_value(&value),
                UpdateSource::ChangeEvent,
            ),
            _ => panic!("invalid data type"),
        });

        let label = self.label();

        let placeholder = match &self.props.placeholder {
            Placeholder::FieldKey => self.props.field_key.to_string(),
            Placeholder::Text(text) => text.clone(),
            Placeholder::None => String::new(),
        };

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
                        placeholder=placeholder
                        oninput=input_oninput
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
