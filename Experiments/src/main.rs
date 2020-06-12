struct Item<'a> {
    id: u8,
    listener: &'a Listener,
}

struct Listener {
    some: String
}

struct Items<'a> {
    items: Vec<Item<'a>>,
    listener: Listener,
}


fn main() {
    let listener = Listener { some: "".to_string() };
    let mut items = Items {
        items: Default::default(),
        listener,
    };

    {
        create_items(&mut items);
    };

}

fn create_items<'a>(items: &'a mut Items<'a>) {
    let item = Item {
        id: 100,
        listener: &items.listener,
    };
    let items:&mut Vec<Item<'a>> = &mut items.items;
    items.push(item);
}