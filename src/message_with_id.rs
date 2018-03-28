pub trait MessageWithId {
    type Id;

    fn id(&self) -> Self::Id;
}
