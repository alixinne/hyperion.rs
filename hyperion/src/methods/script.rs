use super::{Led, Method};

use std::fs;

use rlua::{Function, Lua, MetaMethod, Result, ToLua, UserData, UserDataMethods, Variadic};

use serde_json::Value;
use std::collections::BTreeMap as Map;

/// Dummy LED device which outputs updates to the standard output
pub struct Script {
    lua: Lua,
}

#[derive(Debug, Fail)]
pub enum ScriptError {
    #[fail(display = "loading the script failed: {}", 0)]
    LoadError(failure::Error),
    #[fail(display = "lua error: {}", 0)]
    LuaError(rlua::Error),
}

impl From<rlua::Error> for ScriptError {
    fn from(lua_error: rlua::Error) -> Self {
        ScriptError::LuaError(lua_error)
    }
}

impl Script {
    fn to_lua_value<'lua>(lua_ctx: rlua::Context<'lua>, value: &Value) -> rlua::Result<rlua::Value<'lua>> {
        match value {
            Value::Null => Ok(rlua::Value::Nil),
            Value::Bool(bool_value) => Ok(rlua::Value::Boolean(*bool_value)),
            Value::Number(number_value) => {
                if number_value.is_i64() {
                    Ok(rlua::Value::Integer(number_value.as_i64().unwrap()))
                } else {
                    Ok(rlua::Value::Number(number_value.as_f64().unwrap()))
                }
            },
            Value::String(string_value) => {
                Ok(rlua::Value::String(lua_ctx.create_string(&string_value)?))
            },
            Value::Array(array_value) => {
                let table = lua_ctx.create_table()?;

                for (i, item) in array_value.iter().enumerate() {
                    table.set(i + 1, Self::to_lua_value(lua_ctx, item)?)?;
                }

                Ok(rlua::Value::Table(table))
            },
            Value::Object(object_value) => {
                let table = lua_ctx.create_table()?;

                for (k, item) in object_value.iter() {
                    table.set(k.to_string(), Self::to_lua_value(lua_ctx, item)?)?;
                }

                Ok(rlua::Value::Table(table))
            }
        }
    }

    pub fn new(path: String, params: Map<String, Value>) -> std::result::Result<Self, ScriptError> {
        let lua = Lua::new();

        match lua.context(|lua_ctx| -> std::result::Result<(), failure::Error> {
            // Set params table
            let globals = lua_ctx.globals();

            let params_table = lua_ctx.create_table()?;
            for (key, value) in params.iter() {
                params_table.set(key.to_string(), Self::to_lua_value(lua_ctx, value)?)?;
            }

            globals.set("hyperion_params", params_table)?;

            // Load script
            lua_ctx.load(&fs::read_to_string(path)?).exec()?;

            Ok(())
        }) {
            Ok(_) => Ok(Self { lua }),
            Err(error) => Err(ScriptError::LoadError(error)),
        }
    }
}

impl Method for Script {
    fn write(&self, leds: &[Led]) {
        self.lua
            .context(|lua_ctx| -> std::result::Result<(), ScriptError> {
                let globals = lua_ctx.globals();

                let write_function: Function = globals.get("write")?;

                let led_table = lua_ctx.create_table()?;
                for (i, led) in leds.iter().enumerate() {
                    let color_data = lua_ctx.create_table()?;

                    let (r, g, b) = led.current_color.into_components();
                    color_data.set("r", r)?;
                    color_data.set("g", g)?;
                    color_data.set("b", b)?;

                    led_table.set(i + 1, color_data)?;
                }

                write_function.call::<_, ()>(led_table)?;

                Ok(())
            })
            .expect("failed to write LED data");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_method() {
        let mut map = Map::new();
        map.insert("version".to_owned(), Value::String(format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))));

        let method: Box<dyn Method> =
            Box::new(Script::new("scripts/methods/stdout.lua".into(), map).unwrap());
        let leds = vec![Led::default()];

        method.write(&leds[..]);
    }
}
