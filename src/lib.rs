extern crate iron;
extern crate cookie;

use iron::prelude::*;
use iron::{AroundMiddleware, Handler, typemap};

use iron::headers::{Cookie, SetCookie};

pub use cookie::CookieJar;
pub use cookie::Cookie as CookiePair;

pub struct CookieWrapper(pub &'static [u8]);

pub struct CookieJarProxy;

impl typemap::Key for CookieJarProxy { type Value = CookieJar<'static>; }

struct CookieWrapperHandler<H: Handler> { 
  cookie_wrapper: CookieWrapper, 
  handler: H
}

impl<H: Handler> Handler for CookieWrapperHandler<H> {
  fn handle(&self, req: &mut Request) -> IronResult<Response> {
    
    if let Some(cookie) = req.headers.get::<Cookie>() {
      let jar = cookie.to_cookie_jar(self.cookie_wrapper.0);
      req.extensions.insert::<CookieJarProxy>(jar);
    }
        
    let mut res = self.handler.handle(req);

    if let Ok(&mut ref mut _res) = res.as_mut() {
      if let Some(jar) = req.cookie_jar() {
        _res.headers.set(SetCookie( jar.delta() ));
      }
    }

    res
  }
}

impl AroundMiddleware for CookieWrapper {
  fn around(self, handler: Box<Handler>) -> Box<Handler> {
    Box::new(CookieWrapperHandler {
      cookie_wrapper: self,
      handler: handler
    }) as Box<Handler>
  }
}

pub trait WithCookieJar {
  fn cookie_jar(&self) -> Option<&CookieJar<'static>>;
}

impl<'a, 'b> WithCookieJar for Request<'a, 'b> {
  fn cookie_jar(&self) -> Option<&CookieJar<'static>> {
    self.extensions.get::<CookieJarProxy>()
  }
}

