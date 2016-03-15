extern crate iron;
extern crate cookie;

use iron::prelude::*;
use iron::{AroundMiddleware, Handler, typemap};
use iron::headers::{Cookie, SetCookie};

pub use cookie::CookieJar;
pub use cookie::Cookie as CookiePair;

pub struct Builder(pub &'static [u8]);

pub struct Util(&'static [u8], Option<CookieJar<'static>>);

impl Util {

    fn ext_jar<'a>(req: &'a mut Request) -> Option<&'a CookieJar<'static>> {
        req.extensions.get::<Util>().and_then(|x| x.1.as_ref() )
    }

    pub fn jar<'a>(req: &'a mut Request) -> &'a CookieJar<'static> {
        if let Some(mut util) = req.extensions.get_mut::<Util>() {
            if util.1.is_none() { 
                util.1 = Some(CookieJar::new(util.0));
            }
            if let Some(ref j) = util.1 { return &j; }
        }
        panic!("Cannot use cookie jar in this location");
    }

}

impl typemap::Key for Util { type Value = Self; }

impl AroundMiddleware for Builder {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(Wrapper {
            builder: self,
            handler: handler
        }) as Box<Handler>
    }
}


struct Wrapper<H: Handler> { 
    builder: Builder, 
    handler: H
}

impl<H: Handler> Handler for Wrapper<H> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {

        let jar = req.headers.get::<Cookie>()
            .map(|x| x.to_cookie_jar(self.builder.0) );

        let util = Util(self.builder.0, jar);

        req.extensions.insert::<Util>(util);
        
        let mut res = self.handler.handle(req);

        if let Ok(&mut ref mut r) = res.as_mut() {
            if let Some(jar) = Util::ext_jar(req) {
                println!("{:?}", "setting jar");
                let delta = jar.delta();
                if !delta.is_empty() {
                    r.headers.set(SetCookie(delta));
                } 
            }                 
        }

        res
    }
}
