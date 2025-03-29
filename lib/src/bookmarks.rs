pub(crate) trait Bookmark {
    fn get_bookmark(&self) -> Option<&str>;
}
