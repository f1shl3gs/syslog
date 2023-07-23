#[derive(Clone, Debug, Eq)]
pub struct StructuredElement<S: AsRef<str> + Ord + Clone> {
    pub id: S,
    pub params: Vec<(S, S)>,
}

impl<S: AsRef<str> + Ord + Clone> PartialEq for StructuredElement<S> {
    fn eq(&self, other: &Self) -> bool {
        if self.id.as_ref() != other.id.as_ref() {
            return false;
        }

        let mut params1 = self.params.clone();
        params1.sort();

        let mut params2 = other.params.clone();
        params2.sort();

        params1
            .iter()
            .zip(params2)
            .all(|((ref name1, ref value1), (ref name2, ref value2))| {
                name1.as_ref() == name2.as_ref() && value1.as_ref() == value2.as_ref()
            })
    }
}
