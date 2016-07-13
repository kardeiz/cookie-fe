#[macro_use]
extern crate iron;

extern crate cookie_fe;
extern crate router;
extern crate time;

use iron::prelude::*;
use iron::status;

use iron::AroundMiddleware;

use router::Router;

use cookie_fe::{Builder as CookieBuilder, Util as CookieUtil, CookiePair};

const KEY: &'static [u8] = b"4b8eee793a846531d6d95dd66ae48319";

fn root(req: &mut Request) -> IronResult<Response> {

    let mut res = Response::with((status::Ok));

    let jar = iexpect!(req.extensions.get_mut::<CookieUtil>()
        .and_then(|x| x.jar()));

    let cookie = CookiePair::new("foo".to_string(), 
        format!("{}", time::now().rfc3339()));

    let old = jar.signed().find("foo")
        .map(|x| x.value )
        .unwrap_or_else(|| "none".to_string() );

    jar.signed().add(cookie);

    res.set_mut(old);

    Ok(res)
}

fn main() {
    let mut router = Router::new();
    router.get("/", root);
    let chain = Chain::new(router);
    let wrapped = CookieBuilder::new(KEY).around(Box::new(chain));
    Iron::new(wrapped).http("0.0.0.0:3000").unwrap();
}
