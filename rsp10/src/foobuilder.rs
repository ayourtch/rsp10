use mustache::MapBuilder;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type FooClosure = Box<dyn Fn(MapBuilder) -> MapBuilder>;

struct FooItemBuilder {
    name: String,
    part: Option<FooClosure>,
}

impl FooItemBuilder {
    fn new(name: &str) -> Self {
        FooItemBuilder {
            name: name.to_string(),
            part: None,
        }
    }
    fn add<F>(&mut self, f: F)
    where
        F: Fn(MapBuilder) -> MapBuilder,
        F: 'static,
    {
        self.part = Some(Box::new(f));
    }

    fn add_field<T>(&mut self, data: &Rc<RefCell<T>>)
    where
        T: serde::Serialize + Clone,
        T: 'static,
    {
        self.add({
            let name = self.name.to_owned();
            let data = data.to_owned();
            move |x: MapBuilder| x.insert(name.clone(), &data.borrow().clone()).unwrap()
        })
    }

    fn insert_fn<F>(&mut self, f: F)
    where
        F: FnMut(String) -> String + Clone + Send + Sync + 'static,
    {
        let fn2 = RefCell::new(Box::new(f));
        self.add({
            let name = self.name.to_owned();
            move |x: MapBuilder| x.insert_fn(name.clone(), fn2.clone().into_inner())
        })
    }

    fn insert_fn2<F>(&mut self, f: F)
    where
        F: FnMut(String, &mut (dyn FnMut(String) -> String)) -> String + Clone + Send + Sync + 'static,
    {
        let fn2 = RefCell::new(Box::new(f));
        self.add({
            let name = self.name.to_owned();
            move |x: MapBuilder| x.insert_fn2(name.clone(), fn2.clone().into_inner())
        })
    }

    fn insert_data<T>(&mut self, data: &T)
    where
        T: serde::Serialize + Clone,
        T: 'static,
    {
        let rc = Rc::new(RefCell::new(data.clone()));
        self.add_field(&rc);
    }

    fn build(self, map_builder: MapBuilder) -> MapBuilder {
        match self.part {
            None => map_builder,
            Some(part) => part(map_builder),
        }
    }
}

pub struct FooVecBuilder {
    name: String,
    parts: Vec<Vec<FooClosure>>,
}

impl FooVecBuilder {
    fn new(name: &str) -> Self {
        FooVecBuilder {
            name: name.to_string(),
            parts: vec![],
        }
    }

    fn add<F>(&mut self, f: F)
    where
        F: Fn(MapBuilder) -> MapBuilder,
        F: 'static,
    {
        self.parts.push(vec![Box::new(f)]);
    }

    fn add_at<F>(&mut self, i: usize, f: F)
    where
        F: Fn(MapBuilder) -> MapBuilder,
        F: 'static,
    {
        while self.parts.len() + 1 < i {
            self.parts.push(vec![Box::new(|x| x)])
        }
        if self.parts.len() > i {
            self.parts[i].push(Box::new(f));
        } else {
            self.parts.push(vec![Box::new(f)]);
        }
    }

    pub fn add_field_at<T>(&mut self, i: usize, name: &str, data: &Rc<RefCell<T>>)
    where
        T: serde::Serialize + Clone,
        T: 'static,
    {
        self.add_at(i, {
            let name = name.to_owned();
            let data = data.to_owned();
            move |x: MapBuilder| x.insert(name.clone(), &data.borrow().clone()).unwrap()
        })
    }

    pub fn insert_data_at<T>(&mut self, i: usize, name: &str, data: &T)
    where
        T: serde::Serialize + Clone,
        T: 'static,
    {
        let rc = Rc::new(RefCell::new(data.clone()));
        self.add_field_at(i, name, &rc);
    }

    fn add_field<T>(&mut self, name: &str, data: &Rc<RefCell<T>>)
    where
        T: serde::Serialize + Clone,
        T: 'static,
    {
        self.add_field_at(self.parts.len(), name, data);
    }

    fn build(self, map_builder: MapBuilder) -> MapBuilder {
        let map_builder = map_builder.insert_vec(&self.name, |builder| {
            let mut builder = builder;
            self.parts.iter().fold(builder, |builder, getter| {
                builder.push_map(|m| getter.iter().fold(m, |m, g| g(m)))
            })
        });
        map_builder
    }
}

enum FooAnyBuilder {
    Map(FooMapBuilder),
    Vector(FooVecBuilder),
    Item(FooItemBuilder),
}

pub struct FooMapBuilder {
    builders: HashMap<String, FooAnyBuilder>,
}

impl FooMapBuilder {
    pub fn new() -> Self {
        FooMapBuilder {
            builders: HashMap::new(),
        }
    }

    pub fn vector<'a, F>(&'a mut self, name: &str, vector_action: F)
    where
        F: Fn(&mut FooVecBuilder),
        F: 'a,
    {
        let builder = self.builders.entry(name.to_string());
        let mut builder =
            builder.or_insert_with(|| FooAnyBuilder::Vector(FooVecBuilder::new(name)));
        if let FooAnyBuilder::Vector(ref mut b) = builder {
            vector_action(b);
        } else {
            panic!("wrong builder");
        }
    }

    pub fn item<T>(&mut self, name: &str, data: &Rc<RefCell<T>>)
    where
        T: serde::Serialize + Clone,
        T: 'static,
    {
        let builder = self.builders.entry(name.to_string());
        let mut builder = builder.or_insert_with(|| FooAnyBuilder::Item(FooItemBuilder::new(name)));
        if let FooAnyBuilder::Item(b) = builder {
            b.add_field(data);
        } else {
            panic!("wrong builder");
        }
    }

    pub fn insert<T>(&mut self, name: &str, data: &T)
    where
        T: serde::Serialize + Clone,
        T: 'static,
    {
        let builder = self.builders.entry(name.to_string());
        let mut builder = builder.or_insert_with(|| FooAnyBuilder::Item(FooItemBuilder::new(name)));
        if let FooAnyBuilder::Item(b) = builder {
            b.insert_data(data);
        } else {
            panic!("wrong builder");
        }
    }

    pub fn insert_fn<F>(&mut self, name: &str, data: F)
    where
        F: FnMut(String) -> String + Clone + Send + Sync + 'static,
    {
        let builder = self.builders.entry(name.to_string());
        let mut builder = builder.or_insert_with(|| FooAnyBuilder::Item(FooItemBuilder::new(name)));
        if let FooAnyBuilder::Item(b) = builder {
            b.insert_fn(data);
        } else {
            panic!("wrong builder");
        }
    }

    pub fn insert_fn2<F>(&mut self, name: &str, data: F)
    where
        F: FnMut(String, &mut (dyn FnMut(String) -> String)) -> String + Clone + Send + Sync + 'static,
    {
        let builder = self.builders.entry(name.to_string());
        let mut builder = builder.or_insert_with(|| FooAnyBuilder::Item(FooItemBuilder::new(name)));
        if let FooAnyBuilder::Item(b) = builder {
            b.insert_fn2(data);
        } else {
            panic!("wrong builder");
        }
    }

    pub fn build(self, map_builder: MapBuilder) -> MapBuilder {
        let mut map_builder = map_builder;
        for (k, v) in self.builders {
            match v {
                FooAnyBuilder::Item(ht) => {
                    map_builder = ht.build(map_builder);
                }
                FooAnyBuilder::Vector(ht) => {
                    map_builder = ht.build(map_builder);
                }
                _ => {}
            }
        }
        map_builder
    }
}
