//! Little library that intends to make it a bit easier to send and
//! receive structs via byte streams.
//!
//! Currently all data is expected to be send via [tokio's TcpStream](tokio::net::TcpStream)
//! or [tokio's UdpSocket](tokio::net::UdpSocket)
//!
//! ### Usage
//!
//! To use this crate, you need to implement [Package] for the types you want to use
//! as packages and then create a [PackageMap] instance, which you can use to
//! convert your packages to and from bytes.
//!
//! The [get_from_tcp](PackageMap::get_from_tcp) Method will return a `Box<dyn Any>`
//! which can be matched like an enum via the [match_pkg] macro.
//!
//! ### Example
//!
//! ```
//! # use yserde::{Package, PackageMap, match_pkg};
//! # use tokio::{io::AsyncWriteExt, net::{TcpListener, TcpStream}, runtime::Runtime};
//! # use std::any::Any;
//! # #[derive(Debug, Default, PartialEq)]
//! struct HelloPackage {
//!     some_string: String,
//!     some_num: u32,
//! }
//! impl Package for HelloPackage {
//! #    fn get_new(&self) -> Box<dyn Package> {
//! #        Box::new(HelloPackage::default())
//! #    }
//!     fn as_bytes(&self) -> Vec<u8> {
//!         let mut bytes = vec![];
//!         bytes.extend_from_slice(&self.some_num.to_ne_bytes());
//!         bytes.push(self.some_string.len() as u8);
//!         bytes.extend_from_slice(self.some_string.as_bytes());
//!         bytes
//!     }
//!     fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {
//!         let mut buf = [0;5];
//!         _tcp.try_read(&mut buf)?;
//!         let mut string_buf = vec![0; buf[4].into()];
//!         _tcp.try_read(&mut string_buf)?;
//!         Ok(Box::new(HelloPackage {
//!             some_string: String::from_utf8_lossy(&string_buf).to_string(),
//!             some_num: u32::from_ne_bytes(buf[..4].try_into().unwrap())
//!         }))
//!     }
//! }
//!
//! let tcp_packages = PackageMap::new(vec![
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
//!     tcp_packages.get_from_tcp(&mut tcp_receiver).await.unwrap(),
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
use std::{any::{Any, TypeId}, collections::{HashMap, HashSet}, fmt::Debug};

use tokio::{io::AsyncReadExt, net::TcpStream};

#[cfg(test)]
mod tests;

/// Trait for types that need to be converted to and from bytes
pub trait Package: Any + Debug {
    /// This method is needed because [Package] can't implement [Clone] but needs to be cloned
    /// anyway lol<br>
    /// Just return a `Box::new(MyPkg::default())` and be done with it.
    ///
    /// Example
    /// ```
    /// # use yserde::{PackageMap, Package};
    /// # use tokio::net::TcpStream;
    /// # use std::any::Any;
    /// # #[derive(Debug)]
    /// # struct MyPkg;
    /// impl Package for MyPkg {
    ///     fn get_new(&self) -> Box<dyn Package> {
    ///         Box::new(MyPkg)
    ///     }
    /// #   fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {Ok(Box::new(MyPkg))}
    /// }
    fn get_new(&self) -> Box<dyn Package>;
    /// Transform the package into bytes. This can be ommitted if the [Package] does not have any
    /// fields.<br>
    /// Note that the package type is prepended automatically
    ///
    /// Example
    /// ```
    /// # use yserde::Package;
    /// # use tokio::net::TcpStream;
    /// # use std::any::Any;
    /// # #[derive(Debug)]
    /// struct HelloPackage {
    ///     some_string: String,
    ///     some_num: u32,
    /// }
    /// impl Package for HelloPackage {
    ///     fn as_bytes(&self) -> Vec<u8> {
    ///         let mut bytes = vec![];
    ///         // first 4 bytes are used for some_num
    ///         bytes.extend_from_slice(&self.some_num.to_ne_bytes());
    ///         // fifth byte is used for length of some_string
    ///         bytes.push(self.some_string.len() as u8);
    ///         // rest is some_string
    ///         bytes.extend_from_slice(self.some_string.as_bytes());
    ///         bytes
    ///     }
    /// #    fn get_new(&self) -> Box<(dyn Package + 'static)> { todo!() }
    /// #    fn from_bytes(&self, _: &mut tokio::net::TcpStream) -> Result<Box<(dyn Any + 'static)>, std::io::Error> { todo!() }
    /// }
    /// ```
    fn as_bytes(&self) -> Vec<u8> {vec![]}
    /// Read the package from a [TcpStream].<br>
    /// Note that the package type is already read by [PackageMap].
    ///
    /// Example
    /// ```
    /// # use yserde::Package;
    /// # use tokio::{io::AsyncWriteExt, net::{TcpListener, TcpStream}, runtime::Runtime};
    /// # use std::any::Any;
    /// # #[derive(Debug, Default)]
    /// struct HelloPackage {
    ///     some_string: String,
    ///     some_num: u32,
    /// }
    /// impl Package for HelloPackage {
    ///     fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {
    ///         // get bytes for some_num and length of some_string
    ///         let mut buf = [0;5];
    ///         _tcp.try_read(&mut buf)?;
    ///         // get some_string
    ///         let mut string_buf = vec![0; buf[4].into()];
    ///         _tcp.try_read(&mut string_buf)?;
    ///         Ok(Box::new(HelloPackage {
    ///             some_string: String::from_utf8_lossy(&string_buf).to_string(),
    ///             some_num: u32::from_ne_bytes(buf[..4].try_into().unwrap())
    ///         }))
    ///     }
    /// #    fn get_new(&self) -> Box<(dyn Package + 'static)> { todo!() }
    /// }
    /// ```
    fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>>;
}

/// A "dictionary" used to map package types to bytes
///
/// Example
/// ```
/// # use yserde::{PackageMap, Package};
/// # use tokio::net::TcpStream;
/// # use std::any::Any;
/// # #[derive(Debug)]
/// # struct SomePkg;
/// # impl Package for SomePkg {
/// #    fn get_new(&self) -> Box<dyn Package> {Box::new(SomePkg)}
/// #    fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {Ok(Box::new(SomePkg))}
/// # }
/// # #[derive(Debug)]
/// # struct AnotherPkg;
/// # impl Package for AnotherPkg {
/// #    fn get_new(&self) -> Box<dyn Package> {Box::new(AnotherPkg)}
/// #    fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {Ok(Box::new(AnotherPkg))}
/// # }
/// let package_kinds = PackageMap::new(vec![
///     Box::new(SomePkg),
///     Box::new(AnotherPkg),
/// ]);
/// // SomePkg is now mapped to 0 and AnotherPkg is mapped to 1
/// let some_pkg_bytes = package_kinds.pkg_as_bytes(Box::new(SomePkg));
/// let another_pkg_bytes = package_kinds.pkg_as_bytes(Box::new(AnotherPkg));
/// assert_eq!(some_pkg_bytes, vec![0]);
/// assert_eq!(another_pkg_bytes, vec![1]);
/// ```
pub struct PackageMap {
    ser_map: HashMap<TypeId, u8>,
    de_map: HashMap<u8, Box<dyn Package>>,
}

impl PackageMap {
    /// Create a new PackageMap dictionary
    ///
    /// Every package type can only exist once per PackageMap (as unique identification
    /// of a type is the very point of it)
    ///
    /// Example
    /// ```
    /// # use yserde::{PackageMap, Package};
    /// # use tokio::net::TcpStream;
    /// # use std::any::Any;
    /// # #[derive(Debug)]
    /// # struct HelloPkg;
    /// # impl Package for HelloPkg {
    /// #    fn get_new(&self) -> Box<dyn Package> {Box::new(HelloPkg)}
    /// #    fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {Ok(Box::new(HelloPkg))}
    /// # }
    /// # #[derive(Debug, Default)]
    /// # struct ByePkg;
    /// # impl Package for ByePkg {
    /// #    fn get_new(&self) -> Box<dyn Package> {Box::new(ByePkg)}
    /// #    fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {Ok(Box::new(ByePkg))}
    /// # }
    /// let package_kinds = PackageMap::new(vec![
    ///     Box::new(HelloPkg),
    ///     Box::new(ByePkg::default()),
    /// ]);
    /// ```
    pub fn new(list: Vec<Box<dyn Package>>) -> PackageMap {
        let mut type_list: HashSet<TypeId> = HashSet::new();
       let (ser_map, de_map) = list
                                .into_iter()
                                .filter(|p| type_list.insert((&**p).type_id()))
                                .enumerate()
                                .map(|(index, obj)| (((&*obj).type_id(), index as u8), (index as u8, obj)))
                                .unzip();
        PackageMap {
            de_map,
            ser_map,
        }
    }
    /// Convert a [Package] to a vector of bytes
    pub fn pkg_as_bytes(&self, package: Box<dyn Package>) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.push(*self.ser_map.get(&(&*package).type_id()).expect("TypeId of package did not match any registered packages"));
        bytes.extend_from_slice(&package.as_bytes());
        bytes
    }
    /// Receive a [Package] (represented as `Box<dyn Any>`) from a [TcpStream]<br>
    /// The resulting `Box<dyn Any>` can be matched by the [match_pkg] macro
    pub async fn get_from_tcp(&self, tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {
        let mut kind_buf = [0;1];
        tcp.read(&mut kind_buf).await?;
        if let Some(obj) = self.de_map.get(&kind_buf[0]) {
            let obj = obj.get_new();
            tcp.readable().await?;
            return obj.from_bytes(tcp);
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
/// # use yserde::{match_pkg, Package, PackageMap};
/// # use std::any::Any;
/// # use tokio::net::TcpStream;
/// # #[derive(Debug)]
/// # struct Hello;
/// # #[derive(Debug)]
/// # struct Message;
/// # impl Package for Hello {
/// #    fn get_new(&self) -> Box<dyn Package> {Box::new(Hello)}
/// #    fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {Ok(Box::new(Hello))}
/// # }
/// # impl Package for Message {
/// #    fn get_new(&self) -> Box<dyn Package> {Box::new(Message)}
/// #    fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {Ok(Box::new(Message))}
/// # }
/// # let packages = PackageMap::new(vec![Box::new(Hello), Box::new(Message)]);
/// # let rt = tokio::runtime::Runtime::new().unwrap();
/// # rt.block_on(async {
/// # if let Ok(mut receiver) = TcpStream::connect("127.0.0.1:9983").await {
/// match_pkg!(
///     packages.get_from_tcp(&mut receiver).await.unwrap(),
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
