use cheetah_relay_common::commands::command::float_counter::{IncrementFloat64C2SCommand, SetFloat64Command};
use cheetah_relay_common::commands::command::long_counter::SetLongCommand;
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::constants::FieldID;

use crate::room::command::{CommandContext, ServerObjectCommandExecutor};
use crate::room::object::GameObject;

impl GameObject {
    pub fn set_float(&mut self, field_id: FieldID, value: f64, context: &CommandContext) {
        self.fields.floats.insert(field_id, value);
        self.send_to_clients(context, |_, object_id| {
            S2CCommandUnion::SetFloat64(SetFloat64Command {
                object_id,
                field_id,
                value,
            })
        });
    }


    pub fn get_float(&self, field_id: FieldID) -> f64 {
        *self.fields.floats.get(&field_id).unwrap_or(&0.0)
    }
}


impl ServerObjectCommandExecutor for IncrementFloat64C2SCommand {
    fn execute(self, object: &mut GameObject, context: &CommandContext) {
        let value = object.get_float(self.field_id);
        object.set_float(self.field_id, value + self.increment, context);
    }
}


impl ServerObjectCommandExecutor for SetFloat64Command {
    fn execute(self, object: &mut GameObject, context: &CommandContext) {
        object.set_float(self.field_id, self.value, context);
    }
}
