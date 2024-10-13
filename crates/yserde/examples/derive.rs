use tokio::io::AsyncWriteExt;
use std::io::Read;
use yserde::*;

#[allow(dead_code)]
#[derive(Package, Default, Debug)]
struct NormalTest {
    number: i128,
    word: String,
    #[yserde_ignore]
    internal_id: usize,
    int_list: Vec<u16>,
    string_list: Vec<String>,
    int_option: Option<i8>,
    string_option: Option<String>,
    bool_option: Option<bool>,
    another_type: TupleTest,
    type_list: Vec<TestType>,
    unsupported: char,
}

#[derive(Package, Default, Debug)]
struct TupleTest(usize, String, bool);

#[derive(Package, Default, Debug)]
struct TestType {
    x: i32,
    y: Option<TupleTest>,
}

fn main() -> std::io::Result<()> {
    let packages = PackageIndex::new(vec![
        Box::new(NormalTest::default()),
        Box::new(TupleTest::default())
    ]);
    let bytes0 = packages.pkg_as_bytes(
        Box::new(NormalTest {
            number: 30_009_900,
            word: "hello".to_string(),
            internal_id: 28_438,
            int_list: vec![60_000, 999, 3_456],
            string_list: vec![
                "Rust".to_string(),
                "is so".to_string(),
                "cool".to_string()
            ],
            int_option: Some(-120),
            string_option: Some("I am a string :)".to_string()),
            bool_option: None,
            another_type: TupleTest(12, "Thµs es aWesom€!".to_string(), false),
            type_list: vec![
                TestType {x: -55_000, y: None},
                TestType {x: 300_999, y: Some(TupleTest(0, "".to_string(), true))}
            ],
            unsupported: '|'
        })
    );
    let bytes1 = packages.pkg_as_bytes(
        Box::new(TupleTest(255, "This a ¢ra21ly c00l $tr1n6!".to_string(), false))
    );
    println!("as bytes: {:?}", bytes0);
    println!("as bytes: {:?}", bytes1);
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:9983").await.unwrap();
        let mut client = TcpStream::connect("127.0.0.1:9983").await.unwrap();
        let (mut receiver, _) = listener.accept().await.unwrap();
        let _ = client.write(&bytes0).await;
        let _ = client.write(&bytes1).await;
        let mut i = 0;
        loop {
            match_pkg!(
                packages.read_async_tcp(&mut receiver).await.unwrap(),
                NormalTest => |pkg| {
                    println!("got {pkg:#?}");
                },
                TupleTest => |pkg| {
                    println!("got {pkg:#?}");
                }
            );
            i += 1;
            if i == 2 {break;}
        }
    });
    Ok(())
}
