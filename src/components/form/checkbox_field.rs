use super::{
    FieldKey, FieldLink, FieldMsg, FieldProps, FormField, FormFieldLink, FormMsg,
    NeqAssignFieldProps,
};
use form_validation::{ValidationErrors, AsyncValidatable, AsyncValidator};
use std::{fmt::Debug, rc::Rc, pin::Pin, future::Future};
use yew::{html, Callback, Component, ComponentLink, Properties, Children};
use yewtil::future::LinkFuture;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CheckboxState {
    Checked,
    Unchecked,
}

impl CheckboxState {
    pub fn checked(&self) -> bool {
        match self {
            CheckboxState::Checked => true,
            CheckboxState::Unchecked => false,
        }
    }

    pub fn toggle(&self) -> CheckboxState {
        match self {
            CheckboxState::Checked => CheckboxState::Unchecked,
            CheckboxState::Unchecked => CheckboxState::Checked,
        }
    }
}

impl From<bool> for CheckboxState {
    fn from(checked: bool) -> Self {
        if checked {
            Self::Checked
        } else {
            Self::Unchecked
        }
    }
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

impl<Key> Into<CheckboxFieldMsg<Key>> for FieldMsg {
    fn into(self) -> CheckboxFieldMsg<Key> {
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
    /// Html to use as the label for this field.
    pub children: Children,
    /// The initial state of the checkbox.
    #[prop_or(CheckboxState::Unchecked)]
    pub initial_state: CheckboxState,
    /// (Optional) What validator to use for this field.
    #[prop_or_default]
    pub validator: AsyncValidator<CheckboxState, Key>,
    /// (Optional) A callback for when this field changes.
    #[prop_or_default]
    pub onchange: Callback<CheckboxState>,
}

impl<Key> FieldProps<Key> for CheckboxFieldProps<Key>
where
    Key: FieldKey + 'static,
{
    fn form_link(&self) -> &FormFieldLink<Key> {
        &self.form_link
    }
    fn field_key(&self) -> &Key {
        &self.field_key
    }
}

pub enum CheckboxFieldMsg<Key> {
    Update,
    Validate,
    ValidationErrors(ValidationErrors<Key>)
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
}

impl<Key> Component for CheckboxField<Key>
where
    Key: FieldKey + 'static,
{
    type Message = CheckboxFieldMsg<Key>;
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
        }
    }

    fn update(&mut self, msg: Self::Message) -> yew::ShouldRender {
        match msg {
            CheckboxFieldMsg::Update => {
                self.value = self.value.toggle();
                self.props.onchange.emit(self.value);
                self.form_link
                    .send_form_message(FormMsg::FieldValueUpdate(self.props.field_key.clone()));
                self.update(CheckboxFieldMsg::Validate);

                true
            }
            CheckboxFieldMsg::Validate => {
                let validate_future = self.validate_future_or_empty();
                self.link.send_future(async move {
                    let validation_errors = validate_future.await;

                    CheckboxFieldMsg::ValidationErrors(validation_errors)
                });
                false
            }
            CheckboxFieldMsg::ValidationErrors(validation_errors) => {
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

    fn change(&mut self, props: Self::Properties) -> yew::ShouldRender {
        let link = self.link.clone();
        self.props.neq_assign_field(props, move |new_props| {
            Rc::new(CheckboxFieldLink {
                field_key: new_props.field_key().clone(),
                link: link.clone(),
            })
        })
    }
    fn view(&self) -> yew::Html {
        let onchange = self.link.callback(|_| CheckboxFieldMsg::Update);

        html! {
            <label class="checkbox">
                <input
                    type="checkbox"
                    onchange=onchange
                    checked=self.value.checked()
                    />
                { self.props.children.clone() }
            </label>
        }
    }
}

impl<Key> AsyncValidatable<Key> for CheckboxField<Key>
where
    Key: FieldKey,
{
    fn validate_future(&self) -> Pin<Box<dyn Future<Output = Result<(), ValidationErrors<Key>>>>> {
        let value = self.value.clone();
        let field_key = self.props.field_key.clone();
        let validator = self.props.validator.clone();
        Box::pin(async move { validator.validate_value(&value, &field_key).await })
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
