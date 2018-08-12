use session::Session;
use rpc::*;
use rmpv::Value;
use neovim_api::NeovimApi;
use std::fmt;
use std::error::Error;

pub struct Neovim {
    pub session: Session,
}

pub enum UiOption {
    RGB(bool),
    ExtPopupmenu(bool),
    ExtTabline(bool),
    ExtCmdline(bool),
    ExtWildmenu(bool),
    ExtNewgrid(bool),
}

impl UiOption {
    fn to_value(&self) -> (Value, Value) {
        let name_value = self.to_name_value();
        (name_value.0.into(), name_value.1)
    }

    fn to_name_value(&self) -> (&'static str, Value) {
        match self {
            &UiOption::RGB(val) => ("rgb", val.into()),
            &UiOption::ExtPopupmenu(val) => ("ext_popupmenu", val.into()),
            &UiOption::ExtTabline(val) => ("ext_tabline", val.into()),
            &UiOption::ExtCmdline(val) => ("ext_cmdline", val.into()),
            &UiOption::ExtWildmenu(val) => ("ext_wildmenu", val.into()),
            &UiOption::ExtNewgrid(val) => ("ext_newgrid", val.into()),
        }
    }
}

pub struct UiAttachOptions {
    options: Vec<(&'static str, UiOption)>,
}

impl UiAttachOptions {
    pub fn new() -> UiAttachOptions {
        UiAttachOptions { options: Vec::new() }
    }

    fn set_option(&mut self, option: UiOption) {
        let name = option.to_name_value();
        let position = self.options.iter().position(|o| o.0 == name.0);

        if let Some(position) = position {
            self.options[position].1 = option;
        } else {
            self.options.push((name.0, option));
        }
    }

    pub fn set_rgb(&mut self, rgb: bool) -> &mut Self {
        self.set_option(UiOption::RGB(rgb));
        self
    }

    pub fn set_popupmenu_external(&mut self, popupmenu_external: bool) -> &mut Self {
        self.set_option(UiOption::ExtPopupmenu(popupmenu_external));
        self
    }

    pub fn set_tabline_external(&mut self, tabline_external: bool) -> &mut Self {
        self.set_option(UiOption::ExtTabline(tabline_external));
        self
    }

    pub fn set_cmdline_external(&mut self, cmdline_external: bool) -> &mut Self {
        self.set_option(UiOption::ExtCmdline(cmdline_external));
        self
    }

    pub fn set_wildmenu_external(&mut self, wildmenu_external: bool) -> &mut Self {
        self.set_option(UiOption::ExtWildmenu(wildmenu_external));
        self
    }

    pub fn set_newgrid_external(&mut self, newgrid_external: bool) -> &mut Self {
        self.set_option(UiOption::ExtNewgrid(newgrid_external));
        self
    }

    fn to_value_map(&self) -> Value {
        let map = self.options.iter().map(|o| o.1.to_value()).collect();
        Value::Map(map)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CallError {
    GenericError(String),
    NeovimError(u64, String),
}

impl fmt::Display for CallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CallError::GenericError(ref s) => write!(f, "Unknown error type: {}", s),
            CallError::NeovimError(id, ref s) => write!(f, "{} - {}", id, s),
        }
    }
}

impl Error for CallError {
    fn description(&self) -> &str {
        match *self {
            CallError::GenericError(ref s) => s,
            CallError::NeovimError(_, ref s) => s,
        }
    }
}


#[doc(hidden)]
pub fn map_generic_error(err: Value) -> CallError {
    match err {
        Value::String(val) => CallError::GenericError(val.as_str().unwrap().to_owned()),
        Value::Array(arr) => {
            if arr.len() == 2 {
                match (&arr[0], &arr[1]) {
                    (&Value::Integer(ref id), &Value::String(ref val)) => {
                        CallError::NeovimError(
                            id.as_u64().unwrap(),
                            val.as_str().unwrap().to_owned(),
                        )
                    }
                    _ => CallError::GenericError(format!("{:?}", arr)),
                }
            } else {
                CallError::GenericError(format!("{:?}", arr))
            }
        }
        val => CallError::GenericError(format!("{:?}", val)),
    }
}

#[doc(hidden)]
pub fn map_result<T: FromVal<Value>>(val: Value) -> T {
    T::from_val(val)
}

impl Neovim {
    pub fn new(session: Session) -> Neovim {
        Neovim { session: session }
    }

    /// Register as a remote UI.
    ///
    /// After this method is called, the client will receive redraw notifications.
    pub fn ui_attach(
        &mut self,
        width: u64,
        height: u64,
        opts: &UiAttachOptions,
    ) -> Result<(), CallError> {
        self.session
            .call(
                "nvim_ui_attach",
                call_args!(width, height, opts.to_value_map()),
            )
            .map_err(map_generic_error)
            .map(|_| ())
    }

    /// Send a quit command to Nvim.
    /// The quit command is 'qa!' which will make Nvim quit without
    /// saving anything.
    pub fn quit_no_save(&mut self) -> Result<(), CallError> {
        self.command("qa!")
    }

    /// Same as `ui_set_option` but use `UiOption` as argument to check type at compile time
    pub fn set_option(&mut self, option: UiOption) -> Result<(), CallError> {
        let name_value = option.to_name_value();
        self.ui_set_option(name_value.0, name_value.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_options() {
        let value_map = UiAttachOptions::new()
            .set_rgb(true)
            .set_rgb(false)
            .set_popupmenu_external(true)
            .to_value_map();

        assert_eq!(
            Value::Map(vec![
                ("rgb".into(), false.into()),
                ("ext_popupmenu".into(), true.into()),
            ]),
            value_map
        );
    }
}
