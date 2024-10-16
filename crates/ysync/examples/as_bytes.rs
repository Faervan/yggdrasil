use yserde_bytes::AsBytes;

#[allow(dead_code)]
#[derive(AsBytes, Default)]
enum TestEnum {
    #[default]
    A,
    B(TestStruct2, u8),
    C(TestStruct),
    D {
        x: u16,
        y: TestStruct,
        z: TestStruct2
    }
}

#[allow(dead_code)]
#[derive(AsBytes, Default, Clone)]
struct TestStruct(u8, String, Option<isize>, Vec<u16>);

#[derive(AsBytes, Default, Clone)]
struct TestStruct2 {
    x: u32,
    y: String
}

#[derive(AsBytes, Default)]
struct EmptyStruct;

fn main() {
    let test1 = TestStruct(240, "hello".to_string(), Some(9_000_800), vec![300, 255, 60_000]);
    let test2 = TestStruct2 {x: 5_000_000, y: "This is some string".to_string()};
    println!("test1 as bytes: {:?}", test1.as_bytes());
    println!("test2 as bytes: {:?}", test2.as_bytes());
    let test_enum_a = TestEnum::A;
    let test_enum_b = TestEnum::B(test2.clone(), 19);
    let test_enum_c = TestEnum::C(test1.clone());
    let test_enum_d = TestEnum::D {x: 65_000, y: test1, z: test2};
    println!("test_enum_a as bytes: {:?}", test_enum_a.as_bytes());
    println!("test_enum_b as bytes: {:?}", test_enum_b.as_bytes());
    println!("test_enum_c as bytes: {:?}", test_enum_c.as_bytes());
    println!("test_enum_d as bytes: {:?}", test_enum_d.as_bytes());
    println!("max size of TestStruct: {}", TestStruct::MAX_SIZE);
    println!("max size of TestStruct2: {}", TestStruct2::MAX_SIZE);
    println!("max size of EmptyStruct: {}", EmptyStruct::MAX_SIZE);
    println!("max size of TestEnum: {}", TestEnum::MAX_SIZE);
}
