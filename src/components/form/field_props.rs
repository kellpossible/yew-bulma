use super::{FieldKey, FieldLink, FormFieldLink};
use std::rc::Rc;
use yew::{Properties, ShouldRender};
use form_validation::ValidationErrors;

pub trait FieldProps<Key>
where
    Key: FieldKey,
{
    fn form_link(&self) -> &FormFieldLink<Key>;
    fn field_key(&self) -> &Key;
    fn extra_errors(&self) -> &ValidationErrors<Key>;
}

pub trait NeqAssignFieldProps<Key>: FieldProps<Key> + Properties
where
    Key: FieldKey,
{
    fn neq_assign_field<C>(&mut self, new: Self, create_field_link: C) -> ShouldRender
    where
        C: Fn(&Self) -> Rc<dyn FieldLink<Key>>;
}

impl<Key, T> NeqAssignFieldProps<Key> for T
where
    T: FieldProps<Key> + Properties + PartialEq,
    Key: FieldKey + 'static,
{
    fn neq_assign_field<C>(&mut self, new: Self, create_field_link: C) -> ShouldRender
    where
        C: Fn(&Self) -> Rc<dyn FieldLink<Key>>,
    {
        if self != &new {
            if self.form_link() != new.form_link() {
                let form_link = new.form_link().clone();

                if !form_link.field_is_registered(new.field_key()) {
                    let field_link = create_field_link(&new);
                    form_link.register_field(field_link);
                }
            }
            *self = new;
            true
        } else {
            false
        }
    }
}
