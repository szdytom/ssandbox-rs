use ssandbox::container::Container;

fn main() -> Result<(), ssandbox::container::Error> {
    let mut c = Container::new();
    c.start()?;
    c.wait();
    Ok(())
}
