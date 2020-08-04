use super::{FieldKey, FieldLink, FieldMsg, FormField, FormFieldLink, FormMsg};
use form_validation::{Validatable, Validation, ValidationErrors, Validator};
use std::{fmt::Debug, rc::Rc};
use web_sys::HtmlInputElement;
use yew::{html, Callback, Component, ComponentLink, NodeRef, Properties};

#[derive(Copy, Clone, PartialEq)]
pub enum CheckboxState {
    Checked,
    Unchecked,
}

pub struct CheckboxFieldLink<Key>
where
    Key: FieldKey + 'static,
{
    pub field_key: Key,
    pub link: ComponentLink<CheckboxField<Key>>,
}

impl<Key> Debug for CheckboxFieldLink<Key>
where
    Key: FieldKey + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CheckboxFieldLink<{0:?}>", self.field_key())
    }
}

impl Into<CheckboxFieldMsg> for FieldMsg {
    fn into(self) -> CheckboxFieldMsg {
        match self {
            FieldMsg::Validate => CheckboxFieldMsg::Validate,
        }
    }
}

impl<Key> FieldLink<Key> for CheckboxFieldLink<Key>
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

#[derive(Properties, Clone, PartialEq)]
pub struct CheckboxFieldProps<Key>
where
    Key: FieldKey + 'static,
{
    /// The key used to refer to this field.
    pub field_key: Key,
    /// The link to the form that this field belongs to.
    pub form_link: FormFieldLink<Key>,
    /// The initial state of the checkbox.
    #[prop_or(CheckboxState::Unchecked)]
    pub initial_state: CheckboxState,
    /// (Optional) A label to use for this field.
    #[prop_or_default]
    pub label: Option<String>,
    /// (Optional) What validator to use for this field.
    #[prop_or_default]
    pub validator: Validator<CheckboxState, Key>,
    /// (Optional) A callback for when this field changes.
    #[prop_or_default]
    pub onchange: Callback<CheckboxState>,
}

pub enum CheckboxFieldMsg {
    Update,
    Validate,
}

pub struct CheckboxField<Key>
where
    Key: FieldKey + 'static,
{
    value: CheckboxState,
    props: CheckboxFieldProps<Key>,
    form_link: FormFieldLink<Key>,
    link: ComponentLink<Self>,
    validation_errors: ValidationErrors<Key>,
    node_ref: NodeRef,
}

impl<Key> Component for CheckboxField<Key>
where
    Key: FieldKey + 'static,
{
    type Message = CheckboxFieldMsg;
    type Properties = CheckboxFieldProps<Key>;

    fn create(props: Self::Properties, link: yew::ComponentLink<Self>) -> Self {
        let form_link = props.form_link.clone();

        let field_link = CheckboxFieldLink {
            field_key: props.field_key.clone(),
            link: link.clone(),
        };

        form_link.register_field(Rc::new(field_link));

        Self {
            value: props.initial_state,
            form_link,
            link,
            props,
            validation_errors: ValidationErrors::default(),
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> yew::ShouldRender {
        match msg {
            CheckboxFieldMsg::Update => {
                let element: HtmlInputElement = self
                    .node_ref
                    .cast::<HtmlInputElement>()
                    .expect("unable to cast node ref");

                let value = if element.checked() {
                    CheckboxState::Checked
                } else {
                    CheckboxState::Unchecked
                };

                let changed = value != self.value;

                if changed {
                    self.value = value;
                    self.props.onchange.emit(value);
                    self.form_link
                        .send_form_message(FormMsg::FieldValueUpdate(self.props.field_key.clone()));
                    self.update(CheckboxFieldMsg::Validate);
                }

                true
            }
            CheckboxFieldMsg::Validate => {
                self.validation_errors = self.validate_or_empty();
                self.form_link
                    .send_form_message(FormMsg::FieldValidationUpdate(
                        self.props.field_key.clone(),
                        self.validation_errors.clone(),
                    ));
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> yew::ShouldRender {
        if self.props != props {
            if self.form_link != props.form_link {
                let form_link = props.form_link.clone();

                if !form_link.field_is_registered(&props.field_key) {
                    let field_link = CheckboxFieldLink {
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
    fn view(&self) -> yew::Html {
        let onchange = self.link.callback(|_| CheckboxFieldMsg::Update);

        html! {
            <label class="checkbox">
                <input
                    ref=self.node_ref.clone()
                    type="checkbox"
                    onchange=onchange
                    />
            { "label" }
            </label>
        }
    }
}

impl<Key> Validatable<Key> for CheckboxField<Key>
where
    Key: FieldKey,
{
    fn validate(&self) -> Result<(), form_validation::ValidationErrors<Key>> {
        self.props
            .validator
            .validate_value(&self.value, &self.props.field_key)
    }
}

impl<Key> FormField<Key> for CheckboxField<Key>
where
    Key: FieldKey,
{
    fn validation_errors(&self) -> &ValidationErrors<Key> {
        &self.validation_errors
    }

    fn field_key(&self) -> &Key {
        &self.props.field_key
    }
}
