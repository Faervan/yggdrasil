use tokio::io::AsyncWriteExt;
use yserde::*;

#[allow(dead_code)]
#[derive(Package, Default, Debug)]
struct NormalTest {
    number: i128,
    word: String,
    #[do_not_send]
    internal_id: usize,
    //list: Vec<u16>
}

#[derive(Package, Default, Debug)]
struct TupleTest(u8, String, bool);

fn main() -> std::io::Result<()> {
    let packages = PackageMap::new(vec![
        Box::new(NormalTest::default()),
        Box::new(TupleTest::default())
    ]);
    let bytes0 = packages.pkg_as_bytes(
        Box::new(NormalTest {number: 30_009_900, word: "hello".to_string(), internal_id: 28_438})
    );
    let bytes1 = packages.pkg_as_bytes(
        Box::new(TupleTest(255, "This a @ cra21ly c00l $tr1n6! €€€".to_string(), false))
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
                packages.get_from_socket(&mut receiver).await.unwrap(),
                NormalTest => |pkg| {
                    println!("got {pkg:?}");
                },
                TupleTest => |pkg| {
                    println!("got {pkg:?}");
                }
            );
            i += 1;
            if i == 2 {break;}
        }
    });
    Ok(())
}
