# redbot

(Unofficial) Rust bindings for Reddit's API.

[crates.io link](https://crates.io/crates/redbot)

Reddit's API documentation can be found [here](https://github.com/reddit-archive/reddit/wiki/API) and
endpoint documentation can be found [here](https://www.reddit.com/dev/api).


## Usage

### Example

```
use redbot::{Api, Config, Value};

fn main() {
    let config = Config::load_config("config.json").expect("Could not load confiog");
    let mut api = Api::new(config);
    api.do_login().expect("Could not perform login");

    let mut resp = match api.query("GET", "api/v1/me/karma", None, None) {
        Ok(resp) => resp,
        Err(err) => panic!(err),
    };
    let karma_breakdown: Value = match resp.json() {
        Ok(data) => data,
        Err(err) => panic!(err),
    };

    println!("{:?}", karma_breakdown);
}
```
