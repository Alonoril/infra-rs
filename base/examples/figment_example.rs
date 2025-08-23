use figment::Figment;
use figment::providers::{Format, Toml};

fn main() {
    let yaml_str = r#"
    foo: bar
    baz: qux
"#;

    let config = Figment::from(Toml::string(yaml_str));
    println!("{:?}", config);
}
