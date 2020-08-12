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
    display_validation_errors: ValidationErrors<Key>,
    props: SelectFieldProps<Value, Key>,
    form_link: FormFieldLink<Key>,
    link: ComponentLink<Self>,
}

impl<Value, Key> SelectField<Value, Key>
where
    Value: Clone + PartialEq + Display + Debug + 'static,
    Key: FieldKey + 'static,
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

pub enum SelectFieldMsg<Value, Key> {
    Update(Value),
    Validate,
    ValidationErrors(ValidationErrors<Key>),
    ClearValidationErrors,
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
            FieldMsg::ClearValidationErrors => SelectFieldMsg::ClearValidationErrors,
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
    /// The key used to refer to this field.
    pub field_key: Key,
    pub form_link: FormFieldLink<Key>,
    /// The options available to this select field.
    pub options: Vec<Value>,
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
    /// (Optional) The default selected value.
    #[prop_or_default]
    pub selected: Option<Value>,
    /// (Optional) What validator to use for this field.
    #[prop_or_default]
    pub validator: AsyncValidator<Option<Value>, Key>,
    /// (Optional) A callback for when this field changes.
    #[prop_or_default]
    pub onupdate: Callback<Value>,
    /// (Optional) Whether to validate when the field is updated.
    #[prop_or(true)]
    pub validate_on_update: bool,
    /// (Optional) Extra validation errors to display. These errors
    /// are not reported to the `Form`.
    #[prop_or_default]
    pub extra_errors: ValidationErrors<Key>,
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
    fn extra_errors(&self) -> &ValidationErrors<Key> {
        &self.extra_errors
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
            display_validation_errors: props.extra_errors.clone(),
            props,
            form_link,
            link,
        }
    }

    fn update(&mut self, msg: SelectFieldMsg<Value, Key>) -> ShouldRender {
        match msg {
            SelectFieldMsg::Update(value) => {
                self.value = Some(value.clone());
                self.props.onupdate.emit(value);
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
            SelectFieldMsg::ValidationErrors(errors) => {
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
            SelectFieldMsg::ClearValidationErrors => {
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

    fn view(&self) -> Html {
        let mut classes = vec![];

        let validation_error =
            if let Some(errors) = self.display_validation_errors.get(&self.props.field_key) {
                classes.push("is-danger".to_string());
                let error_message = errors.to_string();
                html! {<p class="help is-danger">{ error_message }</p>}
            } else {
                html! {}
            };

        let select_onchange = self.link.callback(SelectFieldMsg::Update);

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
