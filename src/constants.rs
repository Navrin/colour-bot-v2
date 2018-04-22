pub mod commands {
    /// This defines the accepted amount of distance between two similar strings when fuzzy searching a list of strings.
    pub const MAX_STRING_COMPARE_DELTA: usize = 3;

    pub mod roles_edit {
        pub const COLOUR_DESCRIPTION: &str = "The hex colour that will be assigned";
        pub const NAME_DESCRIPTION: &str = "The name that is used in the list of colours";
        pub const ROLE_NAME_DESCRIPTION: &str =
            "The internal role name that discord uses. \n(editing this will also change the name)";
    }
}
