#[derive(PartialEq, Eq)]
enum CacheState {
    NotChanged,
    Changed,
    Created,
}

pub struct CacheVal<T: Clone + Eq> {
    // TODO: Make this a map from block hash -> value or a list of (block_number,value) pairs to keep older values
    old_val: Option<T>,
    current_val: Option<T>,
    state: CacheState,
}

impl<T: Clone + Eq> CacheVal<T> {
    pub fn new_not_changed(old_value: &T) -> Self {
        let old_val = Some(old_value.clone());
        let current_val = Some(old_value.clone());
        Self {
            old_val,
            current_val,
            state: CacheState::NotChanged,
        }
    }

    pub fn new_changed(old_value: &T, current_value: &Option<T>) -> Self {
        let old_val = Some(old_value.clone());
        let current_val = current_value.clone();
        Self {
            old_val,
            current_val,
            state: CacheState::Changed,
        }
    }

    pub fn new_created(current_value: &T) -> Self {
        let current_val = Some(current_value.clone());
        Self {
            old_val: None,
            current_val,
            state: CacheState::Created,
        }
    }

    pub fn get_current(&self) -> Option<T> {
        self.current_val.clone()
    }

    pub fn get_old(&self) -> Option<T> {
        self.old_val.clone()
    }

    pub fn is_changed(&self) -> bool {
        if self.state == CacheState::NotChanged {
            return false;
        }
        if self.current_val.is_none() && self.old_val.is_none() {
            return false;
        }
        if self.current_val.is_some()
            && self.old_val.is_some()
            && self.current_val.as_ref().unwrap() == self.old_val.as_ref().unwrap()
        {
            return false;
        }
        return true;
    }

    pub fn set_current(&mut self, value: &Option<T>) {
        self.current_val = value.clone();
        if self.state == CacheState::NotChanged {
            self.state = CacheState::Changed;
        }
    }
}
