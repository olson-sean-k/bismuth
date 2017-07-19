error_chain! {
    types {
        CubeError, ErrorKind, ResultExt, Result;
    }
    errors {
        LimitExceeded {
            description("minimum width limit exceeded")
        }
        JoinLeaf {
            description("attempted to join leaf")
        }
        SubdivideBranch {
            description("attempted to subdivide branch")
        }
    }
}
