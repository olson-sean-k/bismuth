error_chain! {
    types {
        FrameworkError, ErrorKind, ResultExt, Result;
    }
    errors {
        Activity {
            description("activity error")
            display("activity error")
        }
        ActivityStack {
            description("activity stack error")
            display("activity stack error")
        }
    }
}
