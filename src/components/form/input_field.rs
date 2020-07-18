use crate::components::form::field::FieldKey;

use form_validation::{Validatable, Validation, ValidationErrors, Validator};
use yew::{html, Callback, ChangeData, Component, ComponentLink, Html, Properties, ShouldRender};

use super::{
    field::{FieldLink, FieldMsg, FormField},
    form::{self, FormFieldLink},
};
use form::FormMsg;
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    rc::Rc,
};

#[derive(Debug, Clone)]
pub enum InputValue {
    String(String),
}

impl InputValue {
    pub fn as_string(&self) -> &String {
        match self {
            InputValue::String(value) => &value,
        }
    }

    pub fn into_string(self) -> String {
        match self {
            InputValue::String(value) => value,
        }
    }
}

impl Display for InputValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputValue::String(value) => Display::fmt(&value, f),
        }
    }
}

#[derive(Debug)]
pub struct InputField<Key>
where
    Key: FieldKey + 'static,
{
    value: InputValue,
    validation_errors: ValidationErrors<Key>,
    props: Props<Key>,
    form_link: FormFieldLink<Key>,
    link: ComponentLink<Self>,
}

pub enum Msg {
    Update(InputValue),
    Validate,
}

pub struct InputFieldLink<Key>
where
    Key: FieldKey + 'static,
{
    pub field_key: Key,
    pub link: ComponentLink<InputField<Key>>,
}

impl<Key> Debug for InputFieldLink<Key>
where
    Key: FieldKey + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SelectFieldLink<{0:?}>", self.field_key())
    }
}

impl Into<Msg> for FieldMsg {
    fn into(self) -> Msg {
        match self {
            FieldMsg::Validate => Msg::Validate,
        }
    }
}

impl<Key> FieldLink<Key> for InputFieldLink<Key>
where
    Key: FieldKey + 'static,
{
    fn field_key(&self) -> &Key {
        &self.field_key
    }
    fn send_message(&self, msg: FieldMsg) {
        self.link.send_message(msg)
    }
}

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props<Key>
where
    Key: FieldKey + 'static,
{
    pub field_key: Key,
    pub form_link: FormFieldLink<Key>,
    #[prop_or_default]
    pub label: Option<String>,
    #[prop_or_default]
    pub validator: Validator<InputValue, Key>,
    #[prop_or_default]
    pub onchange: Callback<InputValue>,
    #[prop_or_default]
    pub placeholder: String,
}

impl<Key> Component for InputField<Key>
where
    Key: Clone + PartialEq + Display + FieldKey + Hash + Eq + 'static,
{
    type Message = Msg;
    type Properties = Props<Key>;

    fn create(props: Props<Key>, link: ComponentLink<Self>) -> Self {
        let form_link = props.form_link.clone();

        let field_link = InputFieldLink {
            field_key: props.field_key.clone(),
            link: link.clone(),
        };

        form_link.register_field(Rc::new(field_link));

        InputField {
            value: InputValue::String(String::default()),
            validation_errors: ValidationErrors::default(),
            props,
            form_link,
            link,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::Update(value) => {
                self.value = value.clone();
                self.props.onchange.emit(value);
                self.props
                    .form_link
                    .send_form_message(FormMsg::FieldValueUpdate(self.props.field_key.clone()));
                self.update(Msg::Validate);
                true
            }
            Msg::Validate => {
                self.validation_errors = self.validate_or_empty();
                self.props
                    .form_link
                    .send_form_message(FormMsg::FieldValidationUpdate(
                        self.props.field_key.clone(),
                        self.validation_errors.clone(),
                    ));
                true
            }
        }
    }

    fn view(&self) -> Html {
        let mut classes = vec!["input".to_string()];
        let validation_error =
            if let Some(errors) = self.validation_errors.get(&self.props.field_key) {
                classes.push("is-danger".to_string());
                let error_message = errors.to_string();
                html! {<p class="help is-danger">{ error_message }</p>}
            } else {
                html! {}
            };

        let input_onchange = self.link.callback(move |data: ChangeData| match data {
            ChangeData::Value(value) => Msg::Update(InputValue::String(value)),
            _ => panic!("invalid data type"),
        });

        html! {
            <div class="field">
                {
                    if let Some(label) = self.props.label.as_ref() {
                        html!{
                            <label class="label">{ label }</label>
                        }
                    } else {
                        html!{}
                    }
                }

                <div class="control">
                    <input
                        class=classes
                        value=self.value
                        type="text"
                        placeholder=self.props.placeholder
                        onchange=input_onchange/>
                </div>
                { validation_error }
            </div>
        }
    }

    fn change(&mut self, props: Props<Key>) -> ShouldRender {
        if self.props != props {
            if self.form_link != props.form_link {
                let form_link = props.form_link.clone();

                if !form_link.field_is_registered(&props.field_key) {
                    let field_link = InputFieldLink {
                        field_key: props.field_key.clone(),
                        link: self.link.clone(),
                    };
                    form_link.register_field(Rc::new(field_link));
                }

                self.form_link = form_link;
            }
            self.props = props;
            true
        } else {
            false
        }
    }
}

impl<Key> Validatable<Key> for InputField<Key>
where
    Key: FieldKey,
{
    fn validate(&self) -> Result<(), ValidationErrors<Key>> {
        self.props
            .validator
            .validate_value(&self.value, &self.props.field_key)
    }
}

impl<Key> FormField<Key> for InputField<Key>
where
    Key: FieldKey + 'static,
{
    fn validation_errors(&self) -> &ValidationErrors<Key> {
        &self.validation_errors
    }
    fn field_key(&self) -> &Key {
        &self.props.field_key
    }
}
