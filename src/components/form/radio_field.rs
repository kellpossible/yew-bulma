use super::{
    multi_value_field::MultiValueField, multi_value_field::MultiValueFieldMsg,
    multi_value_field::MultiValueFieldProps, multi_value_field::MultiValueFieldRenderer,
    FieldProps,
};

use crate::components::form::{FieldKey, FormFieldLink};

use form_validation::{AsyncValidator, ValidationErrors};
use yew::{html, Callback, ChangeData, Html, Properties};

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

/// This is a rather heavy generic component, for large projects
/// consider using String/&str for both the value and the key in forms
/// that use this component for improved compile times.
pub type RadioField<Value, Key> =
    MultiValueField<Value, Key, RadioFieldProps<Value, Key>, RadioFieldRenderer<Value, Key>>;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Layout {
    /// Uses the following layout:
    ///
    /// ```html
    /// <div>
    ///     <input/>
    ///     <label></label>
    /// </div>
    /// ```
    SideBySideInDiv,
    /// Uses the following layout:
    ///
    /// ```html
    /// <label><input/></label>
    /// ```
    InputInLabel,
}

impl Default for Layout {
    fn default() -> Self {
        Self::InputInLabel
    }
}

/// [Properties](yew::Component::Properties) for [RadioField].
#[derive(PartialEq, Clone, Properties, Debug)]
pub struct RadioFieldProps<Value, Key>
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
    /// (Optional) List of options which should be disabled.
    #[prop_or_default]
    pub disabled_options: Vec<Value>,
    /// Whether to show the field label. By default this is `true`. By
    /// default the label text comes fom the `field_key`'s `Display`
    /// implementation, however it can be overriden with the `label`
    /// property.
    #[prop_or(true)]
    pub show_label: bool,
    /// (Optional) Override the default field label. Only displays if
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
    /// (Optional) Classes to apply to each item's `<label>`. Default:
    /// `["radio"]`.
    #[prop_or(vec!["radio".to_string()])]
    pub input_label_classes: Vec<String>,
    /// (Optional) Classes to apply to each item's `<input/>`.
    #[prop_or_default]
    pub input_classes: Vec<String>,
    /// (Optional) What layout to empoy. Default:
    /// [Layout::InputInLabel].
    #[prop_or_default]
    pub layout: Layout,
    /// (Optional) Classes to apply to each item's `<div>` that
    /// contains both the `<input/>` and the `<label>`. Only
    /// appliccable when `layout` is set to [Layout::SideBySideInDiv].
    /// Default: `["is-inline"]`.
    #[prop_or(vec!["is-inline".to_string()])]
    pub input_div_classes: Vec<String>,
}

impl<Value, Key> FieldProps<Key> for RadioFieldProps<Value, Key>
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

impl<Value, Key> MultiValueFieldProps<Value, Key> for RadioFieldProps<Value, Key>
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

pub struct RadioFieldRenderer<Value, Key> {
    value_type: PhantomData<Value>,
    key_type: PhantomData<Key>,
}

impl<Value, Key> RadioFieldRenderer<Value, Key>
where
    Value: Clone + PartialEq + Display + Debug + 'static,
    Key: FieldKey + PartialEq + 'static,
{
    /// Create an `input` onchange handler for a specific `Value`.
    fn onchange_value(value: Value) -> impl Fn(ChangeData) -> MultiValueFieldMsg<Value, Key> {
        move |change_data: ChangeData| match change_data {
            ChangeData::Value(change_value) => match change_value.as_str() {
                "on" => MultiValueFieldMsg::Update(value.clone()),
                _ => {
                    panic!("Unexpected onchange value: {}.", change_value,);
                }
            },
            _ => {
                panic!("Invalid onchange data type: {:?}.", change_data,);
            }
        }
    }

    fn input(
        field: &MultiValueField<Value, Key, RadioFieldProps<Value, Key>, Self>,
        value: Value,
    ) -> Html {
        let selected = field.value.as_ref() == Some(&value);
        let disabled = field
            .props
            .disabled_options
            .iter()
            .find(|v| v == &&value)
            .is_some();
        let label = value.to_string();

        let onchange = field.link.callback(Self::onchange_value(value));
        let field_name = field.props.field_key.to_string();

        match field.props.layout {
            Layout::SideBySideInDiv => {
                let id = uuid::Uuid::new_v4();

                // This structure is used because it is more flexible for
                // custom css layouts than `<label><input/></label>`.
                html! {
                    <div class=field.props.input_div_classes.clone()>
                        <input
                            onchange=onchange
                            id=id.to_string()
                            class=field.props.input_classes.clone()
                            type="radio"
                            name=field_name
                            checked=selected
                            disabled=disabled/>
                        <label
                            for=id.to_string()
                            class=field.props.input_label_classes.clone()
                            disabled=disabled>
                            { label }
                        </label>
                    </div>
                }
            }
            Layout::InputInLabel => {
                html! {
                    <label
                        class=field.props.input_label_classes.clone()
                        disabled=disabled>
                        <input
                            onchange=onchange
                            class=field.props.input_classes.clone()
                            type="radio"
                            name=field_name
                            checked=selected
                            disabled=disabled/>
                        { label }
                    </label>
                }
            }
        }
    }
}

impl<Value, Key> MultiValueFieldRenderer<Value, Key, RadioFieldProps<Value, Key>>
    for RadioFieldRenderer<Value, Key>
where
    Value: Clone + PartialEq + Display + Debug + 'static,
    Key: FieldKey + PartialEq + 'static,
{
    fn render(field: &MultiValueField<Value, Key, RadioFieldProps<Value, Key>, Self>) -> Html {
        let mut classes = vec![];

        let validation_error =
            if let Some(errors) = field.display_validation_errors.get(&field.props.field_key) {
                classes.push("is-danger".to_string());
                let error_message = errors.to_string();
                html! {<p class="help is-danger">{ error_message }</p>}
            } else {
                html! {}
            };

        let label = field.label();

        let inputs: Html = field
            .props
            .options
            .iter()
            .map(|value| Self::input(field, value.clone()))
            .collect();

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
                    { inputs }
                </div>
                { validation_error }
            </div>
        }
    }
}
