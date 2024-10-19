use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};

use yserde_bytes::AsBytes;

#[allow(dead_code)]
#[derive(AsBytes, Default, Clone, Debug)]
struct TestStruct(u8, String, Option<isize>, Vec<u16>);

#[derive(AsBytes, Default, Clone, Debug)]
struct TestStruct2 {
    x: u32,
    y: String,
    z: i8,
    a: Option<String>,
    b: Vec<isize>,
    //c: TestStruct,
}

#[derive(AsBytes, Default, Debug)]
enum TestEnum {
    #[default]
    A,
    B(u8),
    C {
        x: TestStruct,
        y: Option<TestStruct2>
    },
    D(TestStruct3)
}

#[derive(AsBytes, Debug, Default)]
struct TestStruct3(
    #[u16]
    String
);

fn main() -> std::io::Result<()> {
    let test1 = TestStruct(240, "hello".to_string(), Some(9_000_800), vec![300, 255, 60_000]);
    let test2 = TestStruct2 {
        x: 5_000_000,
        y: "This is some string".to_string(),
        z: -100,
        a: Some("next string".to_string()),
        b: vec![20_999, 0, 999, 1, 6_809_800],
        //c: test1.clone()
    };
    println!("test1 as bytes: {:?}", test1.as_bytes());
    println!("test2 as bytes: {:?}", test2.as_bytes());
    let listener = TcpListener::bind("127.0.0.1:9983")?;
    let mut client = TcpStream::connect("127.0.0.1:9983")?;
    let (mut receiver, _) = listener.accept()?;
    let mut len_buf = [0; 4];

    client.write(test2.as_bytes().as_slice())?;
    receiver.read(&mut len_buf)?;
    let len = u32::from_ne_bytes(len_buf);
    let mut buf = vec![0; len as usize];
    receiver.read(&mut buf)?;
    println!("TestStruct2 from buf: {:#?}", TestStruct2::from_buf(&buf));

    let test3 = TestEnum::B(23).as_bytes();
    println!("test3 as bytes: {:?}", test3);
    client.write(test3.as_slice())?;
    receiver.read(&mut len_buf)?;
    let len = u32::from_ne_bytes(len_buf);
    let mut buf = vec![0; len as usize];
    receiver.read(&mut buf)?;
    println!("TestEnum from buf: {:#?}", TestEnum::from_buf(&buf));

    println!("test2: {test2:#?}");
    let test4 = TestEnum::C {x: test1, y: Some(test2)}.as_bytes();
    println!("test4 as bytes: {:?}", test4);
    client.write(test4.as_slice())?;
    receiver.read(&mut len_buf)?;
    let len = u32::from_ne_bytes(len_buf);
    let mut buf = vec![0; len as usize];
    receiver.read(&mut buf)?;
    println!("TestEnum from buf: {:#?}", TestEnum::from_buf(&buf));

    let test5 = TestEnum::D(TestStruct3("Some long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long stringSome long string".to_string())).as_bytes();
    println!("test5 as bytes: {:?}", test5);
    client.write(test5.as_slice())?;
    receiver.read(&mut len_buf)?;
    let len = u32::from_ne_bytes(len_buf);
    let mut buf = vec![0; len as usize];
    receiver.read(&mut buf)?;
    println!("TestEnum from buf: {:#?}", TestEnum::from_buf(&buf));
    Ok(())
}
