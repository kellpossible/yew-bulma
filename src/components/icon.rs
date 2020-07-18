//! A `yew` [Component](yew::Component) to render a `bulma`
//! [icon](https://bulma.io/documentation/elements/icon/).

use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use crate::Color;
use yewtil::NeqAssign;

#[derive(Debug, Clone)]
pub struct Icon {
    pub props: Props,
    link: ComponentLink<Self>,
}

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props {
    #[prop_or_default]
    pub color: Option<Color>,
    #[prop_or_default]
    pub span_class: Vec<String>,
    pub class: Vec<String>,
}

impl Component for Icon {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Icon { props, link }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        let mut span_class = vec!["icon".to_string()];

        match &self.props.color {
            Some(color) => span_class.push(color.text_class()),
            None => {}
        }

        span_class.extend(self.props.span_class.clone());

        html! {
            <span class=span_class>
                <i class=self.props.class.clone()></i>
            </span>
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }
}
