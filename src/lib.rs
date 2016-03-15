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

    pub fn jar<'a>(req: &'a mut Request) -> &'a CookieJar<'static> {
        if let Some(mut util) = req.extensions.get_mut::<Util>() {
            if let Some(ref j) = util.1 {
                return &j;
            } else {
                util.1 = Some(CookieJar::new(util.0));
                if let Some(ref j) = util.1 { return &j; }
            }
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
            let delta = Util::jar(req).delta();
            if !delta.is_empty() {
                r.headers.set(SetCookie(delta));
            }      
        }

        res
    }
}
