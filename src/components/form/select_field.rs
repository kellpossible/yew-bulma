use crate::components::form::{
    FieldKey, FieldLink, FieldMsg, FormField, FormFieldLink, FormMsg, NeqAssignFieldProps,
};
use crate::components::select::Select;

use form_validation::{AsyncValidatable, AsyncValidator, ValidationErrors};
use yew::{html, Callback, Component, ComponentLink, Html, Properties, ShouldRender};

use super::FieldProps;
use std::{
    fmt::{Debug, Display},
    future::Future,
    pin::Pin,
    rc::Rc,
};
use yewtil::future::LinkFuture;

#[derive(Debug)]
pub struct SelectField<Value, Key>
where
    Value: Clone + PartialEq + Display + Debug + 'static,
    Key: FieldKey + 'static,
{
    value: Option<Value>,
    validation_errors: ValidationErrors<Key>,
    props: SelectFieldProps<Value, Key>,
    form_link: FormFieldLink<Key>,
    link: ComponentLink<Self>,
}

pub enum SelectFieldMsg<Value, Key> {
    Update(Value),
    Validate,
    ValidationErrors(ValidationErrors<Key>),
}

pub struct SelectFieldLink<Value, Key>
where
    Value: Clone + PartialEq + Display + Debug + 'static,
    Key: FieldKey + 'static,
{
    pub field_key: Key,
    pub link: ComponentLink<SelectField<Value, Key>>,
}

impl<Value, Key> Debug for SelectFieldLink<Value, Key>
where
    Key: FieldKey + 'static,
    Value: Clone + PartialEq + Display + Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SelectFieldLink<{0:?}>", self.field_key())
    }
}

impl<Value, Key> Into<SelectFieldMsg<Value, Key>> for FieldMsg {
    fn into(self) -> SelectFieldMsg<Value, Key> {
        match self {
            FieldMsg::Validate => SelectFieldMsg::Validate,
        }
    }
}

impl<Value, Key> FieldLink<Key> for SelectFieldLink<Value, Key>
where
    Value: Clone + PartialEq + Display + Debug + 'static,
    Key: FieldKey + 'static,
{
    fn field_key(&self) -> &Key {
        &self.field_key
    }
    fn send_message(&self, msg: FieldMsg) {
        self.link.send_message(msg)
    }
}

/// [Properties](yew::Component::Properties) for [SelectField].
#[derive(PartialEq, Clone, Properties, Debug)]
pub struct SelectFieldProps<Value, Key>
where
    Key: FieldKey + 'static,
    Value: Clone + PartialEq,
{
    pub field_key: Key,
    pub form_link: FormFieldLink<Key>,
    #[prop_or_default]
    pub label: Option<String>,
    #[prop_or_default]
    pub selected: Option<Value>,
    pub options: Vec<Value>,
    #[prop_or_default]
    pub validator: AsyncValidator<Option<Value>, Key>,
    #[prop_or_default]
    pub onchange: Callback<Value>,
}

impl<Value, Key> FieldProps<Key> for SelectFieldProps<Value, Key>
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
}

impl<Value, Key> Component for SelectField<Value, Key>
where
    Value: Clone + PartialEq + ToString + Display + Debug + 'static,
    Key: FieldKey + 'static,
{
    type Message = SelectFieldMsg<Value, Key>;
    type Properties = SelectFieldProps<Value, Key>;

    fn create(props: SelectFieldProps<Value, Key>, link: ComponentLink<Self>) -> Self {
        let form_link = props.form_link.clone();

        let field_link = SelectFieldLink {
            field_key: props.field_key.clone(),
            link: link.clone(),
        };
        form_link.register_field(Rc::new(field_link));

        SelectField {
            value: props.selected.clone(),
            validation_errors: ValidationErrors::default(),
            props,
            form_link,
            link,
        }
    }

    fn update(&mut self, msg: SelectFieldMsg<Value, Key>) -> ShouldRender {
        match msg {
            SelectFieldMsg::Update(value) => {
                self.value = Some(value.clone());
                self.props.onchange.emit(value);
                self.props
                    .form_link
                    .send_form_message(FormMsg::FieldValueUpdate(self.props.field_key.clone()));
                self.update(SelectFieldMsg::Validate);
                true
            }
            SelectFieldMsg::Validate => {
                let validate_future = self.validate_future_or_empty();
                self.link.send_future(async move {
                    let validation_errors = validate_future.await;

                    SelectFieldMsg::ValidationErrors(validation_errors)
                });
                false
            }
            SelectFieldMsg::ValidationErrors(validation_errors) => {
                self.validation_errors = validation_errors;
                self.form_link
                    .send_form_message(FormMsg::FieldValidationUpdate(
                        self.props.field_key.clone(),
                        self.validation_errors.clone(),
                    ));
                true
            }
        }
    }

    fn view(&self) -> Html {
        let mut classes = vec![];
        let validation_error =
            if let Some(errors) = self.validation_errors.get(&self.props.field_key) {
                classes.push("is-danger".to_string());
                let error_message = errors.to_string();
                html! {<p class="help is-danger">{ error_message }</p>}
            } else {
                html! {}
            };

        let select_onchange = self.link.callback(SelectFieldMsg::Update);

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
                    <Select<Value>
                        selected=self.value.clone()
                        options=self.props.options.clone()
                        div_classes=classes
                        onchange=select_onchange
                        />
                </div>
                { validation_error }
            </div>
        }
    }

    fn change(&mut self, props: SelectFieldProps<Value, Key>) -> ShouldRender {
        let link = self.link.clone();
        self.props.neq_assign_field(props, move |new_props| {
            Rc::new(SelectFieldLink {
                field_key: new_props.field_key().clone(),
                link: link.clone(),
            })
        })
    }
}

impl<Value, Key> AsyncValidatable<Key> for SelectField<Value, Key>
where
    Key: FieldKey,
    Value: Clone + PartialEq + Display + Debug,
{
    fn validate_future(&self) -> Pin<Box<dyn Future<Output = Result<(), ValidationErrors<Key>>>>> {
        let value = self.value.clone();
        let field_key = self.props.field_key.clone();
        let validator = self.props.validator.clone();
        Box::pin(async move { validator.validate_value(&value, &field_key).await })
    }
}

impl<Value, Key> FormField<Key> for SelectField<Value, Key>
where
    Key: FieldKey + 'static,
    Value: Clone + PartialEq + Display + Debug,
{
    fn validation_errors(&self) -> &ValidationErrors<Key> {
        &self.validation_errors
    }
    fn field_key(&self) -> &Key {
        &self.props.field_key
    }
}
