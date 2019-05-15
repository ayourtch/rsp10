#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HtmlText {
    pub id: String,
    pub value: String,
    pub labeltext: String,
    pub highlight: bool,
    pub hidden: bool,
    pub disabled: bool,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HtmlButton {
    pub id: String,
    pub value: String,
    pub labeltext: String,
    pub highlight: bool,
    pub hidden: bool,
    pub disabled: bool,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HtmlCheck {
    pub id: String,
    pub labeltext: String,
    pub checked: bool,
    pub highlight: bool,
    pub hidden: bool,
    pub disabled: bool,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HtmlSelectItem<T> {
    pub i: usize,
    pub user_label: String,
    pub value: T,
    pub selected: bool,
}

use std::fmt::Debug;
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HtmlSelect<T: PartialEq + Clone + Debug> {
    pub id: String,
    pub items: Vec<HtmlSelectItem<T>>,
    pub selected_value: T,
    pub highlight: bool,
    pub hidden: bool,
    pub disabled: bool,
}

impl<T> HtmlSelect<T>
where
    T: std::cmp::PartialEq + std::clone::Clone + Debug,
{
    pub fn item(self: &mut HtmlSelect<T>, user_label: &str, value: T) {
        let i = HtmlSelectItem::<T> {
            user_label: user_label.into(),
            value,
            selected: false,
            i: self.items.len(),
        };
        self.items.push(i);
    }

    pub fn set_selected_value(self: &mut HtmlSelect<T>, selected_value: &mut T) {
        let mut found = false;
        // println!("Setting selected value: {:?}", &selected_value);
        for mut item in &mut self.items {
            if item.value == *selected_value {
                item.selected = true;
                found = true;
                self.selected_value = selected_value.clone();
            } else {
                item.selected = false;
            }
            // println!("Item: {:?}", &item);
        }
        if !found {
            if !self.items.is_empty() {
                self.items[0].selected = true;
                *selected_value = self.items[0].value.clone();
                self.selected_value = selected_value.clone();

            // println!("Setting selected value to {:?}", &selected_value);
            } else {
                // println!("Can not set default selected value");
            }
        } else {
            // println!("Found in items, selected value not reset");
        }
    }
}

impl HtmlSelect<String> {
    pub fn item1(self: &mut HtmlSelect<String>, user_label: &str) {
        let i = HtmlSelectItem::<String> {
            i: self.items.len(),
            user_label: user_label.into(),
            value: user_label.into(),
            selected: false,
        };
        self.items.push(i);
    }
}
