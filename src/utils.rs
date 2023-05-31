#[cfg(test)]
pub mod tests {
    use crate::config::Workspace;
    use poem::test::TestJson;
    use serde::Deserialize;
    use tempdir::TempDir;

    #[derive(Debug, Deserialize)]
    struct Reply {
        status: u16,
    }

    pub fn setup_workspace() -> (TempDir, Workspace) {
        let tmp_dir = TempDir::new("sachima").unwrap();
        let wk: Workspace = tmp_dir.path().try_into().unwrap();

        (tmp_dir, wk)
    }

    /// assert the business status code
    pub fn assert_buss_status(expected: u16, reply: TestJson) {
        let reply: Reply = reply.value().deserialize();
        assert_eq!(expected, reply.status);
    }
}
