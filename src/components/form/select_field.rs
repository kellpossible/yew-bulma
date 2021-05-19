use crate::components::form::{FieldKey, FormFieldLink};
use crate::components::select::Select;

use form_validation::{AsyncValidator, ValidationErrors};
use yew::{html, Callback, Html, Properties};

use super::{
    multi_value_field::MultiValueField, multi_value_field::MultiValueFieldMsg,
    multi_value_field::MultiValueFieldProps, multi_value_field::MultiValueFieldRenderer,
    FieldProps,
};
use std::fmt::{Debug, Display};

pub type SelectField<Value, Key> =
    MultiValueField<Value, Key, SelectFieldProps<Value, Key>, SelectFieldRenderer>;

/// [Properties](yew::Component::Properties) for [SelectField].
#[derive(PartialEq, Clone, Properties, Debug)]
pub struct SelectFieldProps<Value, Key>
where
    Key: FieldKey + PartialEq + 'static,
    Value: Clone + PartialEq,
{
    /// The key used to refer to this field.
    pub field_key: Key,
    /// The link to the form that this field belongs to.
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
    Key: FieldKey + PartialEq + 'static,
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

impl<Value, Key> MultiValueFieldProps<Value, Key> for SelectFieldProps<Value, Key>
where
    Key: FieldKey + PartialEq + 'static,
    Value: Clone + PartialEq,
{
    fn options<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Value> + 'a> {
        Box::new(self.options.iter())
    }

    fn show_label(&self) -> bool {
        self.show_label
    }

    fn label(&self) -> &Option<String> {
        &self.label
    }

    fn validator(&self) -> &AsyncValidator<Option<Value>, Key> {
        &self.validator
    }

    fn selected(&self) -> &Option<Value> {
        &self.selected
    }

    fn onupdate(&self) -> &Callback<Value> {
        &self.onupdate
    }
}

pub struct SelectFieldRenderer;

impl<Value, Key> MultiValueFieldRenderer<Value, Key, SelectFieldProps<Value, Key>>
    for SelectFieldRenderer
where
    Value: Clone + PartialEq + Display + Debug + 'static,
    Key: FieldKey + PartialEq + 'static,
{
    fn render(field: &MultiValueField<Value, Key, SelectFieldProps<Value, Key>, Self>) -> Html {
        let mut classes = vec![];

        let validation_error =
            if let Some(errors) = field.display_validation_errors.get(&field.props.field_key) {
                classes.push("is-danger".to_string());
                let error_message = errors.to_string();
                html! {<p class="help is-danger">{ error_message }</p>}
            } else {
                html! {}
            };

        let select_onchange = field.link.callback(MultiValueFieldMsg::Update);

        let label = field.label();

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
                        selected=field.value.clone()
                        options=field.props.options.clone()
                        div_classes=classes
                        onchange=select_onchange
                        />
                </div>
                { validation_error }
            </div>
        }
    }
}
