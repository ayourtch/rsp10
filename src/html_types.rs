use serde::Serialize;
use std::cell::RefCell;
use std::rc::Rc;
use std::string::ToString;

#[derive(Debug, Clone)]
pub struct HtmlForm {
    list: Vec<Rc<RefCell<HtmlInput>>>,
}

impl HtmlForm {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }

    pub fn push<S: HtmlInput + Debug + 'static>(&mut self, input: Rc<RefCell<S>>) {
        self.list.push(input.clone());
    }
    /*
    pub fn mustache_render(&self, data: mustache::MapBuilder) -> mustache::MapBuilder {
        let mut data = data;
        for elt in &self.list {
            println!("elt: {:?}", &elt);
            data = elt.borrow().mustache_render(data);
        }
        data
    }
    */
}

#[derive(Debug, Clone)]
pub struct HtmlFormVector {
    pub name: String,
    pub forms: Vec<HtmlForm>,
}

impl HtmlFormVector {
    pub fn new(a_name: &str) -> Self {
        Self {
            name: a_name.to_string(),
            forms: vec![],
        }
    }
}

impl HtmlInput for HtmlFormVector {
    fn mustache_render(&self, data: mustache::MapBuilder) -> mustache::MapBuilder {
        let mut data = data;
        data = data.insert_vec(format!("{}", self.name), |mut vdata| {
            for (i, ref aform) in self.forms.iter().enumerate() {
                vdata = vdata.push_map(|mut data| {
                    data = aform.mustache_render(data);
                    data
                });
            }
            vdata
        });

        data
    }
}

pub trait HtmlInput: Debug {
    fn get_sid(&self, id: &str) -> String {
        let name_vec: Vec<&str> = id.split("__").collect();
        if name_vec.len() == 1 {
            format!("{}", id)
        } else {
            format!("{}", name_vec[2])
        }
    }
    fn mustache_render(&self, data: mustache::MapBuilder) -> mustache::MapBuilder;
}

impl HtmlInput for HtmlForm {
    fn mustache_render(&self, data: mustache::MapBuilder) -> mustache::MapBuilder {
        let mut data = data;
        for elt in &self.list {
            // println!("elt: {:?}", &elt);
            data = elt.borrow().mustache_render(data);
        }
        data
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HtmlText {
    pub id: String,
    pub value: String,
    pub labeltext: String,
    pub highlight: bool,
    pub hidden: bool,
    pub disabled: bool,
}

impl HtmlInput for HtmlText {
    fn mustache_render(&self, data: mustache::MapBuilder) -> mustache::MapBuilder {
        let mut data = data.insert(self.get_sid(&self.id), &self).unwrap();
        if self.value == "false".to_string() {
            // if it looks like a false, insert a boolean
            data = data.insert(format!("{}.value", self.get_sid(&self.id)), &false).unwrap();
        }
        data
    }
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

impl HtmlInput for HtmlButton {
    fn mustache_render(&self, data: mustache::MapBuilder) -> mustache::MapBuilder {
        data.insert(self.get_sid(&self.id), &self).unwrap()
    }
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

impl HtmlInput for HtmlCheck {
    fn mustache_render(&self, data: mustache::MapBuilder) -> mustache::MapBuilder {
        data.insert(self.get_sid(&self.id), &self).unwrap()
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HtmlSelectItem<T: PartialEq + Clone + Debug + ToString + Serialize> {
    pub i: usize,
    pub user_label: String,
    pub value: T,
    pub selected: bool,
}

impl<T> HtmlInput for HtmlSelectItem<T>
where
    T: PartialEq + Clone + Debug + ToString + Serialize,
{
    fn mustache_render(&self, data: mustache::MapBuilder) -> mustache::MapBuilder {
        let data = data.insert("item_number".to_string(), &self.i).unwrap();
        data.insert(format!("id_{}", self.i), &self).unwrap()
    }
}

use std::fmt::Debug;
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HtmlSelect<T: PartialEq + Clone + Debug + ToString + Serialize> {
    pub id: String,
    pub items: Vec<HtmlSelectItem<T>>,
    pub selected_value: T,
    pub highlight: bool,
    pub hidden: bool,
    pub disabled: bool,
}

impl<T> HtmlInput for HtmlSelect<T>
where
    T: PartialEq + Clone + Debug + ToString + Serialize,
{
    fn mustache_render(&self, data: mustache::MapBuilder) -> mustache::MapBuilder {
        data.insert(self.get_sid(&self.id), &self).unwrap()
    }
}

impl<T> HtmlSelect<T>
where
    T: std::cmp::PartialEq + std::clone::Clone + Debug + ToString + Serialize,
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
