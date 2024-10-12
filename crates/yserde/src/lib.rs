//! Little library that intends to make it a bit easier to send and
//! receive structs via byte streams.
//!
//! # Usage
//!
//! To use this crate, you need to implement [Package] for the types you want to use
//! as packages and then create a [PackageIndex] instance, which you can use to
//! convert your packages to and from bytes.
//!
//! The [read_async_tcp](PackageIndex::read_async_tcp) Method will return a `Box<dyn Any>`
//! which can be matched like an enum via the [match_pkg] macro.
//!
//! # Example
//!
//! ```
//! # use yserde::{Package, PackageIndex, match_pkg};
//! # use tokio::{io::AsyncWriteExt, net::{TcpListener, TcpStream}, runtime::Runtime};
//! # use std::any::Any;
//! #[derive(Package, Debug, Default, PartialEq)]
//! struct HelloPackage {
//!     some_string: String,
//!     some_num: u32,
//! }
//!
//! let tcp_packages = PackageIndex::new(vec![
//!     Box::new(HelloPackage::default())
//! ]);
//!
//! # let rt = Runtime::new().unwrap();
//! # rt.block_on(async {
//! # let listener = TcpListener::bind("127.0.0.1:9983").await.unwrap();
//! # let mut tcp_client = TcpStream::connect("127.0.0.1:9983").await.unwrap();
//! # let (mut tcp_receiver, _) = listener.accept().await.unwrap();
//! tcp_client.write(&tcp_packages.pkg_as_bytes(Box::new(HelloPackage {
//!     some_string: "This is a test String".to_string(),
//!     some_num: 267_509
//! }))).await.unwrap();
//!
//! match_pkg!(
//!     tcp_packages.read_async_tcp(&mut tcp_receiver).await.unwrap(),
//!     HelloPackage => |hello: Box<HelloPackage>| {
//!         println!("hello!\n{hello:#?}");
//!         assert_eq!(
//!             *hello,
//!             HelloPackage {
//!                 some_string: "This is a test String".to_string(),
//!                 some_num: 267_509
//!             }
//!         );
//!    }
//! );
//! # });
//! ```
use std::{any::TypeId, collections::{HashMap, HashSet}, fmt::Debug, io::Read};
use tokio::io::AsyncReadExt;

pub use tokio::net::TcpStream;

/// Automatic implementation of the [Package] trait
///
/// Supports the primitive number types as well as bools,
/// Strings and any other type that implements [Package].<br>
/// `Option`'s and `Vec`'s containing these types can also be used.
///
/// The `#[yserde_ignore]` Attribute may be used to specify that a field should be ignored while
/// serializing. When deserializing, this field will keep the value assigned by the `default()`
/// method.
///
/// Example:
/// ```
/// use yserde::*;
///
/// #[derive(Package, Default, Debug)]
/// struct Coordinate(f32, f32, f32);
///
/// #[derive(Package, Default, Debug)]
/// struct Person {
///     name: String,
///     age: u8,
///     birth_place: Option<Coordinate>,
/// }
///
/// #[derive(Package, Debug)]
/// struct Group {
///     // the internal id should not be send
///     #[yserde_ignore]
///     internal_id: usize,
///     name: String,
///     head_quarter: Coordinate,
///     members: Vec<Person>,
/// }
///
/// impl Group {
///     fn default() -> Self {
///         Group {
///             // When deserializing a `Group`, 99999 will allways stay
///             internal_id: 99999,
///             name: "Gang X".to_string(),
///             head_quarter: Coordinate::default(),
///             members: vec![],
///         }
///     }
/// }
/// ```
pub use yserde_derive::Package;
pub use std::any::Any;

#[cfg(test)]
mod tests;

/// Trait for types that need to be converted to and from bytes
pub trait Package: Any + Debug {
    fn get_new(&self) -> Box<dyn Package>;
    fn as_bytes(&self) -> Vec<u8> {vec![]}
    fn from_tcp(&self, socket: &mut std::net::TcpStream) -> std::io::Result<Box<dyn Any>>;
    fn from_async_tcp(&self, socket: &mut tokio::net::TcpStream) -> tokio::io::Result<Box<dyn Any>>;
    //fn from_udp(&self, socket: &mut std::net::UdpSocket) -> std::io::Result<Box<dyn Any>>;
    //fn from_async_udp(&self, socket: &mut tokio::net::UdpSocket) -> tokio::io::Result<Box<dyn Any>>;
    fn as_bytes_indexed(&self, map: &PackageIndex) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.push(*map.ser_map.get(&(&*self).type_id()).expect("Package is not registered in the given PackageIndex"));
        bytes.extend_from_slice(&self.as_bytes());
        bytes
    }
}

/// A "dictionary" used to map package types to bytes
///
/// Example
/// ```
/// # use yserde::{PackageIndex, Package};
/// # use tokio::net::TcpStream;
/// # use std::any::Any;
/// # #[derive(Package, Default, Debug)]
/// # struct SomePkg;
/// # #[derive(Package, Default, Debug)]
/// # struct AnotherPkg;
/// let package_kinds = PackageIndex::new(vec![
///     Box::new(SomePkg),
///     Box::new(AnotherPkg),
/// ]);
/// // SomePkg is now mapped to 0 and AnotherPkg is mapped to 1
/// let some_pkg_bytes = package_kinds.pkg_as_bytes(Box::new(SomePkg));
/// let another_pkg_bytes = package_kinds.pkg_as_bytes(Box::new(AnotherPkg));
/// assert_eq!(some_pkg_bytes, vec![0]);
/// assert_eq!(another_pkg_bytes, vec![1]);
/// ```
pub struct PackageIndex {
    ser_map: HashMap<TypeId, u8>,
    de_map: HashMap<u8, Box<dyn Package>>,
}

impl PackageIndex {
    /// Create a new PackageIndex dictionary
    ///
    /// Every package type can only exist once per PackageIndex (as unique identification
    /// of a type is the very point of it)
    ///
    /// Example
    /// ```
    /// # use yserde::{PackageIndex, Package};
    /// # use tokio::net::TcpStream;
    /// # use std::any::Any;
    /// # #[derive(Package, Default, Debug)]
    /// # struct HelloPkg;
    /// # #[derive(Package, Debug, Default)]
    /// # struct ByePkg;
    /// let package_kinds = PackageIndex::new(vec![
    ///     Box::new(HelloPkg),
    ///     Box::new(ByePkg::default()),
    /// ]);
    /// ```
    pub fn new(list: Vec<Box<dyn Package>>) -> PackageIndex {
        let mut type_list: HashSet<TypeId> = HashSet::new();
       let (ser_map, de_map) = list
                                .into_iter()
                                .filter(|p| type_list.insert((&**p).type_id()))
                                .enumerate()
                                .map(|(index, obj)| (((&*obj).type_id(), index as u8), (index as u8, obj)))
                                .unzip();
        PackageIndex {
            de_map,
            ser_map,
        }
    }
    /// Convert a [Package] to a vector of bytes<br>
    /// This is an alias for [Package::as_bytes_indexed]
    pub fn pkg_as_bytes(&self, package: Box<dyn Package>) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.push(*self.ser_map.get(&(&*package).type_id()).expect("Package is not registered in the given PackageIndex"));
        bytes.extend_from_slice(&package.as_bytes());
        bytes
    }
    /// Receive a [Package] (represented as `Box<dyn Any>`) from a [TcpStream](std::net::TcpStream)<br>
    /// The resulting `Box<dyn Any>` can be matched by the [match_pkg] macro
    pub fn read_tcp(&self, tcp: &mut std::net::TcpStream) -> std::io::Result<Box<dyn Any>> {
        let mut buf = [0;1];
        tcp.read(&mut buf)?;
        if let Some(obj) = self.de_map.get(&buf[0]) {
            let obj = obj.get_new();
            return obj.from_tcp(tcp);
        }
        Err(tokio::io::ErrorKind::NotFound.into())
    }
    /// Receive a [Package] (represented as `Box<dyn Any>`) from a [tokio] [TcpStream]<br>
    /// The resulting `Box<dyn Any>` can be matched by the [match_pkg] macro
    pub async fn read_async_tcp(&self, tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {
        let mut buf = [0;1];
        tcp.read(&mut buf).await?;
        if let Some(obj) = self.de_map.get(&buf[0]) {
            let obj = obj.get_new();
            tcp.readable().await?;
            return obj.from_async_tcp(tcp);
        }
        Err(tokio::io::ErrorKind::NotFound.into())
    }
}

/// Macro used to match against the inner types of a `Box<dyn Any>`
///
/// First argument is the `Box<dyn Any>` to match against, followed by a list of
/// types with an associated closure for each type that has the downcasted type as input
///
/// ### Example
/// ```
/// # use yserde::{match_pkg, Package, PackageIndex};
/// # use std::any::Any;
/// # use tokio::net::TcpStream;
/// # #[derive(Package, Default, Debug)]
/// # struct Hello;
/// # #[derive(Package, Default, Debug)]
/// # struct Message;
/// # let packages = PackageIndex::new(vec![Box::new(Hello), Box::new(Message)]);
/// # let rt = tokio::runtime::Runtime::new().unwrap();
/// # rt.block_on(async {
/// # if let Ok(mut receiver) = TcpStream::connect("127.0.0.1:9983").await {
/// match_pkg!(
///     packages.read_async_tcp(&mut receiver).await.unwrap(),
///     Hello => |hello| {
///         println!("hello! {hello:?}");
///         println!("as bytes: {:?}", packages.pkg_as_bytes(hello));
///     },
///     Message => |msg| {
///         println!("message! {msg:?}");
///         println!("as bytes: {:?}", packages.pkg_as_bytes(msg));
///     }
/// );
/// # }});
/// ```
#[macro_export]
macro_rules! match_pkg {
    ( $pkg:expr, $( $pkg_variant:ty => $handler:expr ), * ) => {
        'inner: {
            let pkg: Box<dyn Any> = $pkg;
            $(
                if pkg.is::<$pkg_variant>() {
                    $handler(pkg.downcast::<$pkg_variant>().unwrap());
                    break 'inner;
                }
            )*
        }
    };
}
