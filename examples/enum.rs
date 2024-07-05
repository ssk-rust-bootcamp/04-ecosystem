use anyhow::Result;
use serde::Serialize;
use strum::{
    Display, EnumCount, EnumDiscriminants, EnumIs, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr, VariantNames,
};
#[allow(unused)]
#[derive(Display, Debug, Serialize)]
enum Color {
    #[strum(serialize = "redred", to_string = "red")]
    Red,
    Green {
        range: usize,
    },
    Bule(usize),
    Yelllow,
    #[strum(to_string = "purple with {sat} saturation")]
    Purple {
        sat: usize,
    },
}
#[derive(Debug, EnumCount, EnumString, EnumDiscriminants, EnumIter, EnumIs, IntoStaticStr, VariantNames)]
#[allow(unused)]
enum MyEnum {
    A,
    B(String),
    C,
    D,
}

fn main() -> Result<()> {
    println!("{:?}", MyEnum::VARIANTS);
    MyEnum::iter().for_each(|v| println!("{:?}", v));
    println!("total :{:?}", MyEnum::COUNT);

    let my_enum = MyEnum::B("hello".to_string());
    println!("{:?}", my_enum.is_b());
    let s: &'static str = my_enum.into();
    println!("{:?}", s);

    let red = Color::Red;
    let green = Color::Green { range: 10 };
    let bule = Color::Bule(20);
    let yellow = Color::Yelllow;
    let purple = Color::Purple { sat: 30 };

    println!(
        "red :{:?} green :{:?} bule :{:?} yellow :{:?} purple :{:?}",
        red, green, bule, yellow, purple
    );

    let red_str = serde_json::to_string(&red)?;

    println!("red_str :{:?}", red_str);

    Ok(())
}
