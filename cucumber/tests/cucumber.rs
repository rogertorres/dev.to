use cucumber_rust::{async_trait, Cucumber, World};
use std::convert::Infallible;

pub enum MyWorld {
    Init,
    Input(i32, i32),
    Result(i32),
    Error,
}

#[async_trait(?Send)]
impl World for MyWorld {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        Ok(Self::Init)
    }
}

mod test_steps {
    use crate::MyWorld;
    use cucumber_rust::Steps;
    use bdd::mult;

    pub fn steps() -> Steps<MyWorld> {
        let mut builder: Steps<MyWorld> = Steps::new();

        builder.given_regex(
            // This will match the "given" of multiplication
            r#"^the numbers "(\d)" and "(\d)"$"#,
            // and store the values inside context, which is a Vec<String>
            |_world, context| {
                // We start from [1] because [0] is the entire regex match
                let world = MyWorld::Input(
                    context.matches[1].parse::<i32>().unwrap(),
                    context.matches[2].parse::<i32>().unwrap(),
                );
                world
            }
        );

        builder.when(
            "the User multiply them", 
            |world, _context|{
                match world {
                    MyWorld::Input(l, r) => MyWorld::Result(mult(l,r)),
                    _ => MyWorld::Error,
                }
            }
        );

        builder.then_regex(
            r#"^the User gets "(\d)" as result$"#, 
            |world, context|{
                match world {
                    MyWorld::Result(x) => assert_eq!(x.to_string(), context.matches[1]),
                    _ => panic!("Invalid world state"),
                };
                MyWorld::Init
            }
        );

        builder
    }
}

#[tokio::main]
async fn main() {
    Cucumber::<MyWorld>::new()
        .features(&["./features"])
        .steps(test_steps::steps())
        .run_and_exit()
        .await
}
