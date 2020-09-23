// TODO: reduce code duplication with select_field and share multi_value module.

use crate::components::form::{
    FieldKey, FieldLink, FieldMsg, FormField, FormFieldLink, FormMsg, NeqAssignFieldProps,
};

use form_validation::{AsyncValidatable, AsyncValidator, ValidationErrors};
use yew::{Callback, Component, ComponentLink, Html, Properties, ShouldRender};

use super::FieldProps;
use std::{
    fmt::{Debug, Display},
    future::Future,
    pin::Pin,
    rc::Rc,
};
use yewtil::future::LinkFuture;

#[derive(Debug)]
pub struct MultiValueField<Value, Key, Props, Renderer>
where
    Value: Clone + PartialEq + Display + Debug + 'static,
    Key: FieldKey + 'static,
    Props: MultiValueFieldProps<Value, Key> + 'static,
    Renderer: MultiValueFieldRenderer<Value, Key, Props> + ?Sized + 'static,
{
    pub value: Option<Value>,
    pub validation_errors: ValidationErrors<Key>,
    pub display_validation_errors: ValidationErrors<Key>,
    pub props: Props,
    pub form_link: FormFieldLink<Key>,
    pub link: ComponentLink<Self>,
}

pub trait MultiValueFieldProps<Value, Key>: Properties + FieldProps<Key> + PartialEq where Key: FieldKey {
    /// The options available to select with this field.
    fn options<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Value> + 'a>;
    /// Whether to show the label. By default this is `true`. By
    /// default the label text comes fom the `field_key`'s `Display`
    /// implementation, however it can be overriden with the `label`
    /// property.
    fn show_label(&self) -> bool;
    /// If present, this overrides the default label. Only displays if
    /// `show_label` is `true` (which it is by default).
    fn label(&self) -> &Option<String>;
    /// The validator in use for this field.
    fn validator(&self) -> &AsyncValidator<Option<Value>, Key>;
    /// The default selected value.
    fn selected(&self) -> &Option<Value>;
    /// A callback for when this field changes.
    fn onupdate(&self) -> &Callback<Value>;
}

impl<Value, Key, Props, Renderer> MultiValueField<Value, Key, Props, Renderer>
where
    Value: Clone + PartialEq + Display + Debug + 'static,
    Key: FieldKey + 'static,
    Props: MultiValueFieldProps<Value, Key> + 'static,
    Renderer: MultiValueFieldRenderer<Value, Key, Props>
{
    pub fn label(&self) -> Option<String> {
        if self.props.show_label() {
            match &self.props.label() {
                Some(label) => Some(label.clone()),
                None => Some(self.props.field_key().to_string()),
            }
        } else {
            None
        }
    }
}

pub enum MultiValueFieldMsg<Value, Key> {
    Update(Value),
    Validate,
    ValidationErrors(ValidationErrors<Key>),
    ClearValidationErrors,
}

pub struct MultiValueFieldLink<Value, Key, Props, Renderer>
where
    Value: Clone + PartialEq + Display + Debug + 'static,
    Key: FieldKey + 'static,
    Props: MultiValueFieldProps<Value, Key> + 'static,
    Renderer: MultiValueFieldRenderer<Value, Key, Props> + ?Sized + 'static,
{
    pub field_key: Key,
    pub link: ComponentLink<MultiValueField<Value, Key, Props, Renderer>>,
}

impl<Value, Key, Props, Renderer> Debug for MultiValueFieldLink<Value, Key, Props, Renderer>
where
    Key: FieldKey + 'static,
    Value: Clone + PartialEq + Display + Debug + 'static,
    Props: MultiValueFieldProps<Value, Key> + 'static,
    Renderer: MultiValueFieldRenderer<Value, Key, Props> + ?Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MultiValueFieldLink<{0:?}>", self.field_key())
    }
}

impl<Value, Key> Into<MultiValueFieldMsg<Value, Key>> for FieldMsg {
    fn into(self) -> MultiValueFieldMsg<Value, Key> {
        match self {
            FieldMsg::Validate => MultiValueFieldMsg::Validate,
            FieldMsg::ClearValidationErrors => MultiValueFieldMsg::ClearValidationErrors,
        }
    }
}

impl<Value, Key, Props, Renderer> FieldLink<Key> for MultiValueFieldLink<Value, Key, Props, Renderer>
where
    Value: Clone + PartialEq + Display + Debug + 'static,
    Key: FieldKey + 'static,
    Props: MultiValueFieldProps<Value, Key> + Properties + FieldProps<Key> + 'static,
    Renderer: MultiValueFieldRenderer<Value, Key, Props> + ?Sized,
{
    fn field_key(&self) -> &Key {
        &self.field_key
    }
    fn send_message(&self, msg: FieldMsg) {
        self.link.send_message(msg)
    }
}

pub trait MultiValueFieldRenderer<Value, Key, Props>
where 
    Value: Clone + PartialEq + ToString + Display + Debug,
    Key: FieldKey,
    Props: MultiValueFieldProps<Value, Key> + 'static, {
    fn render(field: &MultiValueField<Value, Key, Props, Self>) -> Html;
}

impl<Value, Key, Props, Renderer> Component for MultiValueField<Value, Key, Props, Renderer>
where
    Value: Clone + PartialEq + ToString + Display + Debug + 'static,
    Key: FieldKey + 'static,
    Props: MultiValueFieldProps<Value, Key> + 'static,
    Renderer: MultiValueFieldRenderer<Value, Key, Props> + ?Sized + 'static,
{
    type Message = MultiValueFieldMsg<Value, Key>;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        let form_link = props.form_link().clone();

        let field_link = MultiValueFieldLink {
            field_key: props.field_key().clone(),
            link: link.clone(),
        };
        form_link.register_field(Rc::new(field_link));

        MultiValueField {
            value: props.selected().clone(),
            validation_errors: ValidationErrors::default(),
            display_validation_errors: props.extra_errors().clone(),
            props,
            form_link,
            link,
        }
    }

    fn update(&mut self, msg: MultiValueFieldMsg<Value, Key>) -> ShouldRender {
        match msg {
            MultiValueFieldMsg::Update(value) => {
                self.value = Some(value.clone());
                self.props.onupdate().emit(value);
                self.props
                    .form_link()
                    .send_form_message(FormMsg::FieldValueUpdate(self.props.field_key().clone()));
                self.update(MultiValueFieldMsg::Validate);
                true
            }
            MultiValueFieldMsg::Validate => {
                let validate_future = self.validate_future_or_empty();
                self.link.send_future(async move {
                    let validation_errors = validate_future.await;

                    MultiValueFieldMsg::ValidationErrors(validation_errors)
                });
                false
            }
            MultiValueFieldMsg::ValidationErrors(errors) => {
                self.validation_errors = errors.clone();

                let mut display_errors = errors;
                display_errors.extend(self.props.extra_errors().clone());
                self.display_validation_errors = display_errors;

                self.form_link
                    .send_form_message(FormMsg::FieldValidationUpdate(
                        self.props.field_key().clone(),
                        self.validation_errors.clone(),
                    ));
                true
            }
            MultiValueFieldMsg::ClearValidationErrors => {
                self.validation_errors = ValidationErrors::default();
                self.display_validation_errors = self.props.extra_errors().clone();

                self.form_link
                    .send_form_message(FormMsg::FieldValidationUpdate(
                        self.props.field_key().clone(),
                        self.validation_errors.clone(),
                    ));
                true
            }
        }
    }

    fn view(&self) -> Html {
        Renderer::render(self)
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        let link = self.link.clone();

        self.props.neq_assign_field(props, move |new_props| {
            Rc::new(MultiValueFieldLink {
                field_key: new_props.field_key().clone(),
                link: link.clone(),
            })
        })
    }
}

impl<Value, Key, Props, Renderer> AsyncValidatable<Key> for MultiValueField<Value, Key, Props, Renderer>
where
    Key: FieldKey,
    Value: Clone + PartialEq + Display + Debug,
    Props: MultiValueFieldProps<Value, Key>,
    Renderer: MultiValueFieldRenderer<Value, Key, Props> + ?Sized,
{
    fn validate_future(&self) -> Pin<Box<dyn Future<Output = Result<(), ValidationErrors<Key>>>>> {
        let value = self.value.clone();
        let field_key = self.props.field_key().clone();
        let validator = self.props.validator().clone();
        Box::pin(async move { validator.validate_value(&value, &field_key).await })
    }
}

impl<Value, Key, Props, Renderer> FormField<Key> for MultiValueField<Value, Key, Props, Renderer>
where
    Key: FieldKey + 'static,
    Value: Clone + PartialEq + Display + Debug,
    Props: MultiValueFieldProps<Value, Key>,
    Renderer: MultiValueFieldRenderer<Value, Key, Props>,
{
    fn validation_errors(&self) -> &ValidationErrors<Key> {
        &self.validation_errors
    }
    fn field_key(&self) -> &Key {
        &self.props.field_key()
    }
}
