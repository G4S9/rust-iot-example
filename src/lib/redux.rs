use std::future::Future;
use std::pin::Pin;

type Thunk<AppState, AppAction> = Box<
    dyn for<'a> FnOnce(
        &'a mut Store<AppState, AppAction>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + 'a>>,
>;

pub enum Action<AppState, AppAction> {
    Direct(AppAction),
    Thunk(Thunk<AppState, AppAction>),
}

pub struct Store<AppState, AppAction> {
    state: AppState,
    reducer: fn(AppState, AppAction) -> AppState,
}

impl<AppState, AppAction> Store<AppState, AppAction>
where
    AppState: Clone,
{
    pub fn new(state: AppState, reducer: fn(AppState, AppAction) -> AppState) -> Self {
        Self { state, reducer }
    }
    pub async fn dispatch(&mut self, action: Action<AppState, AppAction>) -> anyhow::Result<()> {
        match action {
            Action::Direct(action) => {
                self.state = (self.reducer)(self.state.clone(), action);
            }
            Action::Thunk(thunk) => {
                thunk(self).await?;
            }
        };
        Ok(())
    }
    pub fn select<'a: 'b, 'b, T: 'b>(&'a self, selector: fn(&'b AppState) -> T) -> T {
        selector(&self.state)
    }
}
